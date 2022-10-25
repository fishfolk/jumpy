use std::any::TypeId;

use bevy::{reflect::FromReflect, utils::HashMap};
use once_cell::sync::Lazy;
use ulid::Ulid;

use crate::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod client;
// pub mod commands;
// pub mod frame_sync;
pub mod serialization;
pub mod server;

#[cfg(not(target_arch = "wasm32"))]
pub type Connection = quinn::Connection;

/// On web we create just enough of a stand-in for `quinn::Connection`, because we can't actually
/// use it in WASM yet.
///
/// We don't do networking on WASM yet, though, so it doesn't need to function.
#[cfg(target_arch = "wasm32")]
pub use wasm_conn::Connection;
#[cfg(target_arch = "wasm32")]
mod wasm_conn {
    use bytes::Bytes;

    #[derive(Clone, Debug)]
    pub struct Connection;

    impl Connection {
        pub fn close_reason(&self) -> Option<()> {
            None
        }

        pub async fn open_uni(&self) -> anyhow::Result<SendStream> {
            Ok(SendStream)
        }

        pub async fn accept_uni(&self) -> anyhow::Result<RecvStream> {
            Ok(RecvStream)
        }

        pub async fn read_datagram(&self) -> anyhow::Result<Vec<u8>> {
            Ok(Vec::new())
        }
        pub fn send_datagram(&self, _: Bytes) -> anyhow::Result<()> {
            Ok(())
        }
        pub async fn closed(&self) -> String {
            String::new()
        }
    }

    pub struct RecvStream;

    impl RecvStream {
        pub async fn read_to_end(&self, _: usize) -> anyhow::Result<Vec<u8>> {
            Ok(Vec::new())
        }
    }

    pub struct SendStream;

    impl SendStream {
        pub async fn write_all(&mut self, _: &[u8]) -> anyhow::Result<()> {
            Ok(())
        }
        pub async fn finish(self) -> anyhow::Result<()> {
            Ok(())
        }
    }
}

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_arch = "wasm32")]
        let _ = app;

        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugin(serialization::SerializationPlugin)
            .add_plugin(server::ServerPlugin)
            .add_plugin(client::ClientPlugin);

        // .add_plugin(commands::NetCommandsPlugin)
        // .add_plugin(frame_sync::NetFrameSyncPlugin)
        // .add_system(handle_server_events.run_if_resource_exists::<RenetServer>())
        // .add_system(client_handle_block_messages.run_if(client_connected))
        // .add_startup_system(setup_synced_queries.exclusive_system());
    }
}

pub static NET_MESSAGE_TYPES: Lazy<Vec<TypeId>> =
    Lazy::new(|| [TypeId::of::<server::Ping>(), TypeId::of::<server::Pong>()].to_vec());

// /// Run condition for running systems if the client is connected
// fn client_connected(client: Option<Res<NetClient>>) -> bool {
//     client
//         .map(|client| client.conn().close_reason().is_none())
//         .unwrap_or(false)
// }

// fn setup_synced_queries(world: &mut World) {
//     world.init_component::<PlayerIdx>();
//     world.init_component::<Transform>();
//     world.init_component::<AnimationBankSprite>();
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
