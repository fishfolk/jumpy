#![doc = include_str!("./networking.md")]

use ggrs::P2PSession;
use rand::Rng;

use crate::prelude::*;

// pub mod client;
pub mod matchmaking;
pub mod proto;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkEndpoint>();
    }
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

#[derive(Resource, Deref, DerefMut)]
pub struct NetworkEndpoint(pub quinn::Endpoint);

/// Bind the network socket when the game starts.
impl Default for NetworkEndpoint {
    fn default() -> Self {
        // Generate certificate
        let (cert, key) = cert::generate_self_signed_cert().unwrap();

        let mut transport_config = quinn::TransportConfig::default();
        transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(5)));

        let mut server_config =
            quinn::ServerConfig::with_single_cert([cert].to_vec(), key).unwrap();
        server_config.transport = Arc::new(transport_config);

        // Open Socket and create endpoint
        let port = rand::thread_rng().gen_range(10000..=11000); // Bind a random port
        let socket = std::net::UdpSocket::bind(("0.0.0.0", port)).unwrap();
        let endpoint = quinn::Endpoint::new(
            quinn::EndpointConfig::default(),
            Some(server_config),
            socket,
            quinn_runtime_bevy::BevyIoTaskPoolExecutor,
        )
        .unwrap();

        Self(endpoint)
    }
}

pub mod cert {
    pub fn generate_self_signed_cert() -> anyhow::Result<(rustls::Certificate, rustls::PrivateKey)>
    {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
        let key = rustls::PrivateKey(cert.serialize_private_key_der());
        Ok((rustls::Certificate(cert.serialize_der()?), key))
    }
}

pub use lan::*;
mod lan;
