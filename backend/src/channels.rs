use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use warp::ws::Message;

pub type ConnectedUsers = HashMap<String, WebSocketSender>;
pub type StateReceiver = mpsc::Receiver<(StateRequest, oneshot::Sender<StateResponse>)>;
pub type StateSender = mpsc::Sender<(StateRequest, oneshot::Sender<StateResponse>)>;
pub type WebSocketReceiver = mpsc::Receiver<WebSocketConnection>;
pub type WebSocketSender = mpsc::Sender<WebSocketConnection>;

#[derive(Clone, Debug)]
pub enum StateRequest {
    AddMessage(Message),
    AddUser((String, WebSocketSender)),
    GetUsers,
    GetMessages,
    RemoveUser(String),
}

#[derive(Clone, Debug)]
pub enum StateResponse {
    Messages(Vec<Message>),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn add_message() -> Result<(), Box<dyn std::error::Error>> {
        let (test_state_sender, mut test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        let test_task = tokio::spawn(async move {
            let mut test_state_messages = Vec::with_capacity(5);

            assert_eq!(test_state_messages.len(), 0);

            while let Some((test_request, test_response)) = test_state_receiver.recv().await {
                match test_request {
                    StateRequest::AddMessage(test_new_message) => {
                        test_state_messages.push(test_new_message);

                        break;
                    }
                    StateRequest::AddUser((test_id, test_channel)) => {
                        unimplemented!();
                    }
                    StateRequest::GetMessages => {
                        unimplemented!();
                    }
                    StateRequest::GetUsers => {
                        unimplemented!();
                    }
                    StateRequest::RemoveUser(_) => {
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
}
