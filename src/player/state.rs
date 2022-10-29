use std::time::Duration;

use bevy::{ecs::schedule::ShouldRun, time::FixedTimestep};
use bevy_mod_js_scripting::run_script_fn_system;

use crate::prelude::*;

pub struct PlayerStatePlugin;

#[derive(StageLabel)]
pub enum PlayerStateStage {
    // This stage hasn't been used yet and needs more evaulation to see if it is helpful.
    // CollectExternalTransitions,
    PerformTransitions,
    HandleState,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct PlayerState {
    /// The unique identifier for the current state
    id: String,
    /// The number of frames that this state has been active
    age: u64,
    /// The ID of the state that the player was in in the last frame
    last_state: String,
}

impl Plugin for PlayerStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerState>()
            // .add_stage_after(
            //     CoreStage::PreUpdate,
            //     PlayerStateStage::CollectExternalTransitions,
            //     SystemStage::parallel().with_system(
            //         run_script_fn_system("playerStateCollectTransitions".into()).at_end(),
            //     ),
            // )
            .add_stage_after(
                CoreStage::PreUpdate,
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
                        .with_system(
                            run_script_fn_system("playerStateTransition".into())
                                .with_run_criteria(in_game_not_paused)
                                .at_end(),
                        ),
                ),
            )
            .add_stage_after(
                PlayerStateStage::PerformTransitions,
                PlayerStateStage::HandleState,
                SystemStage::parallel()
                    .with_run_criteria(FixedTimestep::step(crate::FIXED_TIMESTEP))
                    .with_system(
                        run_script_fn_system("handlePlayerState".into())
                            .with_run_criteria(in_game_not_paused)
                            .at_end(),
                    ),
            )
            .add_system_to_stage(FixedUpdateStage::Last, update_player_state_age);
    }
}

/// Bevy run criteria for when the game is not paused
fn in_game_not_paused(
    game_state: Res<CurrentState<GameState>>,
    in_game_state: Res<CurrentState<InGameState>>,
) -> ShouldRun {
    if game_state.0 == GameState::InGame && in_game_state.0 != InGameState::Paused {
        return ShouldRun::Yes;
    }

    ShouldRun::No
}

fn state_transition_run_criteria(
    mut changed_states: Query<&mut PlayerState, Changed<PlayerState>>,
) -> ShouldRun {
    // Note, this will always run once per frame, because the `update_player_state_age` system runs
    // at the end of every frame.
    let mut has_changed = false;
    for mut state in &mut changed_states {
        has_changed = true;
        if state.last_state != state.id {
            state.age = 0;
        }
    }
    if has_changed {
        ShouldRun::YesAndCheckAgain
    } else {
        ShouldRun::No
    }
}

fn update_player_state_age(mut states: Query<&mut PlayerState>) {
    for mut state in &mut states {
        state.age = state.age.saturating_add(1);
        state.last_state = state.id.clone();
    }
}
