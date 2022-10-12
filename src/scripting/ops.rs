use bevy_mod_js_scripting::OpMap;

mod asset;
mod map;
mod script_info;

pub fn get_ops() -> OpMap {
    let mut ops = OpMap::default();

    ops.insert(
        "jumpy_element_get_spawned_entities",
        Box::new(map::ElementGetSpawnedEntities),
    );
    ops.insert(
        "jumpy_asset_get_handle_id",
        Box::new(asset::AssetGetHandleId),
    );
    ops.insert(
        "jumpy_script_info_get",
        Box::new(script_info::ScriptInfoGet),
    );
    ops.insert("entity_ref_to_js", Box::new(entity::EntityRefToJs));
    ops.insert("entity_ref_from_js", Box::new(entity::EntityRefFromJs));

    ops
}

/// This is a workaround for the fact that we can't store entity refs across frames and we can't
/// create custom marker components. See for context:
/// <https://github.com/jakobhellermann/bevy_mod_js_scripting/issues/32>
mod entity {
    use anyhow::Context;
    use bevy::prelude::Entity;
    use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, JsValueRef, JsValueRefs};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct JsEntity {
        bits_1: u32,
        bits_2: u32,
    }

    impl From<Entity> for JsEntity {
        fn from(e: Entity) -> Self {
            let bits = e.to_bits();

            Self {
                bits_1: bits as u32,
                bits_2: (bits >> 32) as u32,
            }
        }
    }
    impl From<JsEntity> for Entity {
        fn from(e: JsEntity) -> Self {
            Entity::from_bits((e.bits_2 as u64) << 32 | e.bits_1 as u64)
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
}
