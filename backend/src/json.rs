pub enum MessageKind {
    Uuid,
    Message,
    ConnectedUsers,
}

impl MessageKind {
    pub async fn build(&self) -> String {
        match self {
            MessageKind::ConnectedUsers => String::from("connected_users"),
            MessageKind::Message => String::from("message"),
            MessageKind::Uuid => String::from("uuid"),
        }
    }
}
