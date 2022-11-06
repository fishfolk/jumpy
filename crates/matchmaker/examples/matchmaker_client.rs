use std::{net::SocketAddr, sync::Arc, time::Duration};

use bevy_tasks::{IoTaskPool, TaskPool};
use certs::SkipServerVerification;
use jumpy_matchmaker_proto::{MatchInfo, MatchmakerRequest, MatchmakerResponse};
use quinn::{ClientConfig, Endpoint, EndpointConfig};
use quinn_bevy::BevyIoTaskPoolExecutor;
use serde::{Deserialize, Serialize};

static SERVER_NAME: &str = "localhost";

fn client_addr() -> SocketAddr {
    "127.0.0.1:0".parse::<SocketAddr>().unwrap()
}

fn server_addr() -> SocketAddr {
    "127.0.0.1:8943".parse::<SocketAddr>().unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Hello {
    i_am: String,
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
    IoTaskPool::init(TaskPool::new);
    let task_pool = IoTaskPool::get();
    futures_lite::future::block_on(task_pool.spawn(async move {
        if let Err(e) = client().await {
            eprintln!("Error: {e}");
        }
    }));
}

async fn client() -> anyhow::Result<()> {
    let client_config = configure_client();
    let socket = std::net::UdpSocket::bind(client_addr())?;
    // Bind this endpoint to a UDP socket on the given client address.
    let endpoint = Endpoint::new(
        EndpointConfig::default(),
        None,
        socket,
        BevyIoTaskPoolExecutor,
    )?;

    let i_am = std::env::args().nth(2).unwrap();
    let hello = Hello { i_am };
    println!("o  Opened client on {}. {hello:?}", endpoint.local_addr()?);

    // Connect to the server passing in the server name which is supposed to be in the server certificate.
    let conn = endpoint
        .connect_with(client_config, server_addr(), SERVER_NAME)?
        .await?;

    // Send a match request to the server
    let (mut send, recv) = conn.open_bi().await?;

    let message = MatchmakerRequest::RequestMatch(MatchInfo {
        client_count: std::env::args()
            .nth(1)
            .map(|x| x.parse().unwrap())
            .unwrap_or(0),
    });
    println!("=> Sending match request: {message:?}");
    let message = postcard::to_allocvec(&message)?;

    send.write_all(&message).await?;
    send.finish().await?;

    println!("o  Waiting for response");

    let message = recv.read_to_end(256).await?;
    let message: MatchmakerResponse = postcard::from_bytes(&message)?;

    if let MatchmakerResponse::Accepted = message {
        println!("<= Request accepted, waiting for match");
    } else {
        panic!("<= Unexpected message from server!");
    }

    loop {
        let recv = conn.accept_uni().await?;
        let message = recv.read_to_end(256).await?;
        let message: MatchmakerResponse = postcard::from_bytes(&message)?;

        match message {
            MatchmakerResponse::ClientCount(count) => {
                println!("<= {count} players in lobby");
            }
            MatchmakerResponse::Success => {
                println!("<= Match is ready!");
                break;
            }
            _ => panic!("<= Unexpected message from server"),
        }
    }

    let task_pool = IoTaskPool::get();

    let conn_ = conn.clone();
    task_pool
        .spawn(async move {
            let result = async move {
                for _ in 0..3 {
                    println!("=> {hello:?}");
                    let mut sender = conn_.open_uni().await?;
                    sender
                        .write_all(&postcard::to_allocvec(&hello.clone())?)
                        .await?;
                    sender.finish().await?;

                    async_io::Timer::after(Duration::from_secs(1)).await;
                }

                Ok::<_, anyhow::Error>(())
            };

            if let Err(e) = result.await {
                eprintln!("<= Error: {e:?}");
            }
        })
        .detach();

    let conn_ = conn.clone();
    task_pool
        .spawn(async move {
            loop {
                let result = async {
                    let recv = conn_.accept_uni().await?;

                    let incomming = recv.read_to_end(256).await?;
                    let message: Hello = postcard::from_bytes(&incomming).unwrap();

                    println!("<= {message:?}");

                    Ok::<_, anyhow::Error>(())
                };
                if let Err(e) = result.await {
                    eprintln!("Error: {e:?}");
                    break;
                }
            }
        })
        .detach();

    async_io::Timer::after(Duration::from_secs(4)).await;

    println!("Closing connection");
    conn.close(0u8.into(), b"done");

    endpoint.close(0u8.into(), b"done");
    endpoint.wait_idle().await;

    Ok(())
}
