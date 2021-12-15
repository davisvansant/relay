use futures_util::stream::SplitSink;
use futures_util::StreamExt;

use std::net::SocketAddr;

use tokio::sync::mpsc;

use warp::ws::{Message, WebSocket, Ws};
use warp::{ws, Filter};

use uuid::Uuid;

use crate::channels::{StateSender, WebSocketReceiver, WebSocketSender};

pub struct Server {
    socket_address: SocketAddr,
    sender: StateSender,
}

impl Server {
    pub async fn init(
        socket_address: SocketAddr,
        sender: StateSender,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        Ok(Server {
            socket_address,
            sender,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state_sender_ownership = self.sender.to_owned();
        let state_channel = warp::any().map(move || state_sender_ownership.to_owned());

        let filter = warp::path("ws")
            .and(ws())
            .and(state_channel)
            .map(|ws: Ws, state_channel| {
                ws.on_upgrade(|connection| async move {
                    if let Err(error) = Self::handle(connection, state_channel).await {
                        println!("connection error -> {:?}", error);
                    }
                })
            });

        println!("server running -> {:?}", self.socket_address);

        warp::serve(filter).run(self.socket_address).await;

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

        tokio::spawn(async move {
            if let Err(error) = Server::incoming_connection(&mut sink_receiver, &mut sink).await {
                println!("error handling incoming connection -> {:?}", error);
            }
        });

        tokio::spawn(async move {
            if let Err(error) =
                Server::initial_messages(initial_state_sender, &uuid, sink_sender).await
            {
                println!("error running initial connection tasks -> {:?}", error);
            }
        });

        while let Some(incoming) = stream.next().await {
            match incoming {
                Ok(message) => {
                    if message.is_text() {
                        println!("received text -> {:?}", &message);

                        unimplemented!();
                    }
                    if message.is_binary() {
                        println!("received binary -> {:?}", &message);

                        unimplemented!();
                    }
                    if message.is_ping() {
                        println!("received ping -> {:?}", &message);

                        unimplemented!();
                    }
                    if message.is_pong() {
                        println!("received pong -> {:?}", &message);

                        unimplemented!();
                    }
                    if message.is_close() {
                        println!("received close -> {:?}", &message);

                        unimplemented!();
                    }
                }
                Err(error) => println!("incoming websocket connection error -> {:?}", error),
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
        // unimplemented!();

        Ok(())
    }

    async fn initial_messages(
        state: StateSender,
        uuid: &str,
        websocket_sender: WebSocketSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // unimplemented!();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::{StateRequest, StateResponse};
    use std::str::FromStr;
    use tokio::sync::{mpsc, oneshot};

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_address = SocketAddr::from_str("127.0.0.1:1806")?;
        let (test_state_sender, test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        drop(test_state_receiver);

        let test_server = Server::init(test_address, test_state_sender).await;

        assert!(test_server.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, mut test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

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

        test_client.send(Message::close()).await;

        let test_response = test_client.recv().await;

        assert!(!test_response.is_ok());

        Ok(())
    }
}
