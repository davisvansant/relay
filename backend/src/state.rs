use std::collections::HashMap;
use warp::filters::ws::Message;

use crate::{error, info};

use crate::channels::{ConnectedUsers, StateReceiver, WebSocketSender};
use crate::channels::{StateRequest, StateResponse};

pub struct State {
    messages: Vec<Message>,
    users: ConnectedUsers,
    receiver: StateReceiver,
}

impl State {
    pub async fn init(receiver: StateReceiver) -> State {
        let messages = Vec::with_capacity(100);
        let users = HashMap::with_capacity(10);

        State {
            messages,
            users,
            receiver,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some((request, response)) = self.receiver.recv().await {
            match request {
                StateRequest::AddMessage(message) => self.add_message(message).await?,
                StateRequest::AddUser((uuid, connection)) => {
                    self.add_user(uuid, connection).await?;

                    if let Err(error) = response.send(StateResponse::Ok) {
                        error!("add user response -> {:?}", error);
                    }
                }
                StateRequest::GetMessages => {
                    let messages = self.get_messages().await?;

                    if let Err(error) = response.send(StateResponse::Messages(messages)) {
                        error!("get messages response -> {:?}", error);
                    }
                }
                StateRequest::GetUser(uuid) => {
                    match self.users.get(&uuid) {
                        Some(user) => {
                            if let Err(error) = response.send(StateResponse::User(user.to_owned()))
                            {
                                error!("get user response -> {:?}", error);
                            }
                        }
                        None => error!("requsted user not found!"),
                    }
                }
                StateRequest::GetUsers => {
                    let users = self.get_users().await;

                    if let Err(error) = response.send(StateResponse::Users(users)) {
                        error!("get user response -> {:?}", error);
                    }
                }
                StateRequest::RemoveUser(uuid) => {
                    self.remove_user(&uuid).await?;

                    if let Err(error) = response.send(StateResponse::Ok) {
                        error!("remove user response -> {:?}", error);
                    }
                }
                StateRequest::Shutdown => {
                    self.receiver.close();
                }
            }
        }

        Ok(())
    }

    async fn add_message(&mut self, message: Message) -> Result<(), Box<dyn std::error::Error>> {
        self.messages.push(message);

        Ok(())
    }

    async fn add_user(
        &mut self,
        uuid: String,
        websocket_sender: WebSocketSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.users.insert(uuid, websocket_sender) {
            Some(key) => {
                info!("updating user -> {:?}", key);
            }
            None => {
                info!("adding new user...");
            }
        }

        Ok(())
    }

    async fn get_messages(&self) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
        info!("getting messages...");

        Ok(self.messages.to_vec())
    }

    async fn get_users(&self) -> ConnectedUsers {
        self.users.clone()
    }

    async fn remove_user(&mut self, uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(entry) = self.users.remove(uuid) {
            info!("removing user -> {:?}", entry);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use tokio::sync::{mpsc, oneshot};
    use uuid::Uuid;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        drop(test_state_sender);

        let test_state = State::init(test_state_receiver).await;

        assert!(test_state.messages.is_empty());
        assert_eq!(test_state.messages.capacity(), 100);

        assert!(test_state.users.is_empty());
        assert!(test_state.users.capacity() >= 10);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn add_message() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        drop(test_state_sender);

        let mut test_state = State::init(test_state_receiver).await;

        assert!(test_state.messages.is_empty());
        assert_eq!(test_state.messages.len(), 0);

        let test_message = Message::text("test_message");

        test_state.add_message(test_message).await?;

        assert!(!test_state.messages.is_empty());
        assert_eq!(test_state.messages.len(), 1);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn add_user() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        drop(test_state_sender);

        let mut test_state = State::init(test_state_receiver).await;

        assert!(test_state.users.is_empty());
        assert_eq!(test_state.users.len(), 0);

        let test_uuid = uuid::Uuid::new_v4().to_string();
        let (test_websocket_sender, test_websocket_receiver) = mpsc::channel(16);

        drop(test_websocket_receiver);

        test_state
            .add_user(test_uuid, test_websocket_sender)
            .await?;

        assert!(!test_state.users.is_empty());
        assert_eq!(test_state.users.len(), 1);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_messages() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        drop(test_state_sender);

        let mut test_state = State::init(test_state_receiver).await;

        let test_message_one = Message::text("test_message_one");
        let test_message_two = Message::text("test_message_two");
        let test_message_three = Message::text("test_message_three");

        test_state.add_message(test_message_one).await?;
        test_state.add_message(test_message_two).await?;
        test_state.add_message(test_message_three).await?;

        let test_messages = test_state.get_messages().await?;

        assert_eq!(test_messages[0].to_str().unwrap(), "test_message_one");
        assert_eq!(test_messages[1].to_str().unwrap(), "test_message_two");
        assert_eq!(test_messages[2].to_str().unwrap(), "test_message_three");

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_users() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        drop(test_state_sender);

        let mut test_state = State::init(test_state_receiver).await;

        assert!(test_state.users.is_empty());
        assert_eq!(test_state.users.len(), 0);

        let test_uuid = uuid::Uuid::new_v4().to_string();
        let (test_websocket_sender, _test_websocket_receiver) = mpsc::channel(16);

        test_state
            .add_user(test_uuid, test_websocket_sender)
            .await?;

        let test_users = test_state.get_users().await;

        assert!(!test_users.is_empty());
        assert_eq!(test_users.len(), 1);

        for (test_uuid, test_websocket_connection) in test_users.iter() {
            assert_eq!(
                Uuid::from_str(test_uuid)
                    .expect("uuid from &str")
                    .get_version_num(),
                4
            );

            assert!(!test_websocket_connection.is_closed());
            assert_eq!(test_websocket_connection.capacity(), 16);
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_user() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        drop(test_state_sender);

        let mut test_state = State::init(test_state_receiver).await;

        assert!(test_state.users.is_empty());
        assert_eq!(test_state.users.len(), 0);

        let test_uuid = uuid::Uuid::new_v4().to_string();
        let test_remove_user = test_uuid.clone();
        let (test_websocket_sender, test_websocket_receiver) = mpsc::channel(16);

        drop(test_websocket_receiver);

        test_state
            .add_user(test_uuid, test_websocket_sender)
            .await?;

        assert!(!test_state.users.is_empty());
        assert_eq!(test_state.users.len(), 1);

        test_state.remove_user(&test_remove_user).await?;

        assert!(test_state.users.is_empty());
        assert_eq!(test_state.users.len(), 0);

        Ok(())
    }
}
