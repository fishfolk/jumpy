use std::time::Duration;

use bevy::ecs::schedule::ShouldRun;
use bevy_mod_js_scripting::run_script_fn_system;

use crate::prelude::*;

pub struct PlayerStatePlugin;

#[derive(StageLabel)]
pub enum PlayerStateStage {
    CollectExternalTransitions,
    PerformTransitions,
    HandleState,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct PlayerState(u64);

impl Plugin for PlayerStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerState>()
            .add_stage_after(
                CoreStage::PreUpdate,
                PlayerStateStage::CollectExternalTransitions,
                SystemStage::parallel().with_system(
                    run_script_fn_system("playerStateCollectTransitions".into()).at_end(),
                ),
            )
            .add_stage_after(
                PlayerStateStage::CollectExternalTransitions,
                PlayerStateStage::PerformTransitions,
                // Note: We use the iyes_loopless FixedTimestepStage here, instead of the FixedTimestep
                // run critera that we use elsewhere, because it is much easier to compose it with our
                // state_transition_run_critera.
                //
                // The reason we don't _always_ use `FixedTimestepStage` is because it doesn't work with
                // the `app.add_system_to_stage()` method.
                FixedTimestepStage::from_stage(
                    Duration::from_secs_f64(crate::FIXED_TIMESTEP),
                    SystemStage::single_threaded()
                        .with_run_criteria(state_transition_run_criteria)
                        .with_system(run_script_fn_system("playerStateTransition".into()).at_end()),
                ),
            )
            .add_stage_after(
                PlayerStateStage::PerformTransitions,
                PlayerStateStage::HandleState,
                SystemStage::parallel().with_system(
                    run_script_fn_system("handlePlayerState".into()).at_end(),
                ),
            );
    }
}

fn state_transition_run_criteria(
    mut run_once: Local<bool>,
    changed_states: Query<Entity, Changed<PlayerState>>,
) -> ShouldRun {
    if !*run_once {
        *run_once = true;
        ShouldRun::YesAndCheckAgain
    } else if changed_states.iter().count() > 0 {
        ShouldRun::YesAndCheckAgain
    } else {
        *run_once = false;
        ShouldRun::No
    }
}
