//! This is a workaround for the fact that we can't store entity refs across frames and we can't
//! create custom marker components. See for context:
//! <https://github.com/jakobhellermann/bevy_mod_js_scripting/issues/32>

use anyhow::Context;
use bevy::prelude::Entity;
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, JsValueRef, JsValueRefs};
use serde::{Deserialize, Serialize};

use crate::scripting::JsU64;

#[derive(Serialize, Deserialize)]
struct JsEntity(JsU64);

impl From<Entity> for JsEntity {
    fn from(e: Entity) -> Self {
        Self(e.to_bits().into())
    }
}
impl From<JsEntity> for Entity {
    fn from(e: JsEntity) -> Self {
        Entity::from_bits(e.0.into())
    }
}

pub struct EntityRefToJs;
impl JsRuntimeOp for EntityRefToJs {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
                if (!globalThis.EntityRef) {
                    globalThis.EntityRef = {}
                }

                globalThis.EntityRef.toJs = (valueRef) => {
                    return bevyModJsScriptingOpSync("entity_ref_to_js", Value.unwrapValueRef(valueRef));
                }
                "#,
        )
    }

    fn run(
        &self,
        ctx: bevy_mod_js_scripting::OpContext<'_>,
        world: &mut bevy::prelude::World,
        args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let (value_ref,): (JsValueRef,) = serde_json::from_value(args).context("Parse args")?;

        let value_refs = ctx.op_state.get_mut::<JsValueRefs>().unwrap();

        let entity = value_ref.get_entity(world, value_refs)?;

        Ok(serde_json::to_value(JsEntity::from(entity))?)
    }
}

pub struct EntityRefFromJs;
impl JsRuntimeOp for EntityRefFromJs {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
                if (!globalThis.EntityRef) {
                    globalThis.EntityRef = {}
                }

                globalThis.EntityRef.fromJs = (jsRef) => {
                    return Value.wrapValueRef(bevyModJsScriptingOpSync("entity_ref_from_js", jsRef));
                }
                "#,
        )
    }

    fn run(
        &self,
        ctx: bevy_mod_js_scripting::OpContext<'_>,
        _world: &mut bevy::prelude::World,
        args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let (js_ent,): (JsEntity,) = serde_json::from_value(args).context("Parse args")?;

        let value_refs = ctx.op_state.get_mut::<JsValueRefs>().unwrap();

        let entity = Entity::from(js_ent);
        let entity_ref = JsValueRef::new_free(Box::new(entity), value_refs);

        Ok(serde_json::to_value(entity_ref)?)
    }
}
