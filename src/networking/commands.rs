#![allow(dead_code)] // TODO: Remove this when we use more stuff

use std::marker::PhantomData;

use bevy::{
    ecs::{
        entity::Entities,
        system::{Command, EntityCommands, Resource, SystemParam},
    },
    reflect::TypeRegistryArc,
};
use serde::{de::DeserializeSeed, Deserialize, Serialize};

use crate::{
    config::ENGINE_CONFIG,
    networking::serialization::{
        de::CompactReflectDeserializer, deserialize_from_bytes, deserializer_from_bytes,
    },
    prelude::*,
    utils::ResetController,
};

use super::{
    serialization::{
        get_type_name_cache, ser::CompactReflectSerializer, serialize_to_bytes, StringCache,
    },
    NetId, NetIdMap,
};

pub struct NetCommandsPlugin;

impl Plugin for NetCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetIdMap>()
            .add_system(client_handle_net_commands.run_if(super::client_connected));

        if ENGINE_CONFIG.server_mode {
            let type_registry = app.world.resource::<TypeRegistryArc>();
            let string_cache = get_type_name_cache(type_registry);
            app.world.insert_resource(TypeNameCache(string_cache));
        } else {
            app.world.insert_resource(TypeNameCache::default());
        }
    }
}

#[derive(Default)]
pub struct TypeNameCache(pub StringCache);

#[derive(Serialize, Deserialize, Clone)]
pub enum CommandMessage {
    Spawn {
        net_id: NetId,
    },
    Insert {
        net_id: NetId,
        component_bytes: Vec<u8>,
        type_name: String,
    },
    InsertResource {
        resource_bytes: Vec<u8>,
        type_name: String,
    },
    Despawn {
        recursive: bool,
        net_id: NetId,
    },
    NextState(State),
    /// This corresponds to the action implemented by the [`jumpy::utils::ResetController`].
    ///
    /// Technically it's not a "Command", but it's similar enough that we put it in here.
    ResetWorld,
}

impl std::fmt::Debug for CommandMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn { net_id } => f.debug_struct("Spawn").field("net_id", net_id).finish(),
            Self::Insert {
                net_id,
                component_bytes: _,
                type_name,
            } => f
                .debug_struct("Insert")
                .field("net_id", net_id)
                .field("component_bytes", &"...")
                .field("type_name", type_name)
                .finish(),
            Self::InsertResource {
                resource_bytes: _,
                type_name,
            } => f
                .debug_struct("InsertResource")
                .field("resource_bytes", &"...")
                .field("type_name", type_name)
                .finish(),
            Self::Despawn { recursive, net_id } => f
                .debug_struct("Despawn")
                .field("recursive", recursive)
                .field("net_id", net_id)
                .finish(),
            Self::NextState(arg0) => f.debug_tuple("NextState").field(arg0).finish(),
            Self::ResetWorld => write!(f, "ResetWorld"),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum State {
    GameState(GameState),
    InGameState(InGameState),
}

impl From<GameState> for State {
    fn from(s: GameState) -> Self {
        Self::GameState(s)
    }
}
impl From<InGameState> for State {
    fn from(s: InGameState) -> Self {
        Self::InGameState(s)
    }
}

pub fn client_handle_net_commands(
    entities: &Entities,
    mut commands: Commands,
    type_registry: Res<TypeRegistryArc>,
    mut client: ResMut<RenetClient>,
    mut net_ids: ResMut<NetIdMap>,
    type_names: Res<TypeNameCache>,
    mut reset_controller: ResetController,
) {
    while let Some(message) = client.receive_message(NetChannels::Commands) {
        let message: CommandMessage =
            deserialize_from_bytes(&message).expect("Deserialize server message");
        trace!(command=?message, "Received CommandMessage from server");

        match message {
            CommandMessage::Spawn { net_id } => {
                let entity = commands.spawn().insert(net_id).id();
                net_ids.insert(entity, net_id);
            }
            CommandMessage::Insert {
                net_id,
                component_bytes,
                type_name,
            } => {
                let entity = net_ids.get_entity(net_id).expect("Entity not spawned");
                let type_registry = type_registry.read();
                let type_registration = type_registry
                    .get_with_name(&type_name)
                    .expect("Not registered in TypeRegistry");

                let reflect_component = type_registration
                    .data::<ReflectComponent>()
                    .expect("Doesn't have ReflectComponent")
                    .clone();
                let reflect_deserializer =
                    CompactReflectDeserializer::new(&type_registry, &type_names.0);
                let component_data = reflect_deserializer
                    .deserialize(&mut deserializer_from_bytes(&component_bytes))
                    .expect("Deserialize net component");

                commands.add(InsertReflectComponent {
                    entity,
                    component_data,
                    reflect_component,
                });
            }
            CommandMessage::InsertResource {
                resource_bytes,
                type_name,
            } => {
                let type_registry = type_registry.read();
                let type_registration = type_registry
                    .get_with_name(&type_name)
                    .expect("Not registered in TypeRegistry");
                let reflect_resource = type_registration
                    .data::<ReflectResource>()
                    .expect("Doesn't have ReflectResource")
                    .clone();
                let reflect_deserializer =
                    CompactReflectDeserializer::new(&type_registry, &type_names.0);
                let resource_data = reflect_deserializer
                    .deserialize(&mut deserializer_from_bytes(&resource_bytes))
                    .expect("Deserialize component");

                commands.add(InsertReflectResource {
                    reflect_resource,
                    resource_data,
                });
            }
            CommandMessage::NextState(s) => match s {
                State::GameState(s) => commands.insert_resource(NextState(s)),
                State::InGameState(s) => commands.insert_resource(NextState(s)),
            },
            CommandMessage::Despawn { recursive, net_id } => {
                if let Some(entity) = net_ids.remove_net_id(net_id) {
                    if entities.contains(entity) {
                        let mut cmds = commands.entity(entity);
                        if recursive {
                            cmds.despawn_recursive()
                        } else {
                            cmds.despawn()
                        }
                    }
                }
            }
            CommandMessage::ResetWorld => {
                reset_controller.reset_world();
            }
        }
    }
}

struct InsertReflectComponent {
    entity: Entity,
    reflect_component: ReflectComponent,
    component_data: Box<dyn Reflect>,
}
impl Command for InsertReflectComponent {
    fn write(self, world: &mut World) {
        self.reflect_component.apply_or_insert(
            world,
            self.entity,
            self.component_data.as_reflect(),
        );
    }
}

struct InsertReflectResource {
    reflect_resource: ReflectResource,
    resource_data: Box<dyn Reflect>,
}
impl Command for InsertReflectResource {
    fn write(self, world: &mut World) {
        self.reflect_resource
            .apply_or_insert(world, self.resource_data.as_reflect());
    }
}

/// Similar to [`Commands`], but with network synchronization of the executed commands.
#[derive(SystemParam)]
pub struct NetCommands<'w, 's> {
    commands: Commands<'w, 's>,
    res: NetResources<'w, 's>,
}

#[derive(SystemParam)]
pub struct NetResources<'w, 's> {
    type_registry: Res<'w, TypeRegistryArc>,
    net_ids: ResMut<'w, NetIdMap>,
    server: Option<ResMut<'w, RenetServer>>,
    type_names: Option<Res<'w, TypeNameCache>>,
    #[system_param(ignore)]
    _phantom: PhantomData<&'s ()>,
}

impl<'w, 's> NetCommands<'w, 's> {
    pub fn spawn<'a>(&'a mut self) -> NetEntityCommands<'w, 's, 'a> {
        let mut entity_cmds = self.commands.spawn();

        if let Some(server) = &mut self.res.server {
            // Allocate a network identifier for the new entity
            let net_id = NetId::new();
            // Add it to the queued network IDs
            self.res.net_ids.insert(entity_cmds.id(), net_id);

            // Inser the network ID as a component on the entity
            entity_cmds.insert(net_id);

            // Notify clients/server that an entity has been spawned
            let message = serialize_to_bytes(&CommandMessage::Spawn { net_id })
                .expect("Serialize network message");
            server.broadcast_message(NetChannels::Commands, message);
        }

        NetEntityCommands {
            entity_cmds,
            res: &mut self.res,
        }
    }

    pub fn entity<'a>(&'a mut self, entity: Entity) -> NetEntityCommands<'w, 's, 'a> {
        let entity_cmds = self.commands.entity(entity);
        NetEntityCommands {
            entity_cmds,
            res: &mut self.res,
        }
    }

    pub fn next_state(&mut self, state: impl Into<State>) -> &mut Self {
        let state = state.into();

        if let Some(server) = &mut self.res.server {
            let message = serialize_to_bytes(&CommandMessage::NextState(state))
                .expect("Serialize network message");
            server.broadcast_message(NetChannels::Commands, message);
        }

        match state {
            State::GameState(s) => self.commands.insert_resource(NextState(s)),
            State::InGameState(s) => self.commands.insert_resource(NextState(s)),
        }

        self
    }

    pub fn insert_resource(&mut self, resource: impl Resource + Reflect) -> &mut Self {
        if let Some(server) = &mut self.res.server {
            // Serialize the component
            let (resource_bytes, type_name) = {
                let type_registry = self.res.type_registry.read();
                let type_registration = type_registry
                    .get(resource.type_id())
                    .expect("Not registered in TypeRegistry");
                let serializer = CompactReflectSerializer::new(
                    resource.as_reflect(),
                    &type_registry,
                    &self.res.type_names.as_ref().unwrap().0,
                );

                let message = serialize_to_bytes(&serializer).expect("Serialize net message");

                (message, type_registration.type_name())
            };

            // Send the clients/server the inserted component
            let message = serialize_to_bytes(&CommandMessage::InsertResource {
                resource_bytes,
                type_name: type_name.into(),
            })
            .expect("Serialize net message");
            server.broadcast_message(NetChannels::Commands, message);
        }

        self.commands.insert_resource(resource);

        self
    }
}

pub struct NetEntityCommands<'w, 's, 'a> {
    entity_cmds: EntityCommands<'w, 's, 'a>,
    res: &'a mut NetResources<'w, 's>,
}

impl<'w, 's, 'a> NetEntityCommands<'w, 's, 'a> {
    pub fn id(&self) -> Entity {
        self.entity_cmds.id()
    }
    pub fn insert<C: Reflect + Component>(&mut self, c: C) -> &mut Self {
        if let Some(server) = &mut self.res.server {
            let entity = self.entity_cmds.id();

            // Get the network ID
            let net_id = self
                .res
                .net_ids
                .get_net_id(entity)
                .expect("Entity has no NetId");

            // Serialize the component
            let (component_bytes, type_name) = {
                let type_registry = self.res.type_registry.read();
                let type_registration = type_registry.get(c.type_id()).unwrap_or_else(|| {
                    panic!(
                        "{} not registered in TypeRegistry",
                        std::any::type_name::<C>()
                    )
                });
                let serializable = CompactReflectSerializer::new(
                    c.as_reflect(),
                    &type_registry,
                    &self.res.type_names.as_ref().unwrap().0,
                );

                let message = serialize_to_bytes(&serializable).expect("Serialize net message");

                (message, type_registration.type_name())
            };

            // Send the clients/server the inserted component
            let message = serialize_to_bytes(&CommandMessage::Insert {
                net_id,
                component_bytes,
                type_name: type_name.into(),
            })
            .expect("Serialize net message");
            server.broadcast_message(NetChannels::Commands, message);
        }

        self.entity_cmds.insert(c);

        self
    }

    pub fn despawn_recursive(self) {
        if let Some(server) = &mut self.res.server {
            if let Some(net_id) = self.res.net_ids.remove_entity(self.entity_cmds.id()) {
                let message = serialize_to_bytes(&CommandMessage::Despawn {
                    recursive: true,
                    net_id,
                })
                .expect("Serialize net message");
                server.broadcast_message(NetChannels::Commands, message);
            }
        }

        self.entity_cmds.despawn_recursive();
    }
}
