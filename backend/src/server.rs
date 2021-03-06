use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};

use std::net::SocketAddr;

use tokio::sync::mpsc;

use warp::ws::{Message, WebSocket, Ws};
use warp::{ws, Filter};

use uuid::Uuid;

use crate::{error, info};

use crate::channels::{
    add_message, add_user, get_messages, get_user, get_users, remove_user, shutdown,
};
use crate::channels::{ShutdownSignal, StateSender, WebSocketConnection, WebSocketReceiver};
use crate::json::{MessageKind, Object};

pub struct Server {
    socket_address: SocketAddr,
    sender: StateSender,
    shutdown_signal: ShutdownSignal,
}

impl Server {
    pub async fn init(
        socket_address: SocketAddr,
        sender: StateSender,
        shutdown_signal: ShutdownSignal,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        Ok(Server {
            socket_address,
            sender,
            shutdown_signal,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state_sender_ownership = self.sender.to_owned();
        let state_channel = warp::any().map(move || state_sender_ownership.to_owned());

        let mut shutdown_signal = self.shutdown_signal.to_owned();
        let send_shutdown = self.sender.to_owned();

        let filter = warp::path("ws")
            .and(ws())
            .and(state_channel)
            .map(|ws: Ws, state_channel| {
                ws.on_upgrade(|connection| async move {
                    if let Err(error) = Self::handle(connection, state_channel).await {
                        error!("connection error -> {:?}", error)
                    }
                })
            });

        info!("socket address -> {:?}", self.socket_address);

        let (_, server) =
            warp::serve(filter).bind_with_graceful_shutdown(self.socket_address, async move {
                shutdown_signal.changed().await.ok();

                if let Ok(()) = shutdown(&send_shutdown).await {
                    info!("shutting down state...")
                }

                info!("shutting down server...")
            });

        server.await;

        Ok(())
    }

    async fn handle(
        connection: WebSocket,
        state_channel: StateSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (mut sink, mut stream) = connection.split();
        let (sink_sender, mut sink_receiver) = mpsc::channel(16);
        let initial_state_sender = state_channel.clone();
        let (session_id, uuid) = Server::create_account().await;

        add_user(&state_channel, session_id.clone(), sink_sender).await?;

        tokio::spawn(async move {
            if let Err(error) = Server::incoming_connection(&mut sink_receiver, &mut sink).await {
                error!("incoming connection -> {:?}", error)
            }
        });

        tokio::spawn(async move {
            if let Err(error) = Server::initial_messages(initial_state_sender, &uuid).await {
                error!("initial connection tasks -> {:?}", error);
            }
        });

        while let Some(incoming) = stream.next().await {
            match incoming {
                Ok(message) => {
                    if message.is_text() {
                        info!("received text -> {:?}", &message);

                        add_message(&state_channel, &message).await?;

                        let connected_users = get_users(&state_channel).await?;
                        let contents = String::from_utf8(message.to_owned().into_bytes())?;
                        let message_object = Object::build(MessageKind::Message, contents).await;
                        let websocket_message = message_object.to_message().await?;

                        for connected_user in connected_users.values() {
                            connected_user
                                .send(WebSocketConnection::SendMessage(
                                    websocket_message.to_owned(),
                                ))
                                .await?;
                        }
                    }
                    if message.is_binary() {
                        info!("received binary -> {:?}", &message);

                        unimplemented!();
                    }
                    if message.is_ping() {
                        info!("received ping -> {:?}", &message);

                        unimplemented!();
                    }
                    if message.is_pong() {
                        info!("received pong -> {:?}", &message);

                        unimplemented!();
                    }
                    if message.is_close() {
                        info!("received close -> {:?}", &message);

                        let current_user = get_user(&state_channel, &session_id).await?;

                        current_user.send(WebSocketConnection::Close).await?;

                        remove_user(&state_channel, &session_id).await?;

                        let remaining_users = get_users(&state_channel).await?;
                        let connected_users_count = Object::build(
                            MessageKind::ConnectedUsers,
                            remaining_users.len().to_string(),
                        )
                        .await;
                        let connected_users_count_message =
                            connected_users_count.to_message().await?;

                        for remaining_user in remaining_users.values() {
                            remaining_user
                                .send(WebSocketConnection::SendMessage(
                                    connected_users_count_message.to_owned(),
                                ))
                                .await?;
                        }
                    }
                }
                Err(error) => {
                    error!(
                        "relay server incoming websocket connection error -> {:?}",
                        error,
                    );
                }
            }
        }

        Ok(())
    }

    async fn create_account() -> (String, String) {
        let session_id = Uuid::new_v4().to_string();
        let uuid = session_id.to_owned();

        (session_id, uuid)
    }

    async fn incoming_connection(
        sink_receiver: &mut WebSocketReceiver,
        sink: &mut SplitSink<WebSocket, Message>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(incoming) = sink_receiver.recv().await {
            match incoming {
                WebSocketConnection::SendMessage(message) => {
                    sink.send(message).await?;
                }
                WebSocketConnection::Close => {
                    sink.close().await?;
                }
            }
        }

        Ok(())
    }

    async fn initial_messages(
        state: StateSender,
        uuid: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connected_users = get_users(&state).await?;
        let older_messages = get_messages(&state).await?;

        if let Some(current_user) = connected_users.get(uuid) {
            let session_uuid = Object::build(MessageKind::Uuid, uuid.to_string()).await;
            let session_uuid_message = session_uuid.to_message().await?;

            current_user
                .send(WebSocketConnection::SendMessage(session_uuid_message))
                .await?;

            for connected_user in connected_users.values() {
                let connected_user_count = Object::build(
                    MessageKind::ConnectedUsers,
                    connected_users.len().to_string(),
                )
                .await;

                let connected_user_count_message = connected_user_count.to_message().await?;

                connected_user
                    .send(WebSocketConnection::SendMessage(
                        connected_user_count_message,
                    ))
                    .await?;
            }

            if older_messages.is_empty() {
                info!("no older messages to send...");
            } else {
                info!("sending older messages ...");

                for message in &older_messages {
                    let contents = String::from_utf8(message.to_owned().into_bytes())?;
                    let older_message = Object::build(MessageKind::Message, contents).await;
                    let older_message_json = older_message.to_message().await?;

                    current_user
                        .send(WebSocketConnection::SendMessage(older_message_json))
                        .await?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::{StateRequest, StateResponse};
    use std::collections::HashMap;
    use std::str::FromStr;
    use tokio::sync::{mpsc, oneshot, watch};

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_address = SocketAddr::from_str("127.0.0.1:1806")?;
        let (test_state_sender, test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);
        let (test_send_shutdown_signal, test_receive_shutdown_signal) = watch::channel(1);

        drop(test_state_receiver);
        drop(test_send_shutdown_signal);

        let test_server = Server::init(
            test_address,
            test_state_sender,
            test_receive_shutdown_signal,
        )
        .await;

        assert!(test_server.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, mut test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        tokio::spawn(async move {
            let mut test_state_messages = Vec::with_capacity(5);
            let mut test_state_users = HashMap::with_capacity(5);

            while let Some((test_request, test_response)) = test_state_receiver.recv().await {
                match test_request {
                    StateRequest::AddMessage(test_new_message) => {
                        test_state_messages.push(test_new_message);
                    }
                    StateRequest::AddUser((test_id, test_channel)) => {
                        let test_none = test_state_users.insert(test_id.to_string(), test_channel);

                        assert!(test_none.is_none());

                        test_response.send(StateResponse::Ok).unwrap();
                    }
                    StateRequest::GetMessages => {
                        let test_messages = test_state_messages.to_vec();

                        test_response
                            .send(StateResponse::Messages(test_messages))
                            .unwrap();
                    }
                    StateRequest::GetUser(test_uuid) => match test_state_users.get(&test_uuid) {
                        Some(test_user) => {
                            test_response
                                .send(StateResponse::User(test_user.to_owned()))
                                .unwrap();
                        }
                        None => assert!(test_state_users.get(&test_uuid).is_none()),
                    },
                    StateRequest::GetUsers => {
                        test_response
                            .send(StateResponse::Users(test_state_users.clone()))
                            .unwrap();
                    }
                    StateRequest::RemoveUser(_) => {
                        test_state_users.clear();
                    }
                    StateRequest::Shutdown => {
                        unimplemented!();
                    }
                }
            }
        });

        let test_state_channel = warp::any().map(move || test_state_sender.to_owned());
        let test_filter = warp::path("ws").and(ws()).and(test_state_channel).map(
            |ws: warp::ws::Ws, test_state_channel| {
                ws.on_upgrade(|test_connection| async move {
                    if let Err(error) = Server::handle(test_connection, test_state_channel).await {
                        println!("there was an error : {:?}", error);
                    }
                })
            },
        );

        let mut test_client = warp::test::ws().path("/ws").handshake(test_filter).await?;

        let test_uuid = test_client.recv().await?;
        let test_uuid_response: Object = serde_json::from_str(test_uuid.to_str().unwrap())?;

        assert_eq!(test_uuid_response.kind, "uuid");

        let test_connected_users = test_client.recv().await?;
        let test_connected_users_response: Object =
            serde_json::from_str(test_connected_users.to_str().unwrap())?;

        assert_eq!(test_connected_users_response.kind, "connected_users");
        assert_eq!(test_connected_users_response.contents, "1");

        test_client.send_text("test_message").await;

        let test_message = test_client.recv().await?;
        let test_message_response: Object = serde_json::from_str(test_message.to_str().unwrap())?;

        assert_eq!(test_message_response.kind, "message");
        assert_eq!(test_message_response.contents, "test_message");

        test_client.send(Message::close()).await;

        let test_close = test_client.recv_closed().await;

        assert!(test_close.is_ok());

        Ok(())
    }
}
