use std::collections::HashMap;

use macroquad::prelude::*;

use macroquad_particles as particles;

use macroquad::{
    audio::{self, load_sound},
    experimental::{
        collections::storage,
        coroutines::start_coroutine,
        scene::{self, Handle},
    },
};

use macroquad_platformer::Tile;
use particles::{Emitter, EmittersCache};

use std::env;

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
pub mod math;
pub mod text;
pub mod weapons;
pub mod game_world;
pub mod resources;

pub use resources::Resources;
pub use game_world::GameWorld;

use editor::{Editor, EditorCamera, EditorInputScheme, DEFAULT_TOOL_ICON_TEXTURE_ID};

pub use input::{Input, InputScheme};

use crate::nodes::Player;
use map::Map;

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

async fn build_game_scene(map: Map, assets_dir: &str, is_local_game: bool) -> Vec<Handle<Player>> {
    use nodes::{Camera, Decoration, Fxses, SceneRenderer};

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

    for object in &map.layers["decorations"].objects {
        scene::add_node(Decoration::new(object.position, &object.id));
    }

    let objects = map.layers["items"].objects.clone();

    storage::store(GameWorld::new(map));

    drop(resources);

    let players = vec![
        scene::add_node(Player::new(0, 0)),
        scene::add_node(Player::new(1, 1)),
    ];

    for object in &objects {
        for item_desc in items::ITEMS {
            if object.id == item_desc.tiled_name && (is_local_game || item_desc.network_ready) {
                (item_desc.constructor)(vec2(
                    object.position.x + item_desc.tiled_offset.0,
                    object.position.y + item_desc.tiled_offset.1,
                ));
            }
        }
    }

    scene::add_node(Fxses {});

    players
}

async fn game(map: Map, game_type: GameType, assets_dir: &str) {
    use nodes::{LocalNetwork, Network};

    match game_type {
        GameType::Local(players_input) => {
            assert_eq!(
                players_input.len(),
                2,
                "There should be two player input schemes for this game mode!"
            );

            let players = build_game_scene(map, assets_dir, true).await;
            scene::add_node(LocalNetwork::new(players_input, players[0], players[1]));
        }
        GameType::Editor { input_scheme, .. } => {
            let position = map.get_size() * 0.5;

            scene::add_node(EditorCamera::new(position));
            scene::add_node(Editor::new(input_scheme, map));
        }
        GameType::Network {
            input_scheme,
            socket,
            id,
        } => {
            let players = build_game_scene(map, assets_dir, false).await;
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
            let mut gui_resources = storage::get_mut::<crate::gui::GuiResources>();
            gui_resources.gamepads.update();
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
async fn main() {
    let assets_dir = env::var("FISHFIGHT_ASSETS").unwrap_or_else(|_| "./assets".to_string());

    {
        let gui_resources = gui::GuiResources::load(&assets_dir).await;
        storage::store(gui_resources);
    }

    rand::srand(0);

    let resources_loading = start_coroutine({
        let assets_dir = assets_dir.clone();
        async move {
            let resources = Resources::new(&assets_dir).await.unwrap();
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

    loop {
        let game_type = gui::main_menu::game_type().await;

        let map = match &game_type {
            GameType::Local(..) => {
                let map_entry = gui::main_menu::location_select().await;
                Map::load_tiled(&map_entry.path, None).await.unwrap()
            }
            GameType::Editor { is_new_map, .. } => {
                if *is_new_map {
                    let (name, tile_size, grid_size) = gui::main_menu::new_map().await;
                    Map::new(&name, tile_size, grid_size)
                } else {
                    let map_entry = gui::main_menu::location_select().await;
                    if map_entry.is_tiled {
                        Map::load_tiled(&map_entry.path, None).await.unwrap()
                    } else {
                        Map::load(&map_entry.path).await.unwrap()
                    }
                }
            }
            GameType::Network { .. } => Map::load_tiled("assets/levels/lev01.json", None)
                .await
                .unwrap(),
        };

        game(map, game_type, &assets_dir).await;

        scene::clear();
    }
}
