#![allow(dead_code)] // TODO: Remove this when we use more stuff

use std::marker::PhantomData;

use bevy::ecs::{
    entity::Entities,
    system::{Command, Resource},
};

use crate::utils::ResetController;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    /// Technically it's not a command, but it's similar enough we put it in here.
    ResetWorld,
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
    mut reset_controller: ResetController,
) {
    while let Some(message) = client.receive_message(NetChannels::Commands) {
        let message: CommandMessage =
            postcard::from_bytes(&message).expect("Deserialize server message");

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
                let reflect_deserialize = type_registration
                    .data::<ReflectDeserialize>()
                    .expect("Doesn't have ReflectDeserialize");
                let reflect_component = type_registration
                    .data::<ReflectComponent>()
                    .expect("Doesn't have ReflectComponent")
                    .clone();
                let component_data = reflect_deserialize
                    .deserialize(&mut postcard::Deserializer::from_bytes(&component_bytes))
                    .expect("Deserialize component");

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
                let reflect_deserialize = type_registration
                    .data::<ReflectDeserialize>()
                    .expect("Doesn't have ReflectDeserialize");
                let reflect_resource = type_registration
                    .data::<ReflectResource>()
                    .expect("Doesn't have ReflectComponent")
                    .clone();
                let resource_data = reflect_deserialize
                    .deserialize(&mut postcard::Deserializer::from_bytes(&resource_bytes))
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

#[derive(Clone, Debug, Default)]
pub struct NetIdMap {
    ent_to_net: HashMap<Entity, NetId>,
    net_to_ent: HashMap<NetId, Entity>,
}

impl NetIdMap {
    fn insert(&mut self, entity: Entity, net_id: NetId) {
        self.ent_to_net.insert(entity, net_id);
        self.net_to_ent.insert(net_id, entity);
    }

    fn remove_entity(&mut self, entity: Entity) -> Option<NetId> {
        if let Some(net_id) = self.ent_to_net.remove(&entity) {
            self.net_to_ent.remove(&net_id);

            Some(net_id)
        } else {
            None
        }
    }

    fn remove_net_id(&mut self, net_id: NetId) -> Option<Entity> {
        if let Some(entity) = self.net_to_ent.remove(&net_id) {
            self.ent_to_net.remove(&entity);

            Some(entity)
        } else {
            None
        }
    }

    fn get_entity(&self, net_id: NetId) -> Option<Entity> {
        self.net_to_ent.get(&net_id).cloned()
    }

    fn get_net_id(&self, entity: Entity) -> Option<NetId> {
        self.ent_to_net.get(&entity).cloned()
    }
}

#[derive(SystemParam)]
pub struct NetResources<'w, 's> {
    type_registry: Res<'w, TypeRegistryArc>,
    net_ids: ResMut<'w, NetIdMap>,
    server: Option<ResMut<'w, RenetServer>>,
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
            let message = postcard::to_allocvec(&CommandMessage::Spawn { net_id })
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
            let message = postcard::to_allocvec(&CommandMessage::NextState(state))
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
                let reflect_serialize = type_registration
                    .data::<ReflectSerialize>()
                    .expect("Doesn't have ReflectSerialize");
                let serializable = reflect_serialize.get_serializable(&resource);

                (
                    postcard::to_allocvec(serializable.borrow()).expect("Serialize component"),
                    type_registration.type_name(),
                )
            };

            // Send the clients/server the inserted component
            let message = postcard::to_allocvec(&CommandMessage::InsertResource {
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
                let reflect_serialize = type_registration
                    .data::<ReflectSerialize>()
                    .expect("Doesn't have ReflectSerialize");
                let serializable = reflect_serialize.get_serializable(&c);

                (
                    postcard::to_allocvec(serializable.borrow()).expect("Serialize component"),
                    type_registration.type_name(),
                )
            };

            // Send the clients/server the inserted component
            let message = postcard::to_allocvec(&CommandMessage::Insert {
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
            if let Some(net_id) = self.res.net_ids.get_net_id(self.entity_cmds.id()) {
                let message = postcard::to_allocvec(&CommandMessage::Despawn {
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
