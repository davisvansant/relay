use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

use tokio::sync::{mpsc, oneshot};

mod channels;
mod json;
mod server;
mod state;

use crate::channels::{StateRequest, StateResponse};
use crate::server::Server;
use crate::state::State;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = IpAddr::V4(Ipv4Addr::from_str("0.0.0.0")?);
    let port = 1806;
    let socket_address = SocketAddr::new(address, port);

    let (sender, receiver) = mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

    let mut state = State::init(receiver).await;
    let server = Server::init(socket_address, sender).await?;

    let state_task = tokio::spawn(async move {
        if let Err(error) = state.run().await {
            println!("error with state -> {:?}", error);
        }
    });

    let server_task = tokio::spawn(async move {
        if let Err(error) = server.run().await {
            println!("error with server -> {:?}", error);
        }
    });

    tokio::try_join!(state_task, server_task)?;

    Ok(())
}
