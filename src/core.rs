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
pub mod metadata;
pub mod physics;
pub mod player;
pub mod random;
pub mod utils;

/// The target fixed frames-per-second that the game sumulation runs at.
pub const FPS: f32 = 60.0;

/// The maximum number of players per match.
pub const MAX_PLAYERS: usize = 4;

use std::{array, time::Duration};

use crate::prelude::*;

pub mod prelude {
    pub use super::{
        attachment::*, bullet::*, camera::*, damage::*, debug::*, editor::*, editor::*,
        elements::prelude::*, elements::prelude::*, globals::*, input::*, item::*, lifetime::*,
        map::*, map_constructor::*, metadata::*, physics::*, player::*, random::*, utils::*, FPS,
        MAX_PLAYERS,
    };
}

pub fn game_plugin(game: &mut Game) {
    game.init_shared_resource::<AssetServer>()
        .register_asset::<GameMeta>()
        .register_asset::<PlayerMeta>()
        .register_asset::<AudioSource>()
        .register_asset::<HatMeta>()
        .register_asset::<MapMeta>()
        .register_asset::<ElementMeta>()
        .register_asset::<FishSchoolMeta>()
        .register_asset::<KickBombMeta>()
        .register_asset::<AnimatedDecorationMeta>()
        .register_asset::<PlayerSpawner>()
        .register_asset::<SwordMeta>();
}

pub struct MatchPlugin {
    pub map: MapMeta,
    pub selected_players: [Option<Handle<PlayerMeta>>; MAX_PLAYERS],
}

impl SessionPlugin for MatchPlugin {
    fn install(self, session: &mut Session) {
        session.install_plugin(DefaultSessionPlugin);

        physics::install(session);
        input::install(session);
        map::install(session);
        player::plugin(session);
        elements::install(session);
        damage::install(session);
        camera::install(session);
        lifetime::install(session);
        random::plugin(session);
        debug::plugin(session);
        item::install(session);
        attachment::install(session);
        bullet::install(session);
        editor::install(session);

        session.world.insert_resource(LoadedMap(Arc::new(self.map)));
        session.world.insert_resource(PlayerInputs {
            players: array::from_fn(|i| {
                self.selected_players[i]
                    .map(|selected_player| PlayerInput {
                        active: true,
                        selected_player,
                        ..default()
                    })
                    .unwrap_or_default()
            }),
        });
        session.runner = Box::<JumpyDefaultMatchRunner>::default();
    }
}

#[derive(Default)]
pub struct PlayerInputCollector {
    last_controls: [PlayerControl; MAX_PLAYERS],
    current_controls: [PlayerControl; MAX_PLAYERS],
}

impl PlayerInputCollector {
    /// Update the internal state with new inputs. This must be called every render frame with the
    /// input events.
    pub fn update(&mut self, keyboard: &KeyboardInputs, _gamepad: &GamepadInputs) {
        let p1 = &mut self.current_controls[0];

        for event in &keyboard.key_events {
            if event.key_code == Some(KeyCode::Space) {
                p1.jump_pressed = event.button_state.pressed();
            }
        }
    }

    /// Get the player inputs for the next game simulation frame.
    pub fn get(&mut self) -> &[PlayerControl; MAX_PLAYERS] {
        (0..MAX_PLAYERS).for_each(|i| {
            let current = &mut self.current_controls[i];
            let last = &self.last_controls[i];

            current.jump_just_pressed = current.jump_pressed && !last.jump_pressed
        });

        self.last_controls = self.current_controls.clone();

        &self.current_controls
    }
}

#[derive(Default)]
pub struct JumpyDefaultMatchRunner {
    pub input_collector: PlayerInputCollector,
    pub accumulator: f64,
    pub last_run: Option<Instant>,
}

impl SessionRunner for JumpyDefaultMatchRunner {
    fn step(&mut self, frame_start: Instant, world: &mut World, stages: &mut SystemStages) {
        pub const STEP: f64 = 1.0 / FPS as f64;
        let last_run = self.last_run.unwrap_or(frame_start);
        let delta = (frame_start - last_run).as_secs_f64();

        {
            let keyboard = world.resource::<KeyboardInputs>();
            let gamepad = world.resource::<GamepadInputs>();
            self.input_collector.update(&keyboard, &gamepad);
        }

        let mut run = || {
            // Advance the world time
            world
                .resource_mut::<Time>()
                .advance_exact(Duration::from_secs_f64(STEP));

            let input = self.input_collector.get();
            {
                let mut player_inputs = world.resource_mut::<PlayerInputs>();
                (0..MAX_PLAYERS).for_each(|i| {
                    player_inputs.players[i].control = input[i].clone();
                });
            }

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
}
