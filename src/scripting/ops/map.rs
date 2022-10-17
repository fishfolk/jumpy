use crate::{metadata::MapElementMeta, prelude::*};
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, JsValueRef, OpContext};

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct MapElementLoaded;

pub struct ElementGetSpawnedEntities;
impl JsRuntimeOp for ElementGetSpawnedEntities {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.MapElement) {
                globalThis.MapElement = {}
            }
            
            globalThis.MapElement.getSpawnedEntities = () => {
                return bevyModJsScriptingOpSync('jumpy_element_get_spawned_entities')
                    .map(x => globalThis.Value.wrapValueRef(x));
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
        let value_refs = ctx.op_state.get_mut().unwrap();

        let entities = world
            .query_filtered::<(Entity, &MapElementMeta), Without<MapElementLoaded>>()
            .iter(world)
            .filter(|(_, meta)| meta.script_handle.inner == ctx.script_info.handle)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>()
            .into_iter()
            .map(|entity| {
                world.entity_mut(entity).insert(MapElementLoaded);
                JsValueRef::new_free(Box::new(entity), value_refs)
            })
            .collect::<Vec<_>>();

        Ok(serde_json::to_value(&entities)?)
    }
}
