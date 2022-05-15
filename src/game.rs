pub use ff_core::camera::Camera;

#[cfg(feature = "macroquad-backend")]
const GAME_MENU_OPTION_MAIN_MENU: usize = 10;
#[cfg(feature = "macroquad-backend")]
const GAME_MENU_OPTION_QUIT: usize = 20;

#[cfg(feature = "macroquad-backend")]
use ff_core::gui::{Menu, MenuEntry};

use ff_core::ecs::{Entity, World};

use ff_core::prelude::*;

use crate::items::try_get_item;
use crate::player::{
    draw_weapons_hud, spawn_player, update_player_animations, update_player_controllers,
    update_player_events, update_player_inventory, update_player_passive_effects,
    update_player_states, PlayerParams,
};
use crate::{Map, MapLayerKind, MapObjectKind};

use crate::effects::active::debug_draw_active_effects;
use crate::effects::active::projectiles::fixed_update_projectiles;
use crate::effects::active::triggered::fixed_update_triggered_effects;
use crate::items::spawn_item;
use crate::network::{
    fixed_update_network_client, fixed_update_network_host, update_network_client,
    update_network_host,
};
use crate::sproinger::{fixed_update_sproingers, spawn_sproinger};
use ff_core::map::{spawn_decoration, try_get_decoration};

use crate::camera::{update_camera, CameraController};

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
    players: Vec<PlayerParams>,
}

#[allow(dead_code)]
const GAME_MENU_ID: &str = "game_menu";

pub fn build_state_for_game_mode(
    game_mode: GameMode,
    map: Map,
    players: &[PlayerParams],
) -> Result<DefaultGameState<StatePayload>> {
    let mut builder = DefaultGameStateBuilder::new(game_mode.into())
        .with_default_systems()
        .with_map(map)
        .with_empty_world()
        .with_payload(StatePayload {
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
                    dispatch_event(Event::state_transition(state));
                },
                ..Default::default()
            },
            MenuEntry {
                index: GAME_MENU_OPTION_QUIT,
                title: "Quit".to_string(),
                action: || dispatch_event(Event::Quit),
                ..Default::default()
            },
        ],
    );

    #[cfg(feature = "macroquad-backend")]
    state_builder.add_menu(menu);

    if game_mode == GameMode::NetworkClient {
        builder.add_update(update_network_client);
        builder.add_fixed_update(fixed_update_network_client);
    } else if game_mode == GameMode::NetworkHost {
        builder.add_update(update_network_host);
        builder.add_fixed_update(fixed_update_network_host);
    }

    builder
        .add_update(update_player_controllers)
        .add_update(update_player_animations)
        .add_update(update_camera);

    if matches!(game_mode, GameMode::Local | GameMode::NetworkHost) {
        builder
            .add_update(update_player_events)
            .add_update(update_player_states)
            .add_update(update_player_inventory)
            .add_update(update_player_passive_effects);

        builder
            .add_fixed_update(fixed_update_projectiles)
            .add_fixed_update(fixed_update_triggered_effects)
            .add_fixed_update(fixed_update_sproingers);
    }

    builder.add_draw(draw_weapons_hud);

    #[cfg(debug_assertions)]
    builder.add_draw(debug_draw_active_effects);

    let res = builder
        .with_constructor(|world, map, payload| -> Result<()> {
            let payload = payload.unwrap();

            let res = init_game_world(world.unwrap(), map.unwrap().clone(), &payload.players);
            if let Err(err) = res {
                #[cfg(debug_assertions)]
                println!("ERROR: init_game_world: {}", err);
            }

            play_sound("fish_tide", true);

            Ok(())
        })
        .build();

    Ok(res)
}

pub fn init_game_world(world: &mut World, map: Map, players: &[PlayerParams]) -> Result<()> {
    let physics_world = physics_world();

    physics_world.clear();

    physics_world.add_map(&map);

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

    world.spawn((Transform::new(Vec2::ZERO, 0.0), CameraController::new()));

    Ok(())
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
