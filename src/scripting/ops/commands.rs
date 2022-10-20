use anyhow::{format_err, Context};
use bevy::ecs::system::SystemState;
use bevy::prelude::{default, ReflectComponent};
use bevy::reflect::TypeRegistryArc;

use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, JsValueRef, JsValueRefs, OpContext};
use bevy_renet::renet::RenetServer;

use crate::networking::commands::{CommandMessage, TypeNameCache};
use crate::networking::serialization::ser::CompactReflectSerializer;
use crate::networking::serialization::serialize_to_bytes;
use crate::networking::{NetChannels, NetIdMap};
use crate::prelude::NetCommands;

pub struct NetCommandsSpawn;
impl JsRuntimeOp for NetCommandsSpawn {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.NetCommands) {
                globalThis.NetCommands = {}
            }
            
            globalThis.NetCommands.spawn = () => {
                return Value.wrapValueRef(
                    bevyModJsScriptingOpSync("jumpy_net_commands_spawn")
                );
            }
            "#,
        )
    }

    fn run(
        &self,
        context: OpContext<'_>,
        world: &mut bevy::prelude::World,
        args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        // Expect empty arguments
        let _: [(); 0] = serde_json::from_value(args).context("parse args")?;

        let value_refs = context
            .op_state
            .entry::<JsValueRefs>()
            .or_insert_with(default);

        let mut net_commands_state = SystemState::<NetCommands>::new(world);
        let entity = {
            let mut net_commands = net_commands_state.get_mut(world);
            net_commands.spawn().id()
        };

        net_commands_state.apply(world);

        let value_ref = JsValueRef::new_free(Box::new(entity), value_refs);

        Ok(serde_json::to_value(value_ref)?)
    }
}

pub struct NetCommandsInsert;
impl JsRuntimeOp for NetCommandsInsert {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.NetCommands) {
                globalThis.NetCommands = {}
            }
            
            globalThis.NetCommands.insert = (entity, component) => {
                bevyModJsScriptingOpSync(
                    "jumpy_net_commands_insert",
                    Value.unwrapValueRef(entity),
                    Value.unwrapValueRef(component)
                );
            }
            "#,
        )
    }

    fn run(
        &self,
        context: OpContext<'_>,
        world: &mut bevy::prelude::World,
        args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        // Parse args
        let (entity_value_ref, component_value_ref): (JsValueRef, JsValueRef) =
            serde_json::from_value(args).context("parse args")?;

        let mut server = world.remove_resource::<RenetServer>();

        let value_refs = context
            .op_state
            .entry::<JsValueRefs>()
            .or_insert_with(default);

        // Get entity and make sure the entity exists
        let entity = entity_value_ref.get_entity(world, value_refs)?;
        world
            .get_entity(entity)
            .ok_or_else(|| format_err!("Entity does not exist"))?;

        let component_value_ref = value_refs
            .get(component_value_ref.key)
            .ok_or_else(|| format_err!("Value ref doesn't exist"))?
            .clone();

        // Load the type registry
        let type_registry = world.resource::<TypeRegistryArc>();
        let type_names = world.resource::<TypeNameCache>();
        let net_ids = world.resource::<NetIdMap>();
        let type_registry = type_registry.read();

        // Clone the reflect value of the component
        let reflect_value_ref = component_value_ref.get(world)?;
        let type_id = reflect_value_ref.type_id();
        let reflect_value = reflect_value_ref.clone_value();

        // Get the ReflectComponent
        let type_registration = type_registry.get(type_id).expect("Type not registered");
        let reflect_component = type_registration
            .data::<ReflectComponent>()
            .ok_or_else(|| format_err!("ReflectComponent not found for component value ref"))?
            .clone();

        if let Some(server) = &mut server {
            let net_id = net_ids.get_net_id(entity).expect("Entity has no NetId");
            let serializable = CompactReflectSerializer::new(
                reflect_value_ref.as_reflect(),
                &type_registry,
                &type_names.0,
            );
            let component_bytes = serialize_to_bytes(&serializable).expect("Serialize component");

            // Send component insertion message over the network
            let type_name = type_registration.type_name();
            let message = serialize_to_bytes(&CommandMessage::Insert {
                net_id,
                component_bytes,
                type_name: type_name.into(),
            })
            .expect("Serialize net message");
            server.broadcast_message(NetChannels::Commands, message);
        }

        // Drop our immutable borrow of the world
        drop(type_registry);
        drop(reflect_value_ref);

        reflect_component.apply_or_insert(world, entity, reflect_value.as_reflect());

        if let Some(server) = server {
            world.insert_resource(server);
        }

        Ok(serde_json::Value::Null)
    }
}
