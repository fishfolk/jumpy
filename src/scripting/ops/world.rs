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

pub struct WorldSpawn;
impl JsRuntimeOp for WorldSpawn {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.WorldTemp) {
                globalThis.WorldTemp = {}
            }
            
            globalThis.WorldTemp.spawn = () => {
                return Value.wrapValueRef(bevyModJsScriptingOpSync(
                    'jumpy_world_spawn',
                ));
            }
            "#,
        )
    }

    fn run(
        &self,
        ctx: OpContext,
        world: &mut World,
        _args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let value_refs = ctx.op_state.get_mut::<JsValueRefs>().unwrap();

        let id = world.resource_scope(|world, mut rids: Mut<RollbackIdProvider>| {
            world.spawn().insert(Rollback::new(rids.next_id())).id()
        });

        let entity_ref = JsValueRef::new_free(Box::new(id), value_refs);

        Ok(serde_json::to_value(&entity_ref)?)
    }
}
