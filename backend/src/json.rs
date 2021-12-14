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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn message_kind_connected_users() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_kind_connected_users = MessageKind::ConnectedUsers.build().await;

        assert_eq!(
            test_message_kind_connected_users.as_str(),
            "connected_users",
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_kind_message() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_kind_message = MessageKind::Message.build().await;

        assert_eq!(test_message_kind_message.as_str(), "message");

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn message_kind_uuid() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_kind_uuid = MessageKind::Uuid.build().await;

        assert_eq!(test_message_kind_uuid.as_str(), "uuid");

        Ok(())
    }
}
