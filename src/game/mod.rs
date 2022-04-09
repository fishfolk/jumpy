mod camera;

pub use camera::GameCamera;

use fishsticks::{Button, GamepadContext};

#[cfg(feature = "macroquad-backend")]
const GAME_MENU_OPTION_MAIN_MENU: usize = 10;
#[cfg(feature = "macroquad-backend")]
const GAME_MENU_OPTION_QUIT: usize = 20;

#[cfg(feature = "macroquad-backend")]
use ff_core::gui::{Menu, MenuEntry};

use hecs::{Entity, World};

use ff_core::prelude::*;
use ff_core::Result;

use crate::items::try_get_item;
use crate::physics::{debug_draw_physics_bodies, fixed_update_physics_bodies};
use crate::player::{
    draw_weapons_hud, spawn_player, update_player_animations, update_player_camera_box,
    update_player_controllers, update_player_events, update_player_inventory,
    update_player_passive_effects, update_player_states, PlayerParams,
};
use crate::{
    create_collision_world, debug_draw_drawables, debug_draw_rigid_bodies, draw_drawables,
    fixed_update_rigid_bodies, update_animated_sprites, Map, MapLayerKind, MapObjectKind,
};
use crate::{debug, gui, PlayerControllerKind};

use crate::effects::active::debug_draw_active_effects;
use crate::effects::active::projectiles::fixed_update_projectiles;
use crate::effects::active::triggered::fixed_update_triggered_effects;
use crate::gui::MainMenuState;
use crate::items::spawn_item;
use crate::network::{
    fixed_update_network_client, fixed_update_network_host, update_network_client,
    update_network_host,
};
use crate::sproinger::{fixed_update_sproingers, spawn_sproinger};
use ff_core::macroquad::time::get_frame_time;
use ff_core::macroquad::ui::root_ui;
use ff_core::map::spawn_decoration;
use ff_core::particles::{draw_particles, update_particle_emitters};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GameMode {
    Local,
    NetworkHost,
    NetworkClient,
}

impl From<&str> for GameMode {
    fn from(str: &str) -> Self {
        match str {
            LOCAL_GAME_STATE_ID => Self::Local,
            NETWORK_GAME_CLIENT_STATE_ID => Self::NetworkClient,
            NETWORK_GAME_HOST_STATE_ID => Self::NetworkHost,
            _ => panic!("The game state id '{}' does not match any game modes!", str),
        }
    }
}

impl From<String> for GameMode {
    fn from(str: String) -> Self {
        str.as_str().into()
    }
}

impl From<GameMode> for &str {
    fn from(mode: GameMode) -> Self {
        match mode {
            GameMode::Local => LOCAL_GAME_STATE_ID,
            GameMode::NetworkClient => NETWORK_GAME_CLIENT_STATE_ID,
            GameMode::NetworkHost => NETWORK_GAME_HOST_STATE_ID,
        }
    }
}

pub const LOCAL_GAME_STATE_ID: &str = "local_game";
pub const NETWORK_GAME_CLIENT_STATE_ID: &str = "network_game_client";
pub const NETWORK_GAME_HOST_STATE_ID: &str = "network_game_host";

#[derive(Clone)]
pub struct StatePayload {
    map: Map,
    players: Vec<PlayerParams>,
}

const GAME_MENU_ID: &str = "game_menu";

pub fn build_state_for_game_mode(
    game_mode: GameMode,
    map: Map,
    players: &[PlayerParams],
) -> DefaultGameState<StatePayload> {
    let mut state_builder = GameStateBuilder::new(game_mode.into()).with_payload(StatePayload {
        map,
        players: players.to_vec(),
    });

    #[cfg(feature = "macroquad-backend")]
    let mut menu = Menu::new(
        GAME_MENU_ID,
        250.0,
        &[
            MenuEntry {
                index: GAME_MENU_OPTION_MAIN_MENU,
                title: "Main Menu".to_string(),
                action: || {
                    let state = MainMenuState::new();
                    dispatch_event(GameEvent::StateTransition(Box::new(state)));
                },
                ..Default::default()
            },
            MenuEntry {
                index: GAME_MENU_OPTION_QUIT,
                title: "Quit".to_string(),
                action: || dispatch_event(GameEvent::Quit),
                ..Default::default()
            },
        ],
    );

    #[cfg(feature = "macroquad-backend")]
    state_builder.add_menu(menu);

    if game_mode == GameMode::NetworkClient {
        state_builder.add_update(update_network_client);
        state_builder.add_fixed_update(fixed_update_network_client);
    } else if game_mode == GameMode::NetworkHost {
        state_builder.add_update(update_network_host);
        state_builder.add_fixed_update(fixed_update_network_host);
    }

    state_builder
        .add_update(update_player_controllers)
        .add_update(update_player_camera_box)
        .add_update(update_player_animations)
        .add_update(update_animated_sprites)
        .add_update(update_particle_emitters);

    if matches!(game_mode, GameMode::Local | GameMode::NetworkHost) {
        state_builder
            .add_update(update_player_events)
            .add_update(update_player_states)
            .add_update(update_player_inventory)
            .add_update(update_player_passive_effects);

        state_builder
            .add_fixed_update(fixed_update_physics_bodies)
            .add_fixed_update(fixed_update_rigid_bodies)
            .add_fixed_update(fixed_update_projectiles)
            .add_fixed_update(fixed_update_triggered_effects)
            .add_fixed_update(fixed_update_sproingers);
    }

    state_builder
        .add_draw(draw_drawables)
        .add_draw(draw_weapons_hud)
        .add_draw(draw_particles);

    #[cfg(debug_assertions)]
    state_builder
        .add_draw(debug_draw_drawables)
        .add_draw(debug_draw_physics_bodies)
        .add_draw(debug_draw_rigid_bodies)
        .add_draw(debug_draw_active_effects);

    state_builder
        .with_constructor(|world, payload| {
            let payload = payload.unwrap();

            init_game_world(world, payload.map.clone(), &payload.players)?;

            play_sound("fish_tide", true);

            Ok(())
        })
        .build()
}

pub fn init_game_world(world: &mut World, map: Map, players: &[PlayerParams]) -> Result<()> {
    let camera = GameCamera::new(map.get_size());
    storage::store(camera);

    let collision_world = create_collision_world(&map);
    storage::store(collision_world);

    spawn_map_objects(world, &map)?;

    for params in players {
        let position = map.get_random_spawn_point();

        spawn_player(
            world,
            params.index,
            position,
            params.controller.clone(),
            params.character.clone(),
        );
    }

    storage::store(map);

    Ok(())
}

cfg_if! {
    if #[cfg(not(feature = "ultimate"))] {
        use ff_core::macroquad::prelude::scene::{self, Node, RefMut};

        pub struct Game {
            state: Box<dyn GameState>,
        }

        impl Game {
            pub fn new(world: Option<World>, initial_state: Box<dyn GameState>) -> Result<Self> {
                let mut state = initial_state;
                state.begin(world)?;

                Ok(Game {
                    state,
                })
            }

            pub fn set_state(&mut self, state: Box<dyn GameState>) -> Result<()> {
                let world = self.state.end()?;
                self.state = state;
                self.state.begin(world)?;

                Ok(())
            }
        }

        impl Node for Game {
            fn ready(mut node: RefMut<Self>) where Self: Sized {
                node.state.begin(None).unwrap();
            }

            fn update(mut node: RefMut<Self>) where Self: Sized {
                node.state.update(get_frame_time());

                if let Some(mut camera) = storage::try_get_mut::<GameCamera>() {
                    camera.update();
                }
            }

            fn fixed_update(mut node: RefMut<Self>) where Self: Sized {
                node.state.fixed_update(get_frame_time(), 1.0);
            }

            fn draw(mut node: RefMut<Self>) where Self: Sized {
                if let Some(camera) = storage::try_get::<GameCamera>() {
                    let camera_position = camera.bounds.point() + (camera.bounds.size() / 2.0);

                    let map = storage::get::<Map>();
                    map.draw(None, camera_position);
                }

                node.state.draw();
            }
        }
    }
}

pub fn spawn_map_objects(world: &mut World, map: &Map) -> Result<Vec<Entity>> {
    let mut objects = Vec::new();

    for layer in map.layers.values() {
        if layer.is_visible && layer.kind == MapLayerKind::ObjectLayer {
            for map_object in &layer.objects {
                match map_object.kind {
                    MapObjectKind::Decoration => {
                        let res = try_get_decoration(&map_object.id);

                        if let Some(params) = res.cloned() {
                            let decoration = spawn_decoration(world, map_object.position, params);
                            objects.push(decoration);
                        } else {
                            #[cfg(debug_assertions)]
                            println!("WARNING: Invalid decoration id '{}'", &map_object.id)
                        }
                    }
                    MapObjectKind::Item => {
                        let res = try_get_item(&map_object.id);

                        if let Some(params) = res.cloned() {
                            let item = spawn_item(world, map_object.position, params)?;
                            objects.push(item);
                        } else {
                            #[cfg(debug_assertions)]
                            println!("WARNING: Invalid item id '{}'", &map_object.id)
                        }
                    }
                    MapObjectKind::Environment => {
                        if map_object.id == "sproinger" {
                            let sproinger = spawn_sproinger(world, map_object.position)?;
                            objects.push(sproinger);
                        } else {
                            #[cfg(debug_assertions)]
                            println!("WARNING: Invalid environment item id '{}'", &map_object.id)
                        }
                    }
                }
            }
        }
    }

    Ok(objects)
}
