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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::{mpsc, oneshot};

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
}
