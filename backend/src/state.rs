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
}
