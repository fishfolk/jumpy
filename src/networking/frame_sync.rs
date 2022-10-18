use bevy::{
    ecs::component::ComponentId,
    reflect::{ReflectFromPtr, TypeRegistryArc, TypeRegistryInternal},
};
use bevy_ecs_dynamic::dynamic_query::{DynamicQuery, FetchKind, FetchResult};
use bevy_renet::renet::{RenetClient, RenetServer};

use crate::prelude::*;

use super::{NetChannels, NetId, NetIdMap};

mod dynamic_query_info;
use self::dynamic_query_info::DynamicQueryInfo;

pub struct NetFrameSyncPlugin;

impl Plugin for NetFrameSyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkSyncQueries>()
            .add_system_to_stage(
                FixedUpdateStage::First,
                client_get_sync_data.exclusive_system().at_start(),
            )
            .add_system_to_stage(
                FixedUpdateStage::Last,
                server_send_sync_data.exclusive_system().at_end(),
            );
    }
}

#[derive(Default, Deref, DerefMut)]
pub struct NetworkSyncQueries(Vec<NetworkSyncQuery>);

pub struct NetworkSyncQuery {
    query: DynamicQuery,
    prune: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FrameSyncMessage {
    queries: Vec<FrameSyncQuery>,
}

type Bytes = Vec<u8>;
type ComponentsBytes = Vec<Bytes>;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FrameSyncQuery {
    query_info: DynamicQueryInfo,
    items: Vec<FrameSyncQueryItem>,
    prune: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FrameSyncQueryItem {
    net_id: NetId,
    components: ComponentsBytes,
}

fn client_get_sync_data(world: &mut World) {
    if !world.contains_resource::<RenetClient>() {
        return;
    }

    world.resource_scope(|world, mut client: Mut<RenetClient>| {
        world.resource_scope(|world, net_ids: Mut<NetIdMap>| {
            world.resource_scope(|world, type_registry: Mut<TypeRegistryArc>| {
                let type_registry = type_registry.read();
                impl_client_get_sync_data(world, &mut client, &net_ids, &*type_registry)
            });
        });
    });
}

fn impl_client_get_sync_data(
    world: &mut World,
    client: &mut RenetClient,
    net_ids: &NetIdMap,
    type_registry: &TypeRegistryInternal,
) {
    todo!()
}

fn server_send_sync_data(world: &mut World) {
    if !world.contains_resource::<RenetServer>() {
        return;
    }

    world.resource_scope(|world, mut server: Mut<RenetServer>| {
        world.resource_scope(|world, mut queries: Mut<NetworkSyncQueries>| {
            world.resource_scope(|world, net_ids: Mut<NetIdMap>| {
                world.resource_scope(|world, type_registry: Mut<TypeRegistryArc>| {
                    let type_registry = type_registry.read();
                    impl_server_send_sync_data(
                        world,
                        &mut server,
                        &mut queries,
                        &net_ids,
                        &*type_registry,
                    );
                });
            });
        });
    });
}

fn impl_server_send_sync_data(
    world: &mut World,
    server: &mut RenetServer,
    queries: &mut NetworkSyncQueries,
    net_ids: &NetIdMap,
    type_registry: &TypeRegistryInternal,
) {
    let mut frame_sync_queries = Vec::with_capacity(queries.len());

    // Iterate over queries that need to be synced
    for net_query in queries.iter_mut() {
        let query = &mut net_query.query;
        let query_info = DynamicQueryInfo::from_query(query);

        // Get the ReflectFromPtr and ReflectSerialize for each component type in the query
        let reflect_tools = query
            .fetches()
            .iter()
            .map(|fetch_kind| fetch_kind.component_id())
            .map(|component_id| {
                world
                    .components()
                    .get_info(component_id)
                    .unwrap()
                    .type_id()
                    .unwrap()
            })
            .map(|type_id| {
                let type_registration = type_registry.get(type_id).expect("Type not registered");
                let reflect_from_ptr = type_registration
                    .data::<ReflectFromPtr>()
                    .expect("Missing ReflectFromPtr");
                let reflect_serialize = type_registration
                    .data::<ReflectSerialize>()
                    .expect("Missing ReflectFromPtr");
                (reflect_from_ptr, reflect_serialize)
            })
            .collect::<Vec<_>>();

        // Loop over all entities matching the query
        let mut serialized_items = Vec::new();
        for item in query.iter_mut(world) {
            let net_id = if let Some(id) = net_ids.get_net_id(item.entity) {
                id
            } else {
                warn!("Skipping sync of entity without known NetId");
                continue;
            };

            let mut components_bytes = Vec::with_capacity(item.items.len());
            for (fetch_result, (reflect_from_ptr, reflect_serialize)) in
                item.items.into_iter().zip(reflect_tools.iter())
            {
                let ptr = if let FetchResult::Ref(ptr) = fetch_result {
                    ptr
                } else {
                    panic!("Only read-only query supported for network sync");
                };

                let reflect = unsafe { reflect_from_ptr.as_reflect_ptr(ptr) };
                let serializable = reflect_serialize.get_serializable(reflect);
                let bytes =
                    postcard::to_allocvec(&serializable.borrow()).expect("Serialize component");
                components_bytes.push(bytes);
            }
            serialized_items.push(FrameSyncQueryItem {
                net_id,
                components: components_bytes,
            });
        }

        frame_sync_queries.push(FrameSyncQuery {
            query_info,
            items: serialized_items,
            prune: net_query.prune,
        });
    }

    let message =
        postcard::to_allocvec(&frame_sync_queries).expect("Serialize frame sync net message");
    server.broadcast_message(NetChannels::FrameSync, message);
}

trait FetchKindExt {
    fn component_id(&self) -> ComponentId;
}

impl FetchKindExt for FetchKind {
    fn component_id(&self) -> ComponentId {
        match self {
            FetchKind::Ref(id) | FetchKind::RefMut(id) => *id,
        }
    }
}
