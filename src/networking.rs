use std::time::Duration;

use bevy::{reflect::FromReflect, utils::HashMap};
use bevy_ecs_dynamic::dynamic_query::{DynamicQuery, FetchKind};
use bevy_renet::{
    renet::{
        ChannelConfig, ReliableChannelConfig, RenetClient, RenetError, RenetServer, ServerEvent,
        UnreliableChannelConfig,
    },
    RenetClientPlugin, RenetServerPlugin,
};
use renet_visualizer::RenetServerVisualizer;
use ulid::Ulid;

use crate::{config::ENGINE_CONFIG, player::PlayerIdx, prelude::*, animation::AnimationBankSprite};

use self::frame_sync::{NetworkSyncQueries, NetworkSyncQuery};

pub mod commands;
pub mod frame_sync;

pub const PROTOCOL_ID: u64 = 0;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(frame_sync::NetFrameSyncPlugin)
            .add_plugin(commands::NetCommandsPlugin)
            .add_plugin(RenetClientPlugin::default())
            .add_plugin(RenetServerPlugin::default())
            .add_system(handle_server_events.run_if_resource_exists::<RenetServer>())
            .add_system(log_renet_errors)
            .add_startup_system(setup_synced_queries.exclusive_system());

        if ENGINE_CONFIG.debug_tools {
            app.insert_resource(RenetServerVisualizer::<200>::default());
        }
    }
}

/// Run condition for running systems if the client is connected
fn client_connected(client: Option<Res<RenetClient>>) -> bool {
    client.map(|client| client.is_connected()).unwrap_or(false)
}

fn log_renet_errors(mut errors: EventReader<RenetError>) {
    for error in errors.iter() {
        error!("Network error: {}", error);
    }
}

fn setup_synced_queries(world: &mut World) {
    world.init_component::<PlayerIdx>();
    world.init_component::<Transform>();
    world.init_component::<AnimationBankSprite>();
    world.resource_scope(|world, mut queries: Mut<NetworkSyncQueries>| {
        let query = DynamicQuery::new(
            world,
            [
                FetchKind::Ref(world.component_id::<PlayerIdx>().unwrap()),
                FetchKind::Ref(world.component_id::<Transform>().unwrap()),
                FetchKind::Ref(world.component_id::<AnimationBankSprite>().unwrap()),
            ]
            .to_vec(),
            [].to_vec(),
        )
        .unwrap();

        queries.push(NetworkSyncQuery { query, prune: true })
    });
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
    FrameSync,
    PlayerInput,
}

impl NetChannels {
    pub fn get_config() -> Vec<ChannelConfig> {
        [
            ChannelConfig::Reliable(ReliableChannelConfig {
                channel_id: NetChannels::Commands as _,
                max_message_size: 3500,
                message_resend_time: Duration::ZERO,
                ..default()
            }),
            ChannelConfig::Unreliable(UnreliableChannelConfig {
                channel_id: NetChannels::FrameSync as _,
                packet_budget: 10000,
                max_message_size: 7500,
                ..default()
            }),
            ChannelConfig::Unreliable(UnreliableChannelConfig {
                channel_id: NetChannels::PlayerInput as _,
                ..default()
            }),
        ]
        .to_vec()
    }
}

impl From<NetChannels> for u8 {
    fn from(val: NetChannels) -> Self {
        val as u8
    }
}

#[derive(Clone, Debug, Default)]
pub struct NetIdMap {
    ent_to_net: HashMap<Entity, NetId>,
    net_to_ent: HashMap<NetId, Entity>,
}

impl NetIdMap {
    pub fn insert(&mut self, entity: Entity, net_id: NetId) {
        self.ent_to_net.insert(entity, net_id);
        self.net_to_ent.insert(net_id, entity);
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Option<NetId> {
        if let Some(net_id) = self.ent_to_net.remove(&entity) {
            self.net_to_ent.remove(&net_id);

            Some(net_id)
        } else {
            None
        }
    }

    pub fn remove_net_id(&mut self, net_id: NetId) -> Option<Entity> {
        if let Some(entity) = self.net_to_ent.remove(&net_id) {
            self.ent_to_net.remove(&entity);

            Some(entity)
        } else {
            None
        }
    }

    pub fn get_entity(&self, net_id: NetId) -> Option<Entity> {
        self.net_to_ent.get(&net_id).cloned()
    }

    pub fn get_net_id(&self, entity: Entity) -> Option<NetId> {
        self.ent_to_net.get(&entity).cloned()
    }
}

fn handle_server_events(
    mut server_events: EventReader<ServerEvent>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                info!(%id, "Client connected");
                visualizer.add_client(*id);
            }
            ServerEvent::ClientDisconnected(id) => {
                info!(%id, "Client disconnected");
                visualizer.remove_client(*id);
            }
        }
    }
}
