use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use async_executor::Executor;
use certs::SkipServerVerification;
use jumpy_matchmaker_proto::{
    matchmaker::{MatchmakerRequest, MatchmakerResponse},
    ConnectionType,
};
use quinn::{ClientConfig, Endpoint, EndpointConfig, TokioRuntime};
use quinn_smol::SmolExecutor;

static SERVER_NAME: &str = "localhost";

fn client_addr() -> SocketAddr {
    "127.0.0.1:0".parse::<SocketAddr>().unwrap()
}

fn server_addr() -> SocketAddr {
    "127.0.0.1:8943".parse::<SocketAddr>().unwrap()
}

mod certs {
    use std::sync::Arc;

    // Implementation of `ServerCertVerifier` that verifies everything as trustworthy.
    pub struct SkipServerVerification;

    impl SkipServerVerification {
        pub fn new() -> Arc<Self> {
            Arc::new(Self)
        }
    }

    impl rustls::client::ServerCertVerifier for SkipServerVerification {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::Certificate,
            _intermediates: &[rustls::Certificate],
            _server_name: &rustls::ServerName,
            _scts: &mut dyn Iterator<Item = &[u8]>,
            _ocsp_response: &[u8],
            _now: std::time::SystemTime,
        ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
            Ok(rustls::client::ServerCertVerified::assertion())
        }
    }
}

fn configure_client() -> ClientConfig {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    ClientConfig::new(Arc::new(crypto))
}

pub fn main() {
    if let Err(e) = client() {
        eprintln!("Error: {e}");
    }
}

#[tokio::main]
async fn client() -> anyhow::Result<()> {
    let client_config = configure_client();
    let socket = std::net::UdpSocket::bind(client_addr())?;
    // Bind this endpoint to a UDP socket on the given client address.
    let endpoint = Endpoint::new(EndpointConfig::default(), None, socket, TokioRuntime)?.0;

    println!("Opened client on {}", endpoint.local_addr()?);

    // Connect to the server passing in the server name which is supposed to be in the server certificate.
    let conn = endpoint
        .connect_with(client_config, server_addr(), SERVER_NAME)?
        .await?;

    let (mut send, recv) = conn.open_bi().await?;

    let message = ConnectionType::Matchmaker;
    let message = postcard::to_allocvec(&message).context("Serialize connection type")?;
    send.write_all(&message).await?;
    send.finish().await?;
    // Wait for server response before continuing
    recv.read_to_end(1).await?;

    println!("Connection accepted!");

    // Open a bi-directional channel to the server
    let (mut send, recv) = conn.open_bi().await?;

    println!("Sending Request");
    let message = MatchmakerRequest::RequestMatch { players: 4 };
    let message = postcard::to_allocvec(&message)?;

    send.write_all(&message).await?;
    send.finish().await?;

    let message = recv.read_to_end(256).await?;
    let message: MatchmakerResponse = postcard::from_bytes(&message)?;

    println!("Got match ID: {:?}", message);

    conn.close(0u8.into(), b"done");

    Ok(())
}
