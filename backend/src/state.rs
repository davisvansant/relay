use std::collections::HashMap;
use warp::filters::ws::Message;

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

    async fn add_message(&mut self, message: Message) -> Result<(), Box<dyn std::error::Error>> {
        self.messages.push(message);

        Ok(())
    }

    async fn add_user(
        &mut self,
        uuid: String,
        websocket_sender: WebSocketSender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("checking user -> {:?}", &uuid);

        match self.users.insert(uuid, websocket_sender) {
            Some(key) => println!("updating user -> {:?}", key),
            None => println!("adding new user..."),
        }

        Ok(())
    }

    async fn get_messages(&self) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
        println!("getting messages...");

        Ok(self.messages.to_vec())
    }

    async fn get_users(&self) -> ConnectedUsers {
        self.users.clone()
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
        let (test_websocket_sender, test_websocket_receiver) = mpsc::channel(16);

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
}
