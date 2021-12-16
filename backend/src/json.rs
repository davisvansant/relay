use serde::{Deserialize, Serialize};

use warp::filters::ws::Message;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct Object {
    pub kind: String,
    pub contents: String,
}

impl Object {
    pub async fn build(kind: MessageKind, contents: String) -> Object {
        let kind = kind.build().await;

        Object { kind, contents }
    }

    pub async fn to_message(&self) -> Result<Message, Box<dyn std::error::Error>> {
        let json = serde_json::to_string(&self)?;
        let message = Message::text(&json);

        Ok(message)
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

    #[tokio::test(flavor = "multi_thread")]
    async fn object_build() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_kind = MessageKind::Message;
        let test_contents = String::from("test_contents");
        let test_object = Object::build(test_message_kind, test_contents).await;

        assert_eq!(test_object.kind.as_str(), "message");
        assert_eq!(test_object.contents.as_str(), "test_contents");

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn object_to_message() -> Result<(), Box<dyn std::error::Error>> {
        let test_message_kind = MessageKind::Message;
        let test_contents = String::from("test_contents");
        let test_object = Object::build(test_message_kind, test_contents).await;

        assert_eq!(
            test_object
                .to_message()
                .await?
                .to_str()
                .expect("websocket message &str"),
            r#"{"kind":"message","contents":"test_contents"}"#,
        );

        Ok(())
    }
}
