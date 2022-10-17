use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    reflect::{FromReflect, TypeRegistryArc},
    utils::HashMap,
};
use bevy_renet::{
    renet::{
        ChannelConfig, ReliableChannelConfig, RenetClient, RenetError, RenetServer, ServerEvent,
    },
    RenetClientPlugin, RenetServerPlugin,
};
use ulid::Ulid;

use crate::prelude::*;

mod net_commands;
pub use net_commands::*;

pub const PROTOCOL_ID: u64 = 0;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetIdMap>()
            .add_plugin(RenetClientPlugin::default())
            .add_plugin(RenetServerPlugin::default())
            .add_system(handle_server_events.run_if_resource_exists::<RenetServer>())
            .add_system(log_renet_errors)
            .add_system(client_handle_net_commands.run_if(client_connected));
    }
}

fn client_connected(client: Option<Res<RenetClient>>) -> bool {
    client.map(|client| client.is_connected()).unwrap_or(false)
}

fn log_renet_errors(mut errors: EventReader<RenetError>) {
    for error in errors.iter() {
        error!("Network error: {}", error);
    }
}

#[derive(
    Reflect,
    FromReflect,
    Component,
    Deref,
    DerefMut,
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
)]
#[reflect_value(PartialEq, Deserialize, Serialize, Hash)]
pub struct NetId(pub Ulid);

impl NetId {
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl From<Ulid> for NetId {
    fn from(u: Ulid) -> Self {
        Self(u)
    }
}

#[repr(u8)]
pub enum NetChannels {
    Commands,
}

impl NetChannels {
    pub fn get_config() -> Vec<ChannelConfig> {
        vec![ChannelConfig::Reliable(ReliableChannelConfig {
            channel_id: NetChannels::Commands as _,
            packet_budget: 3500,
            max_message_size: 3500,
            ..default()
        })]
    }
}

impl From<NetChannels> for u8 {
    fn from(val: NetChannels) -> Self {
        val as u8
    }
}

fn handle_server_events(mut server_events: EventReader<ServerEvent>) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => info!(%id, "Client connected"),
            ServerEvent::ClientDisconnected(id) => info!(%id, "Client disconnected"),
        }
    }
}
