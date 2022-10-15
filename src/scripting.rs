use std::{
    any::TypeId,
    hash::{Hash, Hasher},
};

use crate::{prelude::*, run_criteria::ShouldRunExt};
use bevy::{
    asset::HandleId, ecs::schedule::ShouldRun, reflect::TypeRegistryArc, time::FixedTimestep,
};
use bevy_mod_js_scripting::{
    bevy_reflect_fns::{
        PassMode, ReflectArg, ReflectFunction, ReflectFunctionError, ReflectMethods,
    },
    run_script_fn_system, JsRuntimeConfig, JsScriptingPlugin,
};

mod ops;

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

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        let custom_ops = ops::get_ops();

        app.register_type::<Time>()
            .insert_non_send_resource(JsRuntimeConfig { custom_ops })
            .add_plugin(JsScriptingPlugin {
                skip_core_stage_setup: true,
            });

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
                        signature: [(PassMode::Owned, TypeId::of::<u64>())]
                            .into_iter()
                            .collect(),
                        f: hash_handle_id_reflect,
                    },
                )]));
        }

        // Add fixed update stages
        app.add_stage_after(
            FixedUpdateStage::First,
            ScriptUpdateStage::First,
            SystemStage::single(run_script_fn_system("first".into()))
                .with_run_criteria(FixedTimestep::step(crate::FIXED_TIMESTEP)),
        )
        .add_stage_after(
            FixedUpdateStage::First,
            ScriptUpdateStage::FirstInGame,
            SystemStage::single(run_script_fn_system("firstInGame".into())).with_run_criteria(
                FixedTimestep::step(crate::FIXED_TIMESTEP).chain(is_in_game_run_criteria),
            ),
        )
        .add_stage_after(
            FixedUpdateStage::PreUpdate,
            ScriptUpdateStage::PreUpdate,
            SystemStage::single(run_script_fn_system("preUpdate".into()))
                .with_run_criteria(FixedTimestep::step(crate::FIXED_TIMESTEP)),
        )
        .add_stage_after(
            FixedUpdateStage::PreUpdate,
            ScriptUpdateStage::PreUpdateInGame,
            SystemStage::single(run_script_fn_system("preUpdateInGame".into())).with_run_criteria(
                FixedTimestep::step(crate::FIXED_TIMESTEP).chain(is_in_game_run_criteria),
            ),
        )
        .add_stage_after(
            FixedUpdateStage::Update,
            ScriptUpdateStage::Update,
            SystemStage::single(run_script_fn_system("update".into()))
                .with_run_criteria(FixedTimestep::step(crate::FIXED_TIMESTEP)),
        )
        .add_stage_after(
            FixedUpdateStage::Update,
            ScriptUpdateStage::UpdateInGame,
            SystemStage::single(run_script_fn_system("updateInGame".into())).with_run_criteria(
                FixedTimestep::step(crate::FIXED_TIMESTEP).chain(is_in_game_run_criteria),
            ),
        )
        .add_stage_after(
            FixedUpdateStage::PostUpdate,
            ScriptUpdateStage::PostUpdate,
            SystemStage::single(run_script_fn_system("postUpdate".into()))
                .with_run_criteria(FixedTimestep::step(crate::FIXED_TIMESTEP)),
        )
        .add_stage_after(
            FixedUpdateStage::PostUpdate,
            ScriptUpdateStage::PostUpdateInGame,
            SystemStage::single(run_script_fn_system("postUpdateInGame".into())).with_run_criteria(
                FixedTimestep::step(crate::FIXED_TIMESTEP).chain(is_in_game_run_criteria),
            ),
        )
        .add_stage_after(
            FixedUpdateStage::Last,
            ScriptUpdateStage::Last,
            SystemStage::single(run_script_fn_system("last".into()))
                .with_run_criteria(FixedTimestep::step(crate::FIXED_TIMESTEP)),
        )
        .add_stage_after(
            FixedUpdateStage::Last,
            ScriptUpdateStage::LastInGame,
            SystemStage::single(run_script_fn_system("lastInGame".into())).with_run_criteria(
                FixedTimestep::step(crate::FIXED_TIMESTEP).chain(is_in_game_run_criteria),
            ),
        );
    }
}

/// Heper stage run criteria that only runs if we are in a gameplay state.
fn is_in_game_run_criteria(
    should_run: In<ShouldRun>,
    game_state: Option<Res<CurrentState<GameState>>>,
    in_game_state: Option<Res<CurrentState<InGameState>>>,
) -> ShouldRun {
    if should_run.0.should_run() {
        let is_in_game = game_state
            .map(|x| x.0 == GameState::InGame)
            .unwrap_or(false)
            && in_game_state
                .map(|x| x.0 != InGameState::Paused)
                .unwrap_or(false);

        ShouldRun::new(is_in_game, should_run.0.check_again())
    } else {
        should_run.0
    }
}

/// Helper function to hash a [`HandlId`].
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
