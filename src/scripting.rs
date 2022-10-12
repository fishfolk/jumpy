use crate::prelude::*;
use bevy::{ecs::schedule::ShouldRun, time::FixedTimestep};
use bevy_mod_js_scripting::{run_script_fn_system, JsRuntimeConfig, JsScriptingPlugin};

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

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        let custom_ops = ops::get_ops();

        app.register_type::<Time>()
            .insert_non_send_resource(JsRuntimeConfig { custom_ops })
            .add_plugin(JsScriptingPlugin {
                skip_core_stage_setup: true,
            });

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
    match should_run.0 {
        no @ (ShouldRun::NoAndCheckAgain | ShouldRun::No) => no,
        yes @ (ShouldRun::Yes | ShouldRun::YesAndCheckAgain) => {
            let is_in_game = game_state
                .map(|x| x.0 == GameState::InGame)
                .unwrap_or(false)
                && in_game_state
                    .map(|x| x.0 != InGameState::Paused)
                    .unwrap_or(false);

            if is_in_game {
                yes
            } else if yes == ShouldRun::YesAndCheckAgain {
                ShouldRun::NoAndCheckAgain
            } else {
                ShouldRun::No
            }
        }
    }
}
