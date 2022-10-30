use crate::{
    networking::{
        client::NetClient,
        proto::{self, ClientMatchInfo},
    },
    player::PlayerIdx,
    prelude::*,
};
use anyhow::Context;
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, JsValueRef, JsValueRefs, OpContext};

#[derive(Serialize)]
struct JsNetInfo {
    is_server: bool,
    is_client: bool,
}

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
