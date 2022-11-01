use std::sync::Mutex;

use crate::{
    item::{Item, ItemDropEvent, ItemGrabEvent},
    networking::{
        client::NetClient,
        proto::{self, ClientMatchInfo},
    },
    physics::KinematicBody,
    player::PlayerIdx,
    prelude::*,
};
use anyhow::Context;
use bevy::ecs::system::SystemState;
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, JsValueRef, JsValueRefs, OpContext};
use once_cell::sync::OnceCell;

pub struct PlayerKill;
impl JsRuntimeOp for PlayerKill {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.Player) {
                globalThis.Player = {}
            }
            
            globalThis.Player.kill = (entity) => {
                return bevyModJsScriptingOpSync('jumpy_player_kill', Value.unwrapValueRef(entity));
            }
            "#,
        )
    }

    fn run(
        &self,
        ctx: OpContext,
        world: &mut World,
        args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let (value_ref,): (JsValueRef,) = serde_json::from_value(args).context("Parse args")?;
        let value_refs = ctx.op_state.get_mut::<JsValueRefs>().unwrap();
        let entity = value_ref.get_entity(world, value_refs)?;

        // If the entity is a player
        if let Some(player_idx) = world.get::<PlayerIdx>(entity).cloned() {
            // If this is a network game
            if let Some(client) = world.get_resource::<NetClient>() {
                if let Some(match_info) = world.get_resource::<ClientMatchInfo>() {
                    if match_info.player_idx == player_idx.0 {
                        // Send a network message to kill the player
                        client.send_reliable(&proto::game::PlayerEvent::KillPlayer);
                        despawn_with_children_recursive(world, entity);
                    } else {
                        warn!("Tried to kill remote player in local game");
                    }
                } else {
                    warn!("Tried to kill remote player in local game");
                }

            // If this is a local game
            } else {
                despawn_with_children_recursive(world, entity);
            }
        }

        Ok(serde_json::Value::Null)
    }
}

pub struct PlayerGetInventory;
impl JsRuntimeOp for PlayerGetInventory {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.Player) {
                globalThis.Player = {}
            }
            
            globalThis.Player.getInventory = (entity) => {
                return Value.wrapValueRef(bevyModJsScriptingOpSync(
                    'jumpy_player_get_inventory',
                    Value.unwrapValueRef(entity)
                ));
            }
            "#,
        )
    }

    fn run(
        &self,
        ctx: OpContext,
        world: &mut World,
        args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let (value_ref,): (JsValueRef,) = serde_json::from_value(args).context("Parse args")?;
        let value_refs = ctx.op_state.get_mut::<JsValueRefs>().unwrap();
        let entity = value_ref.get_entity(world, value_refs)?;

        let item_ent = get_player_inventory(world, entity);
        let inventory = item_ent.map(|x| JsValueRef::new_free(Box::new(x), value_refs));

        Ok(serde_json::to_value(&inventory)?)
    }
}

fn get_player_inventory(world: &mut World, entity: Entity) -> Option<Entity> {
    let mut item_ent = None;
    let mut items_query = world.query_filtered::<Entity, With<Item>>();
    if let Some(children) = world.get::<Children>(entity) {
        for child in children {
            if items_query.get(world, *child).is_ok() {
                if item_ent.is_none() {
                    item_ent = Some(*child);
                } else {
                    warn!("Multiple items in player inventory is not supported!");
                }
            }
        }
    }
    item_ent
}

pub struct PlayerSetInventory;
impl JsRuntimeOp for PlayerSetInventory {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.Player) {
                globalThis.Player = {}
            }
            
            globalThis.Player.setInventory = (player, inventory) => {
                return bevyModJsScriptingOpSync(
                    'jumpy_player_set_inventory',
                    Value.unwrapValueRef(player),
                    Value.unwrapValueRef(inventory)
                );
            }
            "#,
        )
    }

    fn run(
        &self,
        ctx: OpContext,
        world: &mut World,
        args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let (player_ref, item_ref): (JsValueRef, Option<JsValueRef>) =
            serde_json::from_value(args).context("Parse args")?;
        let value_refs = ctx.op_state.get_mut::<JsValueRefs>().unwrap();
        let player_ent = player_ref.get_entity(world, value_refs)?;
        let item_ent = item_ref
            .map(|x| x.get_entity(world, value_refs))
            .transpose()?;

        let player_transform = *world
            .get::<Transform>(player_ent)
            .expect("Player missing transform");
        let player_velocity = world
            .get::<KinematicBody>(player_ent)
            .expect("Player missing kinematic body")
            .velocity;

        let current_inventory = get_player_inventory(world, player_ent);

        type Param<'w, 's> = (
            Commands<'w, 's>,
            EventWriter<'w, 's, ItemGrabEvent>,
            EventWriter<'w, 's, ItemDropEvent>,
        );
        static STATE: OnceCell<Mutex<SystemState<Param>>> = OnceCell::new();
        let mut state = STATE
            .get_or_init(|| Mutex::new(SystemState::new(world)))
            .lock()
            .unwrap();
        let (mut commands, mut grab_events, mut drop_events) = state.get_mut(world);

        let mut player = commands.entity(player_ent);
        if let Some(current_item) = current_inventory {
            drop_events.send(ItemDropEvent {
                player: player_ent,
                item: current_item,
                position: player_transform.translation,
                velocity: player_velocity,
            });
            player.remove_children(&[current_item]);
        }
        if let Some(item) = item_ent {
            grab_events.send(ItemGrabEvent {
                player: player_ent,
                item,
                position: player_transform.translation,
            });
            player.push_children(&[item]);
        }

        state.apply(world);

        Ok(serde_json::Value::Null)
    }
}
