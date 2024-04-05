//! Track scoring of rounds / tournament

use crate::prelude::*;

/// Timer tracking how long until round is scored once one or fewer players are alive
#[derive(HasSchema, Clone, Default)]
pub struct RoundScoringState {
    /// Timer used to count down to scoring, or to round transition post scoring.
    /// Is `None` if round is in progress.
    timer: Option<Timer>,

    /// If true: round has been scored, timer counts down to round transition.
    round_scored: bool,
}

/// Store player's match score's (rounds won)
#[derive(HasSchema, Clone, Default)]
pub struct MatchScore {
    /// Map player to score, if no entry is 0.
    player_score: HashMap<Entity, u32>,

    /// How many rounds have completed this match
    rounds_completed: u32,
}

impl MatchScore {
    /// Get player's score
    pub fn score(&self, entity: Entity) -> u32 {
        self.player_score.get(&entity).map_or(0, |s| *s)
    }

    /// Mark round as completed and increment score of winner. None should be provided
    /// on a draw.
    pub fn complete_round(&mut self, winner: Option<Entity>) {
        self.rounds_completed += 1;

        // Increment winner's score if not a draw
        if let Some(winner) = winner {
            if let Some(score) = self.player_score.get_mut(&winner) {
                *score += 1;
            } else {
                self.player_score.insert(winner, 1);
            }
        }
    }

    /// How many rounds have been played in this match
    pub fn rounds_completed(&self) -> u32 {
        self.rounds_completed
    }
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PostUpdate, round_end);

    session.world.insert_resource(MatchScore::default());
}

pub fn round_end(
    mut commands: Commands,
    meta: Root<GameMeta>,
    entities: Res<Entities>,
    rng: Res<GlobalRng>,
    mut map_pool: ResMut<MapPool>,
    mut score: ResMut<MatchScore>,
    mut sessions: ResMut<Sessions>,
    time: Res<Time>,
    mut state: ResMutInit<RoundScoringState>,
    killed_players: Comp<PlayerKilled>,
    player_indices: Comp<PlayerIdx>,
) {
    // Count players so we can avoid ending round if it's a one player match
    let mut player_count = 0;

    // Is Some if one player left, or none if all players dead.
    // Exits function if >= 2 players left: otherwise we handle continue to handle
    // round scoring.
    let last_player_or_draw: Option<Entity> = {
        let mut last_player: Option<Entity> = None;
        for (ent, (_player_idx, killed)) in
            entities.iter_with((&player_indices, &Optional(&killed_players)))
        {
            player_count += 1;
            if killed.is_none() {
                if last_player.is_some() {
                    // At least two players alive, not the round end.
                    return;
                }

                last_player = Some(ent);
            }
        }

        // We either found only one player or None.
        last_player
    };

    if player_count == 1 {
        // Single player match - don't end round.
        return;
    }

    // Tick any round end timer we have
    if let Some(timer) = state.timer.as_mut() {
        timer.tick(time.delta());
    }

    // There are one or fewer players alive if we have not already returned from function
    match state.timer {
        Some(ref t) if t.finished() && !state.round_scored => {
            // Score the round
            state.round_scored = true;
            score.complete_round(last_player_or_draw);

            if let Some(winner) = last_player_or_draw {
                // commands.add(PlayerCommand::won_round(winner));
                commands.add(spawn_win_indicator(winner));
            }

            // Start the post-score linger timer before next round
            state.timer = Some(Timer::new(
                meta.core.config.round_end_post_score_linger_time,
                TimerMode::Once,
            ));
        }
        Some(ref t) if t.finished() && state.round_scored => {
            // post-score linger timer complete, go to next round
            map_pool.randomize_current_map(&rng);
                sessions.add_command(Box::new(|sessions: &mut Sessions| {
                sessions.restart_game();
            }));
        }
        None => {
            // Scoring timer does not exist, start a new one
            state.timer = Some(Timer::new(
                meta.core.config.round_end_score_time,
                TimerMode::Once,
            ));
            state.round_scored = false;
        }
        _ => (), // Timer still ticking
    };
}
