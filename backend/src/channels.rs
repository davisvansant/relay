use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot, watch};
use warp::ws::Message;

use crate::info;

pub type ConnectedUsers = HashMap<String, WebSocketSender>;
pub type ShutdownSignal = watch::Receiver<u8>;
pub type StateReceiver = mpsc::Receiver<(StateRequest, oneshot::Sender<StateResponse>)>;
pub type StateSender = mpsc::Sender<(StateRequest, oneshot::Sender<StateResponse>)>;
pub type WebSocketReceiver = mpsc::Receiver<WebSocketConnection>;
pub type WebSocketSender = mpsc::Sender<WebSocketConnection>;

#[derive(Clone, Debug)]
pub enum StateRequest {
    AddMessage(Message),
    AddUser((String, WebSocketSender)),
    GetUser(String),
    GetUsers,
    GetMessages,
    RemoveUser(String),
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum StateResponse {
    Messages(Vec<Message>),
    User(WebSocketSender),
    Users(ConnectedUsers),
    Ok,
}

#[derive(Clone, Debug)]
pub enum WebSocketConnection {
    SendMessage(Message),
    Close,
}

pub async fn add_message(
    state: &StateSender,
    message: &Message,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    state
        .send((StateRequest::AddMessage(message.to_owned()), request))
        .await?;

    Ok(())
}

pub async fn add_user(
    state: &StateSender,
    uuid: String,
    websocket: WebSocketSender,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    state
        .send((StateRequest::AddUser((uuid, websocket)), request))
        .await?;

    match response.await? {
        StateResponse::Ok => {
            info!("successfully added user...");

            Ok(())
        }
        _ => panic!("unexpected response!"),
    }
}

pub async fn get_messages(state: &StateSender) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    state.send((StateRequest::GetMessages, request)).await?;

    match response.await? {
        StateResponse::Messages(messages) => Ok(messages),
        _ => panic!("unexpected response!"),
    }
}

pub async fn get_user(
    state: &StateSender,
    uuid: &str,
) -> Result<WebSocketSender, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    state
        .send((StateRequest::GetUser(uuid.to_owned()), request))
        .await?;

    match response.await? {
        StateResponse::User(user) => Ok(user),
        _ => panic!("unexpected response!"),
    }
}

pub async fn get_users(state: &StateSender) -> Result<ConnectedUsers, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    state.send((StateRequest::GetUsers, request)).await?;

    match response.await? {
        StateResponse::Users(connected_users) => Ok(connected_users),
        _ => panic!("unexpected response!"),
    }
}

pub async fn remove_user(
    state: &StateSender,
    session_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    state
        .send((StateRequest::RemoveUser(session_id.to_owned()), request))
        .await?;

    match response.await? {
        StateResponse::Ok => {
            info!("closing time...");

            Ok(())
        }
        _ => panic!("unexpected response!"),
    }
}

pub async fn shutdown(state: &StateSender) -> Result<(), Box<dyn std::error::Error>> {
    let (_request, _response) = oneshot::channel();

    state.send((StateRequest::Shutdown, _request)).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use uuid::Uuid;

    #[tokio::test(flavor = "multi_thread")]
    async fn add_message() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, mut test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        let test_task = tokio::spawn(async move {
            let mut test_state_messages = Vec::with_capacity(5);

            assert_eq!(test_state_messages.len(), 0);

            while let Some((test_request, _test_response)) = test_state_receiver.recv().await {
                match test_request {
                    StateRequest::AddMessage(test_new_message) => {
                        test_state_messages.push(test_new_message);

                        break;
                    }
                    StateRequest::AddUser(_) => {
                        unimplemented!();
                    }
                    StateRequest::GetMessages => {
                        unimplemented!();
                    }
                    StateRequest::GetUser(_) => unimplemented!(),
                    StateRequest::GetUsers => {
                        unimplemented!();
                    }
                    StateRequest::RemoveUser(_) => {
                        unimplemented!();
                    }
                    StateRequest::Shutdown => {
                        unimplemented!();
                    }
                }
            }

            assert_eq!(test_state_messages.len(), 1);
        });

        let test_message = Message::text("test_message");

        super::add_message(&test_state_sender, &test_message).await?;

        assert!(test_task.await.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn add_user() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, mut test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        let test_task = tokio::spawn(async move {
            let mut test_state_users = HashMap::with_capacity(5);

            assert_eq!(test_state_users.len(), 0);

            while let Some((test_request, test_response)) = test_state_receiver.recv().await {
                match test_request {
                    StateRequest::AddMessage(_) => {
                        unimplemented!();
                    }
                    StateRequest::AddUser((test_id, test_channel)) => {
                        let test_none = test_state_users.insert(test_id, test_channel);

                        assert!(test_none.is_none());

                        test_response.send(StateResponse::Ok).unwrap();

                        break;
                    }
                    StateRequest::GetMessages => {
                        unimplemented!();
                    }
                    StateRequest::GetUser(_) => unimplemented!(),
                    StateRequest::GetUsers => {
                        unimplemented!();
                    }
                    StateRequest::RemoveUser(_) => {
                        unimplemented!();
                    }
                    StateRequest::Shutdown => {
                        unimplemented!();
                    }
                }
            }

            assert_eq!(test_state_users.len(), 1);
        });

        let test_uuid = uuid::Uuid::new_v4().to_string();
        let (test_websocket_sender, test_websocket_receiver) = mpsc::channel(16);

        drop(test_websocket_receiver);

        super::add_user(&test_state_sender, test_uuid, test_websocket_sender).await?;

        assert!(test_task.await.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_messages() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, mut test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        let test_task = tokio::spawn(async move {
            let mut test_state_messages = Vec::with_capacity(5);

            assert_eq!(test_state_messages.len(), 0);

            let test_message = Message::text("test_message");

            test_state_messages.push(test_message);

            assert_eq!(test_state_messages.len(), 1);

            while let Some((test_request, test_response)) = test_state_receiver.recv().await {
                match test_request {
                    StateRequest::AddMessage(_) => {
                        unimplemented!();
                    }
                    StateRequest::AddUser(_) => {
                        unimplemented!()
                    }
                    StateRequest::GetMessages => {
                        let test_messages = test_state_messages.to_vec();

                        test_response
                            .send(StateResponse::Messages(test_messages))
                            .unwrap();

                        break;
                    }
                    StateRequest::GetUser(_) => unimplemented!(),
                    StateRequest::GetUsers => {
                        unimplemented!();
                    }
                    StateRequest::RemoveUser(_) => {
                        unimplemented!();
                    }
                    StateRequest::Shutdown => {
                        unimplemented!();
                    }
                }
            }
        });

        let test_messages = super::get_messages(&test_state_sender).await?;

        assert!(test_task.await.is_ok());
        assert_eq!(test_messages.len(), 1);

        for test_message in &test_messages {
            assert_eq!(test_message.to_str().unwrap(), "test_message");
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_users() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, mut test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        let test_task = tokio::spawn(async move {
            let mut test_state_users = HashMap::with_capacity(5);

            assert_eq!(test_state_users.len(), 0);

            let test_uuid = uuid::Uuid::new_v4().to_string();
            let (test_websocket_sender, test_websocket_receiver) = mpsc::channel(16);

            drop(test_websocket_receiver);

            assert!(test_state_users
                .insert(test_uuid, test_websocket_sender)
                .is_none());
            assert_eq!(test_state_users.len(), 1);

            while let Some((test_request, test_response)) = test_state_receiver.recv().await {
                match test_request {
                    StateRequest::AddMessage(_) => {
                        unimplemented!();
                    }
                    StateRequest::AddUser(_) => {
                        unimplemented!()
                    }
                    StateRequest::GetMessages => {
                        unimplemented!();
                    }
                    StateRequest::GetUser(_) => unimplemented!(),
                    StateRequest::GetUsers => {
                        test_response
                            .send(StateResponse::Users(test_state_users.clone()))
                            .unwrap();

                        break;
                    }
                    StateRequest::RemoveUser(_) => {
                        unimplemented!();
                    }
                    StateRequest::Shutdown => {
                        unimplemented!();
                    }
                }
            }
        });

        let test_users = super::get_users(&test_state_sender).await?;

        assert!(test_task.await.is_ok());
        assert_eq!(test_users.len(), 1);

        for (test_uuid, test_websocket_connection) in test_users.iter() {
            assert_eq!(
                Uuid::from_str(test_uuid)
                    .expect("test uuid")
                    .get_version_num(),
                4,
            );
            assert!(test_websocket_connection.is_closed());
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_user() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, mut test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        let test_uuid = uuid::Uuid::new_v4().to_string();
        let test_lookup_uuid = test_uuid.clone();

        let test_task = tokio::spawn(async move {
            let mut test_state_users = HashMap::with_capacity(5);

            assert_eq!(test_state_users.len(), 0);

            let (test_websocket_sender, test_websocket_receiver) =
                mpsc::channel::<WebSocketConnection>(16);

            drop(test_websocket_receiver);

            test_state_users.insert(test_uuid, test_websocket_sender);

            assert_eq!(test_state_users.len(), 1);

            while let Some((test_request, test_response)) = test_state_receiver.recv().await {
                match test_request {
                    StateRequest::AddMessage(_) => {
                        unimplemented!();
                    }
                    StateRequest::AddUser(_) => {
                        unimplemented!()
                    }
                    StateRequest::GetMessages => {
                        unimplemented!();
                    }
                    StateRequest::GetUser(_) => unimplemented!(),
                    StateRequest::GetUsers => {
                        unimplemented!()
                    }
                    StateRequest::RemoveUser(_) => {
                        test_state_users.clear();

                        test_response.send(StateResponse::Ok).unwrap();

                        break;
                    }
                    StateRequest::Shutdown => {
                        unimplemented!();
                    }
                }
            }

            assert_eq!(test_state_users.len(), 0);
        });

        super::remove_user(&test_state_sender, &test_lookup_uuid).await?;

        assert!(test_task.await.is_ok());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn shutdown() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, mut test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        let test_task = tokio::spawn(async move {
            while let Some((test_request, _test_response)) = test_state_receiver.recv().await {
                match test_request {
                    StateRequest::AddMessage(_) => {
                        unimplemented!();
                    }
                    StateRequest::AddUser(_) => {
                        unimplemented!()
                    }
                    StateRequest::GetMessages => {
                        unimplemented!();
                    }
                    StateRequest::GetUser(_) => unimplemented!(),
                    StateRequest::GetUsers => {
                        unimplemented!()
                    }
                    StateRequest::RemoveUser(_) => {
                        unimplemented!();
                    }
                    StateRequest::Shutdown => {
                        test_state_receiver.close();
                    }
                }
            }
        });

        super::shutdown(&test_state_sender).await?;

        assert!(test_task.await.is_ok());
        assert!(test_state_sender.is_closed());

        Ok(())
    }
}
