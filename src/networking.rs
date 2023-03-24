#![doc = include_str!("./networking.md")]

use ggrs::P2PSession;
use rand::Rng;

use crate::prelude::*;

// pub mod client;
pub mod matchmaking;
pub mod proto;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, _app: &mut App) {}
}

/// The [`ggrs::Config`] implementation used by Jumpy.
#[derive(Debug)]
pub struct GgrsConfig;
impl ggrs::Config for GgrsConfig {
    type Input = proto::DensePlayerControl;
    type State = bones::World;
    /// Addresses are the same as the player handle for our custom socket.
    type Address = usize;
}

/// The QUIC network endpoint used for all network communications.
pub static NETWORK_ENDPOINT: Lazy<quinn::Endpoint> = Lazy::new(|| {
    // Generate certificate
    let (cert, key) = cert::generate_self_signed_cert().unwrap();

    let mut transport_config = quinn::TransportConfig::default();
    transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(5)));

    let mut server_config = quinn::ServerConfig::with_single_cert([cert].to_vec(), key).unwrap();
    server_config.transport = Arc::new(transport_config);

    // Open Socket and create endpoint
    let port = rand::thread_rng().gen_range(10000..=11000); // Bind a random port
    let socket = std::net::UdpSocket::bind(("0.0.0.0", port)).unwrap();

    let client_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(cert::SkipServerVerification::new())
        .with_no_client_auth();
    let client_config = quinn::ClientConfig::new(Arc::new(client_config));

    let mut endpoint = quinn::Endpoint::new(
        quinn::EndpointConfig::default(),
        Some(server_config),
        socket,
        quinn_runtime_bevy::BevyIoTaskPoolExecutor,
    )
    .unwrap();

    endpoint.set_default_client_config(client_config);

    endpoint
});

pub mod cert {
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

    pub fn generate_self_signed_cert() -> anyhow::Result<(rustls::Certificate, rustls::PrivateKey)>
    {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
        let key = rustls::PrivateKey(cert.serialize_private_key_der());
        Ok((rustls::Certificate(cert.serialize_der()?), key))
    }
}

pub use lan::*;
mod lan;
