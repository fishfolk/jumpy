use macroquad::prelude::*;
use std::collections::HashMap;

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

use editor::{Editor, EditorCamera, EditorInputScheme};

pub use input::{Input, InputScheme};

use crate::{map::CollisionKind, nodes::Player};
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
                for (i, (_, _, tile)) in map.get_tiles(layer_id, None).enumerate() {
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
                                CollisionKind::None => {}
                            }
                        }
                    }
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
    fx_explosion_fire: Emitter,
    fx_explosion_particles: EmittersCache,
    fx_smoke: Emitter,
    whale_green: Texture2D,
    whale_blue: Texture2D,
    whale_boots_blue: Texture2D,
    whale_boots_green: Texture2D,
    broken_turtleshell: Texture2D,
    turtleshell: Texture2D,
    flappy_jellyfish: Texture2D,
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
    async fn new(assets_dir: &str) -> Result<Resources, macroquad::prelude::FileError> {
        let tileset = load_texture(&format!("{}/assets/tileset.png", assets_dir)).await?;
        tileset.set_filter(FilterMode::Nearest);

        let decorations = load_texture(&format!("{}/assets/decorations1.png", assets_dir)).await?;
        decorations.set_filter(FilterMode::Nearest);

        let whale_green = load_texture(&format!(
            "{}/assets/Whale/Whale(76x66)(Green).png",
            assets_dir
        ))
        .await?;

        whale_green.set_filter(FilterMode::Nearest);

        let whale_blue = load_texture(&format!(
            "{}/assets/Whale/Whale(76x66)(Blue).png",
            assets_dir
        ))
        .await?;

        whale_blue.set_filter(FilterMode::Nearest);

        let whale_boots_green = load_texture(&format!(
            "{}/assets/Whale/WhaleBoots(76x66)(Green).png",
            assets_dir
        ))
        .await?;

        whale_boots_green.set_filter(FilterMode::Nearest);

        let whale_boots_blue = load_texture(&format!(
            "{}/assets/Whale/WhaleBoots(76x66)(Blue).png",
            assets_dir
        ))
        .await?;

        whale_boots_blue.set_filter(FilterMode::Nearest);

        let broken_turtleshell = load_texture(&format!(
            "{}/assets/Whale/BrokenTurtleShell(32x32).png",
            assets_dir
        ))
        .await?;

        broken_turtleshell.set_filter(FilterMode::Nearest);

        let turtleshell = load_texture(&format!(
            "{}/assets/Whale/TurtleShell(32x32).png",
            assets_dir
        ))
        .await?;

        turtleshell.set_filter(FilterMode::Nearest);

        let flappy_jellyfish = load_texture(&format!(
            "{}/assets/Whale/FlappyJellyfish(34x47).png",
            assets_dir
        ))
        .await?;

        flappy_jellyfish.set_filter(FilterMode::Nearest);

        let background_01 =
            load_texture(&format!("{}/assets/Background/01.png", assets_dir)).await?;
        background_01.set_filter(FilterMode::Nearest);

        let background_02 =
            load_texture(&format!("{}/assets/Background/02.png", assets_dir)).await?;
        background_02.set_filter(FilterMode::Nearest);

        let background_03 =
            load_texture(&format!("{}/assets/Background/03.png", assets_dir)).await?;
        background_03.set_filter(FilterMode::Nearest);

        let jump_sound = load_sound(&format!("{}/assets/sounds/jump.wav", assets_dir)).await?;
        let shoot_sound = load_sound(&format!("{}/assets/sounds/shoot.ogg", assets_dir)).await?;
        let sword_sound = load_sound(&format!("{}/assets/sounds/sword.wav", assets_dir)).await?;
        let pickup_sound = load_sound(&format!("{}/assets/sounds/pickup.wav", assets_dir)).await?;
        let player_landing_sound =
            load_sound(&format!("{}/assets/sounds/player_landing.wav", assets_dir)).await?;
        let player_throw_sound =
            load_sound(&format!("{}/assets/sounds/throw_noiz.wav", assets_dir)).await?;
        let player_die_sound =
            load_sound(&format!("{}/assets/sounds/fish_fillet.wav", assets_dir)).await?;

        const HIT_FX: &str = include_str!("../assets/fxses/hit.json");
        const EXPLOSION_FX: &str = include_str!("../assets/fxses/explosion.json");
        const LIFE_UI_FX: &str = include_str!("../assets/fxses/life_ui_explosion.json");
        const CANNONBALL_HIT_FX: &str = include_str!("../assets/fxses/canonball_hit.json");
        const EXPLOSION_PARTICLES: &str = include_str!("../assets/fxses/explosion_particles.json");
        const SMOKE_FX: &str = include_str!("../assets/fxses/smoke.json");
        let hit_fxses = EmittersCache::new(nanoserde::DeJson::deserialize_json(HIT_FX).unwrap());
        let explosion_fxses =
            EmittersCache::new(nanoserde::DeJson::deserialize_json(EXPLOSION_FX).unwrap());
        let life_ui_explosion_fxses =
            EmittersCache::new(nanoserde::DeJson::deserialize_json(LIFE_UI_FX).unwrap());
        let fx_explosion_fire =
            Emitter::new(nanoserde::DeJson::deserialize_json(CANNONBALL_HIT_FX).unwrap());
        let fx_explosion_particles =
            EmittersCache::new(nanoserde::DeJson::deserialize_json(EXPLOSION_PARTICLES).unwrap());
        let fx_smoke = Emitter::new(nanoserde::DeJson::deserialize_json(SMOKE_FX).unwrap());

        let mut items_textures = HashMap::new();
        let mut items_fxses = HashMap::new();
        for item in items::ITEMS {
            for (id, path) in item.textures {
                let texture = load_texture(&format!("{}/{}", assets_dir, path)).await?;
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
        textures.insert("tileset".to_string(), tileset);
        textures.insert("decorations".to_string(), decorations);

        #[allow(clippy::inconsistent_struct_constructor)]
        Ok(Resources {
            hit_fxses,
            explosion_fxses,
            life_ui_explosion_fxses,
            fx_smoke,
            items_fxses,
            whale_blue,
            whale_green,
            whale_boots_blue,
            whale_boots_green,
            turtleshell,
            broken_turtleshell,
            flappy_jellyfish,
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
            fx_explosion_fire,
            fx_explosion_particles,
        })
    }
}

async fn build_game_scene(map: Map, assets_dir: &str, is_local_game: bool) -> Vec<Handle<Player>> {
    use nodes::{Camera, Decoration, Fxses, SceneRenderer};

    let resources_loading = start_coroutine({
        let assets_dir = assets_dir.to_string();
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

    let battle_music = load_sound(&format!("{}/assets/music/fish tide.ogg", assets_dir))
        .await
        .unwrap();

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
        scene::add_node(Decoration::new(object.position, &object.name));
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
    let assets_dir = env::var("FISHFIGHT_ASSETS").unwrap_or_else(|_| String::from("."));
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
                let level = gui::main_menu::location_select().await;
                Map::load_tiled(&level.map, None).await.unwrap()
            }
            GameType::Editor { is_new_map, .. } => {
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
            GameType::Network { .. } => Map::load_tiled("assets/levels/lev01.json", None)
                .await
                .unwrap(),
        };

        game(map, game_type, &assets_dir).await;

        scene::clear();
    }
}
