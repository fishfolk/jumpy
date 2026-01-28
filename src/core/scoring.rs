//! Track scoring of rounds / tournament

use crate::{prelude::*, ui::scoring::ScoringMenuState};

/// Timer tracking how long until round is scored once one or fewer players are alive
#[derive(HasSchema, Clone, Default)]
pub struct RoundScoringState {
    /// Timer used to count down to scoring, or to round transition post scoring.
    /// Is `None` if round is in progress.
    pub timer: Option<Timer>,

    /// If true: round has been scored, timer counts down to round transition.
    pub round_scored: bool,

    /// Save MapPool state to transitin with when determining round end.
    pub next_maps: Option<MapPool>,

    /// Save the frame round was marked to transition on in network play.
    /// Transition does not execute until this is confirmed by remote players.
    pub network_round_end_frame: Option<i32>,
}

impl RoundScoringState {
    /// Are timers for round scoring + linger before transition complete?
    pub fn transition_timers_done(&self) -> bool {
        if let Some(timer) = self.timer.as_ref() {
            return timer.finished() && self.round_scored;
        }
        false
    }

    /// Is scoring timer complete but round not yet scored?
    pub fn should_score_round(&self) -> bool {
        if let Some(timer) = self.timer.as_ref() {
            return timer.finished() && !self.round_scored;
        }
        false
    }
}

/// Store player's match score's (rounds won)
#[derive(HasSchema, Clone, Default, Debug)]
pub struct MatchScore {
    /// Map player to score, if no entry is 0.
    player_score: HashMap<PlayerIdx, u32>,

    /// How many rounds have completed this match
    rounds_completed: u32,
}

impl MatchScore {
    /// Get player's score
    pub fn score(&self, player: PlayerIdx) -> u32 {
        self.player_score.get(&player).map_or(0, |s| *s)
    }

    /// Mark round as completed and increment score of winner. None should be provided
    /// on a draw.
    pub fn complete_round(&mut self, winner: Option<PlayerIdx>) {
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

pub fn session_plugin(session: &mut SessionBuilder) {
    session.add_system_to_stage(CoreStage::PostUpdate, round_end);
}

pub fn round_end(
    mut commands: Commands,
    meta: Root<GameMeta>,
    entities: Res<Entities>,
    rng: Res<GlobalRng>,
    map_pool: Res<MapPool>,
    mut score: ResMutInit<MatchScore>,
    mut sessions: ResMut<Sessions>,
    mut session_options: ResMut<SessionOptions>,
    time: Res<Time>,
    mut state: ResMutInit<RoundScoringState>,
    mut scoring_menu: ResMut<ScoringMenuState>,
    killed_players: Comp<PlayerKilled>,
    player_indices: Comp<PlayerIdx>,
    #[cfg(not(target_arch = "wasm32"))] syncing_info: Option<Res<SyncingInfo>>,
) {
    // Count players so we can avoid ending round if it's a one player match
    let mut player_count = 0;

    // Is Some if one player left, or none if all players dead.
    // Exits function if >= 2 players left: otherwise we handle continue to handle
    // round scoring.
    let last_player_or_draw: Option<(PlayerIdx, Entity)> = {
        let mut last_player: Option<(PlayerIdx, Entity)> = None;
        for (ent, (player_idx, killed)) in
            entities.iter_with((&player_indices, &Optional(&killed_players)))
        {
            player_count += 1;
            if killed.is_none() {
                if last_player.is_some() {
                    // At least two players alive, not the round end.
                    return;
                }

                last_player = Some((*player_idx, ent));
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

    // Ready to score the round?
    if state.should_score_round() {
        state.round_scored = true;
        score.complete_round(last_player_or_draw.map(|x| x.0));

        if let Some((_, winner_ent)) = last_player_or_draw {
            // commands.add(PlayerCommand::won_round(winner));
            commands.add(spawn_win_indicator(winner_ent));
        }

        // Start the post-score linger timer before next round
        state.timer = Some(Timer::new(
            meta.core.config.round_end_post_score_linger_time,
            TimerMode::Once,
        ));
    } else if state.transition_timers_done() {
        // post-score linger timer complete, go to next round if all players confirmed transition

        // Is round transition sycnrhonized on all clients in network play?
        // Will evaluate to true in local play.
        #[allow(unused_assignments)]
        let mut round_transition_synchronized = false;

        // If in network play and determined a prev frame round should end on:
        #[allow(unused_variables)]
        if let Some(end_net_frame) = state.network_round_end_frame {
            // check if this frame is confirmed by all players.
            #[cfg(not(target_arch = "wasm32"))]
            {
                round_transition_synchronized = match syncing_info {
                    Some(syncing_info) => end_net_frame <= syncing_info.last_confirmed_frame(),
                    None => true,
                };
            }
        } else {
            // Network frame for round end not yet recorded (or in local only)

            // Randomize map and save MapPool to be used for transition
            let mut map_pool = map_pool.clone();
            map_pool.randomize_current_map(&rng);
            state.next_maps = Some(map_pool);

            // Save current predicted frame for round end.
            // Will not follow through with transition until this frame is confirmed
            // by all players in network play.
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(syncing_info) = syncing_info {
                state.network_round_end_frame = Some(syncing_info.current_frame());
            } else {
                // No sync info - must be local and sychronized.
                round_transition_synchronized = true;
            }
        }

        // Wasm32 is always local, can transition now.
        #[cfg(target_arch = "wasm32")]
        {
            round_transition_synchronized = true;
        }

        if round_transition_synchronized {
            if score
                .rounds_completed
                .is_multiple_of(meta.core.config.rounds_between_intermission)
            {
                scoring_menu.active = true;
                scoring_menu.match_score = score.clone();
                scoring_menu.next_maps = state.next_maps.clone();

                session_options.active = false;
            } else {
                // Not at intermission, tranisition immediately

                // Use maps originally determined on synchronized transition frame
                let next_maps = state.next_maps.clone().unwrap();
                sessions.add_command(Box::new(|sessions: &mut Sessions| {
                    sessions.restart_game(Some(next_maps), false);
                }));
            }
        }
    } else if state.timer.is_none() {
        // Scoring timer does not exist, start a new one

        state.timer = Some(Timer::new(
            meta.core.config.round_end_score_time,
            TimerMode::Once,
        ));
        state.round_scored = false;
    } else {
        // Timer still ticking
    }
}
