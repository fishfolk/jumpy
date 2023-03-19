// #![doc = include_str!("./networking.md")]

use ggrs::P2PSession;
use rand::Rng;

use crate::prelude::*;

pub mod certs;
pub mod proto;

pub use lan::*;
mod lan;

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
    let (cert, key) = certs::generate_self_signed_cert().unwrap();

    let mut transport_config = quinn::TransportConfig::default();
    transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(5)));

    let mut server_config = quinn::ServerConfig::with_single_cert([cert].to_vec(), key).unwrap();
    server_config.transport = Arc::new(transport_config);

    // Open Socket and create endpoint
    let port = rand::thread_rng().gen_range(10000..=11000); // Bind a random port
    let socket = std::net::UdpSocket::bind(("0.0.0.0", port)).unwrap();

    let client_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(certs::SkipServerVerification::new())
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
