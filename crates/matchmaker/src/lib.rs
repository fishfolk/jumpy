#[macro_use]
extern crate tracing;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use once_cell::sync::Lazy;
use quinn::{Endpoint, EndpointConfig, ServerConfig, TransportConfig};
use quinn_smol::SmolExecutor;

pub static EXE: Lazy<SmolExecutor> =
    Lazy::new(|| SmolExecutor(Arc::new(async_executor::Executor::default())));

pub mod cli;

mod certs;
mod game_server;
mod matchmaker;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Config {
    /// The server address to listen on
    #[clap(short, long = "listen", default_value = "0.0.0.0:8943")]
    listen_addr: SocketAddr,
}

async fn server(args: Config) -> anyhow::Result<()> {
    // Put Jumpy in server mode
    std::env::set_var(jumpy::config::SERVER_MODE_ENV_VAR, "true");

    // Generate certificate
    let (cert, key) = certs::generate_self_signed_cert()?;

    let mut transport_config = TransportConfig::default();
    transport_config.keep_alive_interval(Some(Duration::from_secs(5)));

    let mut server_config = ServerConfig::with_single_cert([cert].to_vec(), key)?;
    server_config.transport = Arc::new(transport_config);

    // Open Socket and create endpoint
    let socket = std::net::UdpSocket::bind(args.listen_addr)?;
    let (endpoint, mut incoming) = Endpoint::new(
        EndpointConfig::default(),
        Some(server_config),
        socket,
        EXE.clone(),
    )?;
    info!(address=%endpoint.local_addr()?, "Started server");

    // Listen for incomming connections
    while let Some(connecting) = incoming.next().await {
        let connection = connecting.await;

        match connection {
            Ok(conn) => {
                info!(
                    connection_id = conn.stable_id(),
                    "Accepted connection from client"
                );

                // Spawn a task to handle the new connection
                EXE.spawn(matchmaker::handle_connection(conn)).detach();
            }
            Err(e) => error!("Error opening client connection: {e:?}"),
        }
    }

    info!("Server shutdown");

    Ok(())
}
