//! Extensions to the script environment `world` global.
//!
//! TODO: These ops should be migrated to the `bevy_mod_js_scripting` crate.

use anyhow::Context;
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, JsValueRef, JsValueRefs, OpContext};

use crate::prelude::*;

pub struct WorldDespawnRecursive;
impl JsRuntimeOp for WorldDespawnRecursive {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.WorldTemp) {
                globalThis.WorldTemp = {}
            }
            
            globalThis.WorldTemp.despawnRecursive = (entity) => {
                return bevyModJsScriptingOpSync(
                    'jumpy_world_despawn_recursive',
                    Value.unwrapValueRef(entity)
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
        let (value_ref,): (JsValueRef,) = serde_json::from_value(args).context("Parse args")?;
        let value_refs = ctx.op_state.get_mut::<JsValueRefs>().unwrap();
        let entity = value_ref.get_entity(world, value_refs)?;

        despawn_with_children_recursive(world, entity);

        Ok(serde_json::Value::Null)
    }
}
