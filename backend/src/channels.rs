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
