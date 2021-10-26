use fishsticks::GamepadContext;
use std::env;

use macroquad::prelude::*;
use macroquad::{
    audio,
    experimental::{
        collections::storage,
        coroutines::start_coroutine,
        scene::{self, Handle},
    },
};

use editor::{Editor, EditorCamera, EditorInputScheme};
use error::Result;

pub use game_world::GameWorld;
pub use input::{Input, InputScheme};

use map::{Map, MapLayerKind, MapObjectKind};

use nodes::Player;

use resources::MapResource;

use crate::items::Sproinger;
pub use resources::Resources;

mod capabilities;
mod gui;
mod input;
mod items;
mod nodes;

mod noise;

pub mod components;
pub mod json;
pub mod map;

pub mod editor;
pub mod game_world;
pub mod math;
pub mod resources;
pub mod text;
#[macro_use]
pub mod error;
pub mod effects;
pub mod debug;

pub use effects::{
    PassiveEffect,
    PassiveEffectParams,
    ActiveEffectParams,
    ActiveEffectKind,
    CustomActiveEffectCoroutine,
};

const ASSETS_DIR_ENV_VAR: &str = "FISHFIGHT_ASSETS";

pub type CollisionWorld = macroquad_platformer::World;

pub enum GameType {
    Local(Vec<InputScheme>),
    Editor {
        input_scheme: EditorInputScheme,
        is_new_map: bool,
    },
    Network {
        socket: std::net::UdpSocket,
        id: usize,
        input_scheme: InputScheme,
    },
}

fn build_game_scene(map: Map, is_local_game: bool) -> Vec<Handle<Player>> {
    use effects::active_effects::{Projectiles, TriggeredEffects};
    use nodes::{Camera, Decoration, ParticleEmitters, SceneRenderer};
    use items::Item;

    let resources = storage::get::<Resources>();
    let battle_music = resources.music["fish_tide"];

    audio::play_sound(
        battle_music,
        audio::PlaySoundParams {
            looped: true,
            volume: 0.6,
        },
    );

    let bounds = {
        let w = map.grid_size.x as f32 * map.tile_size.x;
        let h = map.grid_size.y as f32 * map.tile_size.y;
        Rect::new(0., 0., w, h)
    };

    scene::add_node(Camera::new(bounds));

    scene::add_node(SceneRenderer::new());

    let resources = storage::get::<Resources>();

    // Objects are cloned since Item constructor requires `GameWorld` in storage
    let mut map_objects = Vec::new();
    for layer in map.layers.values() {
        if layer.kind == MapLayerKind::ObjectLayer {
            map_objects.append(&mut layer.objects.clone());
        }
    }

    let mut spawn_points = Vec::new();
    let mut items = Vec::new();

    for object in map_objects {
        match object.kind {
            MapObjectKind::Decoration => {
                scene::add_node(Decoration::new(object.position, &object.id));
            }
            MapObjectKind::Environment => {
                if object.id == Sproinger::OBJECT_ID {
                    Sproinger::spawn(object.position);
                } else {
                    println!("WARNING: Invalid environment object id '{}'", &object.id);
                }
            }
            MapObjectKind::SpawnPoint => {
                spawn_points.push(object.position);
            }
            MapObjectKind::Item => {
                if let Some(params) = resources.items.get(&object.id).cloned() {
                    if params.is_network_ready || is_local_game {
                        items.push((object.position, params));
                    }
                } else {
                    println!("WARNING: Invalid item id '{}'", &object.id);
                }
            }
        }
    }

    storage::store(GameWorld::new(map, spawn_points));

    for (position, params) in items {
        scene::add_node(Item::new(position, params));
    }

    drop(resources);

    let players = vec![
        scene::add_node(Player::new(0, 0)),
        scene::add_node(Player::new(1, 1)),
    ];

    scene::add_node(TriggeredEffects::new());
    scene::add_node(Projectiles::new());
    scene::add_node(ParticleEmitters::new());

    players
}

async fn game(map_resource: MapResource, game_type: GameType) -> Result<()> {
    use nodes::{LocalNetwork, Network};

    match game_type {
        GameType::Local(players_input) => {
            assert_eq!(
                players_input.len(),
                2,
                "Local: There should be two player input schemes for this game mode"
            );

            let players = build_game_scene(map_resource.map, true);
            scene::add_node(LocalNetwork::new(players_input, players[0], players[1]));
        }
        GameType::Editor { input_scheme, .. } => {
            let position = map_resource.map.get_size() * 0.5;

            scene::add_node(EditorCamera::new(position));
            scene::add_node(Editor::new(input_scheme, map_resource));
        }
        GameType::Network {
            input_scheme,
            socket,
            id,
        } => {
            let players = build_game_scene(map_resource.map, false);
            scene::add_node(Network::new(
                id,
                socket,
                input_scheme,
                players[0],
                players[1],
            ));
        }
    }

    loop {
        {
            let mut gamepad_system = storage::get_mut::<GamepadContext>();
            gamepad_system.update()?;
        }

        next_frame().await;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "FishFight".to_owned(),
        high_dpi: false,
        window_width: 955,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<()> {
    let assets_dir = env::var(ASSETS_DIR_ENV_VAR).unwrap_or_else(|_| "./assets".to_string());

    {
        let gui_resources = gui::GuiResources::load(&assets_dir).await;
        storage::store(gui_resources);
    }

    rand::srand(0);

    let resources_loading = start_coroutine({
        let assets_dir = assets_dir.clone();
        async move {
            let resources = match Resources::new(&assets_dir).await {
                Ok(val) => val,
                Err(err) => panic!("{}: {}", err.kind().as_str(), err.to_string()),
            };

            storage::store(resources);
        }
    });

    while !resources_loading.is_done() {
        clear_background(BLACK);
        draw_text(
            &format!(
                "Loading resources {}",
                ".".repeat(((get_time() * 2.0) as usize) % 4)
            ),
            screen_width() / 2.0 - 160.0,
            screen_height() / 2.0,
            40.,
            WHITE,
        );

        next_frame().await;
    }

    {
        let gamepad_system = fishsticks::GamepadContext::init().unwrap();
        storage::store(gamepad_system);
    }

    loop {
        let game_type = gui::main_menu::game_type().await;

        let map_resource = match &game_type {
            GameType::Local(..) => gui::main_menu::location_select().await,
            GameType::Editor { is_new_map, .. } => {
                if *is_new_map {
                    gui::main_menu::create_map().await?
                } else {
                    gui::main_menu::location_select().await
                }
            }
            GameType::Network { .. } => {
                let resources = storage::get::<Resources>();

                resources
                    .maps
                    .iter()
                    .find(|res| res.meta.path.ends_with("level_01.json"))
                    .cloned()
                    .unwrap()
            }
        };

        game(map_resource, game_type).await?;

        scene::clear();
    }
}
