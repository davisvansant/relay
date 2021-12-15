use std::net::SocketAddr;

use crate::channels::StateSender;

pub struct Server {
    socket_address: SocketAddr,
    sender: StateSender,
}

impl Server {
    pub async fn init(
        socket_address: SocketAddr,
        sender: StateSender,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        Ok(Server {
            socket_address,
            sender,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::{StateRequest, StateResponse};
    use std::str::FromStr;
    use tokio::sync::{mpsc, oneshot};

    #[tokio::test(flavor = "multi_thread")]
    async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let test_address = SocketAddr::from_str("127.0.0.1:1806")?;
        let (test_state_sender, test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        drop(test_state_receiver);

        let test_server = Server::init(test_address, test_state_sender).await;

        assert!(test_server.is_ok());

        Ok(())
    }
}
