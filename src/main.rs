use std::ops::Deref;

use std::collections::HashMap;
use macroquad::prelude::*;

use macroquad_particles as particles;

use macroquad_tiled as tiled;
use macroquad::{
    audio::{self, load_sound},
    experimental::{
        collections::storage,
        coroutines::start_coroutine,
        scene::{
            self,
            Handle,
        },
    },
};

use macroquad_platformer::Tile;

use particles::EmittersCache;
mod capabilities;
mod gui;
mod input;
mod items;
mod nodes;

mod noise;

pub mod components;
pub mod json;
pub mod map;

pub mod math;
pub mod editor;

use editor::{
    Editor,
    EditorCamera,
    EditorInputScheme,
};

pub use input::{Input, InputScheme};

use map::Map;
use crate::{
    nodes::Player,
    map::CollisionKind,
};

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

pub struct GameWorld {
    pub map: Map,
    pub collision_world: CollisionWorld,
}

impl GameWorld {
    pub fn new(map: Map) -> Self {
        let tile_cnt = (map.grid_size.x * map.grid_size.y) as usize;
        let mut static_colliders = Vec::with_capacity(tile_cnt);
        for _ in 0..tile_cnt {
            static_colliders.push(Tile::Empty);
        }

        for layer_id in &map.draw_order {
            let layer = map.layers.get(layer_id).unwrap();
            if layer.collision != CollisionKind::None {
                let mut i = 0;
                for (_, _, tile) in map.get_tiles(layer_id, None) {
                    if let Some(tile) = tile {
                        if tile.attributes.contains(&"jumpthrough".to_string()) {
                            static_colliders[i] = Tile::JumpThrough;
                        } else {
                            match layer.collision {
                                CollisionKind::Solid => {
                                    static_colliders[i] = Tile::Solid;
                                }
                                CollisionKind::Barrier => {
                                    static_colliders[i] = Tile::Solid;
                                }
                                CollisionKind::None => {},
                            }
                        }
                    }
                    i += 1;
                }
            }
        }

        let mut collision_world = CollisionWorld::new();
        collision_world.add_static_tiled_layer(
            static_colliders,
            map.tile_size.x,
            map.tile_size.y,
            map.grid_size.x as usize,
            1,
        );

        GameWorld {
            map,
            collision_world,
        }
    }
}

struct Resources {
    hit_fxses: EmittersCache,
    explosion_fxses: EmittersCache,
    life_ui_explosion_fxses: EmittersCache,
    whale_green: Texture2D,
    whale_blue: Texture2D,
    whale_boots_blue: Texture2D,
    whale_boots_green: Texture2D,
    broken_turtleshell: Texture2D,
    turtleshell: Texture2D,
    background_01: Texture2D,
    background_02: Texture2D,
    background_03: Texture2D,
    decorations: Texture2D,
    jump_sound: audio::Sound,
    shoot_sound: audio::Sound,
    sword_sound: audio::Sound,
    pickup_sound: audio::Sound,
    player_landing_sound: audio::Sound,
    player_throw_sound: audio::Sound,
    player_die_sound: audio::Sound,
    items_textures: HashMap<String, Texture2D>,
    items_fxses: HashMap<String, EmittersCache>,

    // This holds textures that can be referenced by an ID
    textures: HashMap<String, Texture2D>,
}

impl Resources {
    // TODO: fix macroquad error type here
    async fn new() -> Result<Resources, macroquad::prelude::FileError> {
        let tileset = load_texture("assets/tileset.png").await?;
        tileset.set_filter(FilterMode::Nearest);

        let decorations = load_texture("assets/decorations1.png").await?;
        decorations.set_filter(FilterMode::Nearest);

        let whale_green = load_texture("assets/Whale/Whale(76x66)(Green).png").await?;
        whale_green.set_filter(FilterMode::Nearest);

        let whale_blue = load_texture("assets/Whale/Whale(76x66)(Blue).png").await?;
        whale_blue.set_filter(FilterMode::Nearest);

        let whale_boots_green = load_texture("assets/Whale/WhaleBoots(76x66)(Green).png").await?;
        whale_boots_green.set_filter(FilterMode::Nearest);

        let whale_boots_blue = load_texture("assets/Whale/WhaleBoots(76x66)(Blue).png").await?;
        whale_boots_blue.set_filter(FilterMode::Nearest);

        let broken_turtleshell = load_texture("assets/Whale/BrokenTurtleShell(32x32).png").await?;
        broken_turtleshell.set_filter(FilterMode::Nearest);

        let turtleshell = load_texture("assets/Whale/TurtleShell(32x32).png").await?;
        turtleshell.set_filter(FilterMode::Nearest);

        let background_01 = load_texture("assets/Background/01.png").await?;
        background_01.set_filter(FilterMode::Nearest);

        let background_02 = load_texture("assets/Background/02.png").await?;
        background_02.set_filter(FilterMode::Nearest);

        let background_03 = load_texture("assets/Background/03.png").await?;
        background_03.set_filter(FilterMode::Nearest);

        let jump_sound = load_sound("assets/sounds/jump.wav").await?;
        let shoot_sound = load_sound("assets/sounds/shoot.ogg").await?;
        let sword_sound = load_sound("assets/sounds/sword.wav").await?;
        let pickup_sound = load_sound("assets/sounds/pickup.wav").await?;
        let player_landing_sound = load_sound("assets/sounds/player_landing.wav").await?;
        let player_throw_sound = load_sound("assets/sounds/throw_noiz.wav").await?;
        let player_die_sound = load_sound("assets/sounds/fish_fillet.wav").await?;

        const HIT_FX: &str = include_str!("../assets/fxses/hit.json");
        const EXPLOSION_FX: &str = include_str!("../assets/fxses/explosion.json");
        const LIFE_UI_FX: &str = include_str!("../assets/fxses/life_ui_explosion.json");

        let hit_fxses = EmittersCache::new(nanoserde::DeJson::deserialize_json(HIT_FX).unwrap());
        let explosion_fxses =
            EmittersCache::new(nanoserde::DeJson::deserialize_json(EXPLOSION_FX).unwrap());
        let life_ui_explosion_fxses =
            EmittersCache::new(nanoserde::DeJson::deserialize_json(LIFE_UI_FX).unwrap());

        let mut items_textures = HashMap::new();
        let mut items_fxses = HashMap::new();
        for item in items::ITEMS {
            for (id, path) in item.textures {
                let texture = load_texture(path).await?;
                texture.set_filter(FilterMode::Nearest);
                items_textures.insert(format!("{}/{}", item.tiled_name, id.to_string()), texture);
            }

            for (id, path) in item.fxses {
                let json = load_string(path).await?;
                let emitter_cache =
                    EmittersCache::new(nanoserde::DeJson::deserialize_json(&json).unwrap());
                items_fxses.insert(
                    format!("{}/{}", item.tiled_name, id.to_string()),
                    emitter_cache,
                );
            }
        }

        let mut textures = HashMap::new();
        textures.insert("tileset".to_string(), tileset.clone());
        textures.insert("decorations".to_string(), decorations.clone());

        #[allow(clippy::inconsistent_struct_constructor)]
            Ok(Resources {
            hit_fxses,
            explosion_fxses,
            life_ui_explosion_fxses,
            items_fxses,
            whale_blue,
            whale_green,
            whale_boots_blue,
            whale_boots_green,
            turtleshell,
            broken_turtleshell,
            background_01,
            background_02,
            background_03,
            decorations,
            jump_sound,
            shoot_sound,
            sword_sound,
            pickup_sound,
            player_landing_sound,
            player_throw_sound,
            player_die_sound,
            items_textures,
            textures,
        })
    }
}

async fn setup_game_scene(map: Map, is_local_game: bool) -> Vec<Handle<Player>> {
    use nodes::{Camera, SceneRenderer, Decoration, Fxses};

    let battle_music = load_sound("assets/music/fish tide.ogg").await.unwrap();

    audio::play_sound(
        battle_music,
        audio::PlaySoundParams {
            looped: true,
            volume: 0.6,
        },
    );

    let bounds = {
        let w =
            map.grid_size.x as f32 * map.tile_size.x;
        let h =
            map.grid_size.y as f32 * map.tile_size.y;
        Rect::new(0., 0., w, h)
    };

    scene::add_node(Camera::new(bounds));

    scene::add_node(SceneRenderer::new());

    let resources = storage::get::<Resources>();

    for object in &map.layers["decorations"].objects {
        scene::add_node(Decoration::new(
            object.position,
            &object.name,
        ));
    }

    let objects = map.layers["items"].objects.clone();

    storage::store(GameWorld::new(map));

    drop(resources);

    let players = vec!(
        scene::add_node(Player::new(0, 0)),
        scene::add_node(Player::new(1, 1)),
    );

    for object in &objects {
        for item_desc in items::ITEMS {
            if object.name == item_desc.tiled_name && (is_local_game || item_desc.network_ready) {
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

async fn game(map: Map, game_type: GameType) {
    use nodes::{LocalNetwork, Network, Player};
    use editor::{EditorCamera, Editor};

    match game_type {
        GameType::Local(players_input) => {
            assert_eq!(players_input.len(), 2, "There should be two player input schemes for this game mode!");

            let players = setup_game_scene(map, true).await;
            scene::add_node(LocalNetwork::new(players_input, players[0], players[1]));
        }
        GameType::Editor {
            input_scheme,
            ..
        } => {
            scene::add_node(EditorCamera::new(Vec2::ZERO));
            scene::add_node(Editor::new(input_scheme, map));
        }
        GameType::Network {
            input_scheme,
            socket,
            id,
        } => {
            let players = setup_game_scene(map, false).await;
            scene::add_node(Network::new(id, socket, input_scheme, players[0], players[1]));
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
    {
        let gui_resources = gui::GuiResources::load().await;
        storage::store(gui_resources);
    }

    rand::srand(0);

    let resources_loading = start_coroutine({
        async move {
            let resources = Resources::new().await.unwrap();
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
                let level = gui::main_menu::location_select().await;
                Map::load_tiled(&level.map, None).await.unwrap()
            },
            GameType::Editor { input_scheme: _, is_new_map } => {
                if *is_new_map {
                    let (name, tile_size, grid_size) = gui::main_menu::new_map().await;
                    Map::new(&name, tile_size, grid_size)
                } else {
                    let level = gui::main_menu::location_select().await;
                    if level.is_tiled {
                        Map::load_tiled(&level.map, None).await.unwrap()
                    } else {
                        Map::load(&level.map).await.unwrap()
                    }
                }
            }
            GameType::Network { .. } => {
                Map::load_tiled("assets/levels/lev01.json", None).await.unwrap()
            },
        };

        game(map, game_type).await;

        scene::clear();
    }
}
