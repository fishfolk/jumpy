use std::any::TypeId;

use bevy::{reflect::FromReflect, utils::HashMap};
use once_cell::sync::Lazy;
use ulid::Ulid;

use crate::{config::ENGINE_CONFIG, prelude::*};

#[cfg(not(target_arch = "wasm32"))]
pub mod client;
// pub mod commands;
// pub mod frame_sync;
pub mod proto;
pub mod serialization;
pub mod server;

// use self::frame_sync::{NetworkSyncConfig, NetworkSyncQuery};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(serialization::SerializationPlugin)
            .add_plugin(client::ClientPlugin);

        // app
        // .add_plugin(commands::NetCommandsPlugin)
        // .add_plugin(frame_sync::NetFrameSyncPlugin)
        // .add_startup_system(setup_synced_queries.exclusive_system());

        if ENGINE_CONFIG.server_mode {
            app.add_plugin(server::ServerPlugin);
        }
    }
}

pub static NET_MESSAGE_TYPES: Lazy<Vec<TypeId>> = Lazy::new(|| {
    [
        TypeId::of::<proto::Ping>(),
        TypeId::of::<proto::Pong>(),
        TypeId::of::<proto::ClientMatchInfo>(),
        TypeId::of::<proto::match_setup::MatchSetupFromClient>(),
        TypeId::of::<proto::match_setup::MatchSetupFromServer>(),
        TypeId::of::<serialization::TypeNameCache>(),
        TypeId::of::<proto::game::GameEventFromServer>(),
        TypeId::of::<proto::game::GamePlayerEvent>(),
        // TypeId::of::<commands::CommandMessage>(),
        // TypeId::of::<frame_sync::FrameSyncMessage>(),
        // TypeId::of::<proto::player_input::PlayerInputFromClient>(),
        // TypeId::of::<proto::player_input::PlayerInputFromServer>(),
    ]
    .to_vec()
});

// fn setup_synced_queries(world: &mut World) {
//     world.init_component::<crate::player::PlayerIdx>();
//     world.init_component::<Transform>();
//     world.resource_scope(|world, mut network_sync: Mut<NetworkSyncConfig>| {
//         let query = DynamicQuery::new(
//             world,
//             [
//                 FetchKind::Ref(world.component_id::<PlayerIdx>().unwrap()),
//                 FetchKind::Ref(world.component_id::<Transform>().unwrap()),
//             ]
//             .to_vec(),
//             [].to_vec(),
//         )
//         .unwrap();

//         network_sync
//             .queries
//             .push(NetworkSyncQuery { query, prune: true })
//     });
// }

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

impl Default for NetId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Ulid> for NetId {
    fn from(u: Ulid) -> Self {
        Self(u)
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
