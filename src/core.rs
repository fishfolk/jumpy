pub mod attachment;
pub mod bullet;
pub mod camera;
pub mod damage;
pub mod debug;
pub mod editor;
pub mod elements;
pub mod globals;
pub mod input;
pub mod item;
pub mod lifetime;
pub mod map;
pub mod map_constructor;
pub mod map_pool;
pub mod metadata;
pub mod physics;
pub mod player;
pub mod random;
pub mod scoring;
pub mod utils;
pub mod win_indicator;

/// The target fixed frames-per-second that the game sumulation runs at.
pub const FPS: f32 = 60.0;

/// The maximum number of players per match.
pub const MAX_PLAYERS: u32 = 4;

use std::time::Duration;

use crate::prelude::*;

pub mod prelude {
    pub use super::{
        attachment::*, bullet::*, camera::*, damage::*, debug::*, editor::*, elements::prelude::*,
        flappy_jellyfish::*, globals::*, input::*, item::*, lifetime::*, map::*,
        map_constructor::*, map_pool::*, metadata::*, physics::*, player::*, random::*, scoring::*,
        utils::*, win_indicator::*, FPS, MAX_PLAYERS,
    };
}

pub fn game_plugin(game: &mut Game) {
    PlayerMeta::register_schema();
    AudioSource::register_schema();
    HatMeta::register_schema();
    MapMeta::register_schema();
    game.install_plugin(elements::game_plugin)
        .install_plugin(bullet::game_plugin)
        .install_plugin(win_indicator::game_plugin)
        .init_shared_resource::<AssetServer>();
}

pub struct MatchPlugin {
    pub maps: MapPool,
    pub player_info: [PlayerInput; MAX_PLAYERS as usize],
    /// The lua plugins to enable for this match.
    pub plugins: Arc<Vec<Handle<LuaPlugin>>>,

    /// Tracks score for match. Should be default if installing for
    /// new match, but if restarting MatchPlugin to transition between rounds,
    /// should be inputted from previous session resourc.
    pub score: MatchScore,

    pub session_runner: Box<dyn SessionRunner>,
}

pub struct MatchPlayerInfo {
    /// If control_source is `None` then the player is an AI.
    pub control_source: Option<ControlSource>,
    /// The player skin that has been selected.
    pub selected_player: Handle<PlayerMeta>,
}

impl SessionPlugin for MatchPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session
            .install_plugin(DefaultSessionPlugin)
            .install_plugin(LuaPluginLoaderSessionPlugin(self.plugins));

        physics::install(session);
        input::install(session);
        map::install(session);
        player::plugin(session);
        elements::session_plugin(session);
        damage::install(session);
        camera::install(session);
        lifetime::install(session);
        random::plugin(session);
        debug::plugin(session);
        item::install(session);
        attachment::install(session);
        bullet::session_plugin(session);
        editor::install(session);
        scoring::session_plugin(session);

        let current_map = self.maps.current_map;
        session.insert_resource(self.maps);

        // Initialize LoadedMap on startup as we cannot access AssetServer during MatchPlugin install
        // to get map meta.
        session.add_startup_system(
            move |mut loaded_map: ResMutInit<LoadedMap>, assets: Res<AssetServer>| {
                let map_meta = assets.get(current_map).clone();
                *loaded_map = LoadedMap(Arc::new(map_meta))
            },
        );

        session.insert_resource(MatchInputs {
            players: self.player_info,
        });
        session.insert_resource(self.score);
        session.set_session_runner(self.session_runner);
    }
}

#[derive(Default)]
pub struct JumpyDefaultMatchRunner {
    /// The jumpy match runner has it's own input collector instead of using the global one, because
    /// it needs different `just_[input]` behavior due to it running on a fixed update.
    pub input_collector: PlayerInputCollector,
    pub accumulator: f64,
    pub last_run: Option<Instant>,
    /// Disables local input for session.
    disable_local_input: bool,
}

impl SessionRunner for JumpyDefaultMatchRunner {
    fn step(&mut self, frame_start: Instant, world: &mut World, stages: &mut SystemStages) {
        pub const STEP: f64 = 1.0 / FPS as f64;
        let last_run = self.last_run.unwrap_or(frame_start);
        let delta = (frame_start - last_run).as_secs_f64();

        self.input_collector.apply_inputs(world);

        let mut run = || {
            // Advance the world time
            world
                .resource_mut::<Time>()
                .advance_exact(Duration::from_secs_f64(STEP));

            self.input_collector.update_just_pressed();
            let input = self.input_collector.get_current_controls();
            {
                let mut player_inputs = world.resource_mut::<MatchInputs>();
                (0..MAX_PLAYERS as usize).for_each(|i| {
                    let player_input = &mut player_inputs.players[i];
                    let Some(source) = &player_input.control_source else {
                        return;
                    };
                    if !self.disable_local_input {
                        player_input.control = *input.get(source).unwrap();
                    } else {
                        // This runner is only used for offline play, use default control for all to
                        // disable input by using default (no input).
                        player_input.control = PlayerControl::default();
                    }
                });
            }

            // Mark inputs as consumed for this frame
            self.input_collector.advance_frame();

            // Advance the simulation
            stages.run(world);
        };

        self.accumulator += delta;

        let loop_start = Instant::now();
        loop {
            if self.accumulator >= STEP {
                let loop_too_long = (Instant::now() - loop_start).as_secs_f64() > STEP;

                if loop_too_long {
                    warn!("Frame took too long: couldn't keep up with fixed update.");
                    self.accumulator = 0.0;
                    break;
                } else {
                    self.accumulator -= STEP;
                    run()
                }
            } else {
                break;
            }
        }

        self.last_run = Some(frame_start);
    }

    fn restart_session(&mut self) {
        *self = JumpyDefaultMatchRunner::default();
    }

    fn disable_local_input(&mut self, disable_input: bool) {
        self.disable_local_input = disable_input;
    }
}
