use std::{
    any::TypeId,
    hash::{Hash, Hasher},
};

use crate::prelude::*;
use bevy::{asset::HandleId, reflect::TypeRegistryArc};
use bevy_mod_js_scripting::{
    bevy_reflect_fns::{
        PassMode, ReflectArg, ReflectFunction, ReflectFunctionError, ReflectMethods,
    },
    JsRuntimeConfig, JsScriptingPlugin,
};

pub mod ops;

pub struct ScriptingPlugin;

#[derive(StageLabel)]
pub enum ScriptUpdateStage {
    First,
    FirstInGame,
    PreUpdate,
    PreUpdateInGame,
    Update,
    UpdateInGame,
    PostUpdate,
    PostUpdateInGame,
    Last,
    LastInGame,
}

/// A JS that represents a u64 with two u32's that can be serialized to JavaScript `number` types
/// without losing precision ( I think. )
#[derive(Serialize, Deserialize)]
pub struct JsU64(u32, u32);

impl From<u64> for JsU64 {
    fn from(n: u64) -> Self {
        JsU64((n >> 32) as u32, n as u32)
    }
}

impl From<JsU64> for u64 {
    fn from(JsU64(n2, n1): JsU64) -> Self {
        (n2 as u64) << 32 | n1 as u64
    }
}

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

// struct ScriptingRollbackHooks;

// impl RollbackEventHook for ScriptingRollbackHooks {
//     fn pre_save(&mut self, frame: Frame, max_snapshots: usize, world: &mut World) {
//         let runtime = world.remove_non_send_resource::<JsRuntime>().unwrap();

//         // We use extremely brief keys here to avoid encoding more string across the FFI
//         let args = serde_json::json!({
//             "f": frame,
//             "m": max_snapshots,
//         });
//         if let Err(e) = runtime.eval(&format!("globalThis.saveSnapshot({})", args), world) {
//             error!("Error running JS save snapshot hook: {e:?}");
//         }

//         world.insert_non_send_resource(runtime);
//     }

//     fn post_load(
//         &mut self,
//         frame: Frame,
//         max_snapshots: usize,
//         entity_map: &EntityMap,
//         world: &mut World,
//     ) {
//         let runtime = world.remove_non_send_resource::<JsRuntime>().unwrap();

//         let mut entity_map_json = Vec::new();

//         for from in entity_map.keys() {
//             let to = entity_map.get(from).unwrap();
//             if from != to {
//                 entity_map_json.push(serde_json::json!({
//                     "f": JsEntity::from(from),
//                     "t": JsEntity::from(to),
//                 }));
//             }
//         }

//         // We use extremely brief keys here to avoid encoding more string across the FFI
//         let args = serde_json::json!({
//             "f": frame,
//             "m": max_snapshots,
//             "e": entity_map_json,
//         });
//         if let Err(e) = runtime.eval(&format!("globalThis.loadSnapshot({})", args), world) {
//             error!("Error running JS save snapshot hook: {e:?}");
//         }

//         world.insert_non_send_resource(runtime);
//     }
// }

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        let custom_ops = ops::get_ops();

        app.register_type::<Time>()
            .insert_non_send_resource(JsRuntimeConfig { custom_ops })
            .add_plugin(JsScriptingPlugin {
                skip_core_stage_setup: true,
            });
        // .extend_rollback_plugin(|plugin| plugin.add_rollback_hook(ScriptingRollbackHooks));

        {
            let type_registry = app.world.resource::<TypeRegistryArc>();
            let mut type_registry = type_registry.write();
            type_registry
                .get_mut(TypeId::of::<HandleId>())
                .unwrap()
                .insert(ReflectMethods::from_methods([(
                    "hash",
                    ReflectFunction {
                        fn_name: "hash",
                        signature: [(PassMode::Owned, TypeId::of::<HandleId>())]
                            .into_iter()
                            .collect(),
                        f: hash_handle_id_reflect,
                    },
                )]));
        }

        // TODO: For now scripting is disabled for performance reasons.

        // Add fixed update stages
        // app.extend_rollback_schedule(|schedule| {
        // schedule
        //     .add_stage_after(
        //         RollbackStage::First,
        //         ScriptUpdateStage::First,
        //         SystemStage::single(run_script_fn_system("first".into())),
        //     )
        //     .add_stage_after(
        //         RollbackStage::First,
        //         ScriptUpdateStage::FirstInGame,
        //         SystemStage::single(run_script_fn_system("firstInGame".into()))
        //             .with_run_criteria(is_in_game_run_criteria),
        //     )
        //     .add_stage_after(
        //         RollbackStage::PreUpdate,
        //         ScriptUpdateStage::PreUpdate,
        //         SystemStage::single(run_script_fn_system("preUpdate".into())),
        //     )
        //     .add_stage_after(
        //         RollbackStage::PreUpdate,
        //         ScriptUpdateStage::PreUpdateInGame,
        //         SystemStage::single(run_script_fn_system("preUpdateInGame".into()))
        //             .with_run_criteria(is_in_game_run_criteria),
        //     )
        //     .add_stage_after(
        //         RollbackStage::Update,
        //         ScriptUpdateStage::Update,
        //         SystemStage::single(run_script_fn_system("update".into())),
        //     )
        //     .add_stage_after(
        //         RollbackStage::Update,
        //         ScriptUpdateStage::UpdateInGame,
        //         SystemStage::single(run_script_fn_system("updateInGame".into()))
        //             .with_run_criteria(is_in_game_run_criteria),
        //     )
        //     .add_stage_after(
        //         RollbackStage::PostUpdate,
        //         ScriptUpdateStage::PostUpdate,
        //         SystemStage::single(run_script_fn_system("postUpdate".into())),
        //     )
        //     .add_stage_after(
        //         RollbackStage::PostUpdate,
        //         ScriptUpdateStage::PostUpdateInGame,
        //         SystemStage::single(run_script_fn_system("postUpdateInGame".into()))
        //             .with_run_criteria(is_in_game_run_criteria),
        //     )
        //     .add_stage_after(
        //         RollbackStage::Last,
        //         ScriptUpdateStage::Last,
        //         SystemStage::single(run_script_fn_system("last".into())),
        //     )
        //     .add_stage_after(
        //         RollbackStage::Last,
        //         ScriptUpdateStage::LastInGame,
        //         SystemStage::single(run_script_fn_system("lastInGame".into()))
        //             .with_run_criteria(is_in_game_run_criteria),
        //     );
        // });
    }
}

/// Helper function to hash a [`HandleId`].
fn hash_handle_id(id: HandleId) -> String {
    let mut hasher = fnv::FnvHasher::default();
    id.hash(&mut hasher);
    // The bit shift makes the hash fit within the safe integer range for a JavaScript number
    base64::encode(hasher.finish().to_le_bytes())
}

/// Wrapper around [`hash_handle_id`] for exposing as a reflect function.
fn hash_handle_id_reflect(
    args: &mut [ReflectArg],
) -> Result<Box<dyn Reflect>, ReflectFunctionError> {
    let arg_count = args.len();
    if arg_count != 1 {
        return Err(ReflectFunctionError::ArgCountMismatch {
            expected: 1,
            got: arg_count,
        });
    }

    let id = &args[0];
    let id: HandleId = id.from_reflect()?;
    let hash = hash_handle_id(id);
    Ok(Box::new(hash) as _)
}
