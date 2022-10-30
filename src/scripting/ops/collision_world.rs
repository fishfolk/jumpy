use std::sync::Mutex;

use crate::{physics::collisions::CollisionWorld, prelude::*};
use anyhow::Context;
use bevy::ecs::system::SystemState;
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, JsValueRef, JsValueRefs, OpContext};
use once_cell::sync::OnceCell;

#[derive(Serialize)]
struct JsNetInfo {
    is_server: bool,
    is_client: bool,
}

pub struct CollisionWorldActorCollisions;
impl JsRuntimeOp for CollisionWorldActorCollisions {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.CollisionWorld) {
                globalThis.CollisionWorld = {}
            }
            
            globalThis.CollisionWorld.actorCollisions = (entity) => {
                return bevyModJsScriptingOpSync(
                    'jumpy_collision_world_actor_collisions',
                    Value.unwrapValueRef(entity)
                ).map(x => Value.wrapValueRef(x));
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
        static STATE: OnceCell<Mutex<SystemState<CollisionWorld>>> = OnceCell::new();
        let mut state = STATE
            .get_or_init(|| Mutex::new(SystemState::new(world)))
            .lock()
            .unwrap();

        let (value_ref,): (JsValueRef,) = serde_json::from_value(args).context("Parse args")?;
        let value_refs = ctx.op_state.get_mut::<JsValueRefs>().unwrap();
        let entity = value_ref.get_entity(world, value_refs)?;

        let collision_world = state.get_mut(world);
        let collisions = collision_world.actor_collisions(entity);
        let collisions = collisions
            .into_iter()
            .map(|x| JsValueRef::new_free(Box::new(x), value_refs))
            .collect::<Vec<_>>();

        Ok(serde_json::to_value(&collisions)?)
    }
}
