use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

use tokio::signal;
use tokio::sync::{mpsc, oneshot, watch};

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
    let (send_shutdown_signal, receive_shutdown_signal) = watch::channel(1);

    let mut state = State::init(receiver).await;
    let server = Server::init(socket_address, sender, receive_shutdown_signal).await?;

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

    let shutdown_task = tokio::spawn(async move {
        if let Ok(()) = signal::ctrl_c().await {
            println!("received shutdown signal!");

            if let Err(error) = send_shutdown_signal.send(0) {
                println!(
                    "error sending shutdown signal to running tasks -> {:?}",
                    error,
                );
            }
        }
    });

    tokio::try_join!(state_task, server_task, shutdown_task)?;

    Ok(())
}
