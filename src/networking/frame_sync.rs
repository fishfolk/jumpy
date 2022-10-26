use bevy::{
    ecs::component::ComponentId,
    reflect::{ReflectFromPtr, TypeRegistryArc, TypeRegistryInternal},
};
use bevy_ecs_dynamic::dynamic_query::{DynamicQuery, FetchKind, FetchResult};
use serde::de::DeserializeSeed;

use crate::prelude::*;

use super::{
    client::NetClient,
    serialization::{
        de::CompactReflectDeserializer, deserializer_from_bytes, ser::CompactReflectSerializer,
        serialize_to_bytes, TypeNameCache,
    },
    server::NetServer,
    NetId, NetIdMap,
};

mod dynamic_query_info;
use self::dynamic_query_info::{DynamicQueryInfo, DynamicQueryInfoResources};

pub struct NetFrameSyncPlugin;

impl Plugin for NetFrameSyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkSyncConfig>()
            .add_system_to_stage(
                FixedUpdateStage::First,
                client_get_sync_data.exclusive_system().at_start(),
            )
            .add_system_to_stage(
                FixedUpdateStage::First,
                server_send_sync_data.exclusive_system().at_end(),
            );
    }
}

#[derive(Default)]
pub struct NetworkSyncConfig {
    pub queries: Vec<NetworkSyncQuery>,
}

pub struct NetworkSyncQuery {
    pub query: DynamicQuery,
    pub prune: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FrameSyncMessage {
    pub queries: Vec<FrameSyncQuery>,
}

type Bytes = Vec<u8>;
type ComponentsBytes = Vec<Bytes>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FrameSyncQuery {
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
    if !world.contains_resource::<NetClient>() {
        return;
    }
    if !world.contains_resource::<TypeNameCache>() {
        return;
    }

    world.resource_scope(|world, mut client: Mut<NetClient>| {
        world.resource_scope(|world, net_ids: Mut<NetIdMap>| {
            world.resource_scope(|world, type_registry: Mut<TypeRegistryArc>| {
                world.resource_scope(|world, type_names: Mut<TypeNameCache>| {
                    let type_registry = type_registry.read();
                    impl_client_get_sync_data(
                        world,
                        &mut client,
                        &net_ids,
                        &*type_registry,
                        &type_names,
                    )
                });
            });
        });
    });
}

fn impl_client_get_sync_data(
    world: &mut World,
    client: &mut NetClient,
    net_ids: &NetIdMap,
    type_registry: &TypeRegistryInternal,
    type_names: &TypeNameCache,
) {
    let mut message = None;
    // We only care about the last message, so loop until we stop getting messages and store the
    // latest one.
    while let Some(m) = client.recv_unreliable::<FrameSyncMessage>() {
        message = Some(m);
    }

    // If we have a frame update
    if let Some(message) = message {
        for net_query in message.queries {
            let mut query = net_query
                .query_info
                .with_all_fetches_mutable()
                .get_query(DynamicQueryInfoResources {
                    world,
                    type_names,
                    type_registry,
                })
                .expect("Invalid query");
            let reflect_tools = get_reflect_from_ptr(world, type_registry, &query);

            // Collect entities matching the query
            let entities = query.iter_mut(world).map(|x| x.entity).collect::<Vec<_>>();

            // Collect entities that have a net ID
            let mut net_entities = Vec::with_capacity(entities.len());
            for entity in entities {
                if let Some(net_id) = net_ids.get_net_id(entity) {
                    net_entities.push((entity, net_id));

                // Delete non-networked entities matching the query
                } else if net_query.prune {
                    despawn_with_children_recursive(world, entity);
                }
            }

            // Sort entities by net ID
            net_entities.sort_unstable_by_key(|x| x.1);

            let mut net_items_iter = net_query.items.into_iter();
            'entities: for (entity, net_id) in net_entities {
                // Get the components bytes for the matching networked entity
                let components_bytes = loop {
                    if let Some(item) = net_items_iter.next() {
                        if item.net_id == net_id {
                            break item.components;
                        }
                    } else {
                        break 'entities;
                    }
                };

                // Collect the pointers for the components of the entity
                let components_target_pointers = query
                    .get_mut(world, entity)
                    .expect("Query entity")
                    .into_iter()
                    .map(|x| {
                        if let FetchResult::RefMut { value, .. } = x {
                            value
                        } else {
                            unreachable!("All fetches should have been mutable")
                        }
                    });

                // Loop through the components and update them from the data from the network
                for ((target_pointer, component_bytes), reflect_from_ptr) in
                    components_target_pointers
                        .zip(components_bytes)
                        .zip(&reflect_tools)
                {
                    let target_reflect =
                        unsafe { reflect_from_ptr.as_reflect_ptr_mut(target_pointer) };

                    let deserializer =
                        CompactReflectDeserializer::new(type_registry, &type_names.0);

                    let component_data = deserializer
                        .deserialize(&mut deserializer_from_bytes(&component_bytes))
                        .expect("Deserialize net component");

                    target_reflect.apply(component_data.as_ref());
                }
            }
        }
    }
}

fn server_send_sync_data(world: &mut World) {
    if !world.contains_resource::<NetServer>() {
        return;
    }
    if !world.contains_resource::<TypeNameCache>() {
        return;
    }

    world.resource_scope(|world, mut server: Mut<NetServer>| {
        world.resource_scope(|world, mut queries: Mut<NetworkSyncConfig>| {
            world.resource_scope(|world, net_ids: Mut<NetIdMap>| {
                world.resource_scope(|world, type_registry: Mut<TypeRegistryArc>| {
                    world.resource_scope(|world, type_names: Mut<TypeNameCache>| {
                        let type_registry = type_registry.read();
                        impl_server_send_sync_data(
                            world,
                            &mut server,
                            &mut queries,
                            &net_ids,
                            &*type_registry,
                            &type_names,
                        );
                    });
                });
            });
        });
    });
}

fn impl_server_send_sync_data(
    world: &mut World,
    server: &mut NetServer,
    network_sync: &mut NetworkSyncConfig,
    net_ids: &NetIdMap,
    type_registry: &TypeRegistryInternal,
    type_names: &TypeNameCache,
) {
    let mut frame_sync_queries = Vec::with_capacity(network_sync.queries.len());

    // Iterate over queries that need to be synced
    for net_query in network_sync.queries.iter_mut() {
        let query = &mut net_query.query;
        let query_info = DynamicQueryInfo::from_query(
            query,
            DynamicQueryInfoResources {
                world,
                type_names,
                type_registry,
            },
        );

        // Get the ReflectFromPtr and ReflectSerialize for each component type in the query
        let reflect_tools = get_reflect_from_ptr(world, type_registry, query);

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
            for (fetch_result, reflect_from_ptr) in item.items.into_iter().zip(reflect_tools.iter())
            {
                let ptr = if let FetchResult::Ref(ptr) = fetch_result {
                    ptr
                } else {
                    panic!("Only read-only query supported for network sync");
                };

                let reflect = unsafe { reflect_from_ptr.as_reflect_ptr(ptr) };
                let serializable =
                    CompactReflectSerializer::new(reflect, type_registry, &type_names.0);
                let bytes = serialize_to_bytes(&serializable).expect("Serialize component");
                components_bytes.push(bytes);
            }
            serialized_items.push(FrameSyncQueryItem {
                net_id,
                components: components_bytes,
            });
        }

        // Sort items by net_id
        serialized_items.sort_unstable_by_key(|x| x.net_id);

        frame_sync_queries.push(FrameSyncQuery {
            query_info,
            items: serialized_items,
            prune: net_query.prune,
        });
    }

    let message = FrameSyncMessage {
        queries: frame_sync_queries,
    };

    server.broadcast_unreliable(&message);
}

fn get_reflect_from_ptr<'a, 'b>(
    world: &'b World,
    type_registry: &'a TypeRegistryInternal,
    query: &'b DynamicQuery,
) -> Vec<&'a ReflectFromPtr> {
    query
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
            reflect_from_ptr
        })
        .collect::<Vec<_>>()
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
