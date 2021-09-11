use macroquad::prelude::*;

use macroquad_particles as particles;
use macroquad_tiled as tiled;

use macroquad::{
    audio::{self, load_sound},
    experimental::{collections::storage, coroutines::start_coroutine, scene},
};

use macroquad_platformer::{Tile, World as CollisionWorld};
use particles::EmittersCache;

use std::collections::HashMap;

mod capabilities;
mod gui;
mod input;
mod items;
mod nodes;
mod noise;

pub mod components;

pub use input::{Input, InputScheme};

pub enum GameType {
    Local(Vec<InputScheme>),
    Network {
        id: usize,
        self_addr: String,
        other_addr: String,
        input_scheme: InputScheme,
    },
}

struct Resources {
    hit_fxses: EmittersCache,
    explosion_fxses: EmittersCache,
    life_ui_explosion_fxses: EmittersCache,
    tiled_map: tiled::Map,
    collision_world: CollisionWorld,
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
}

impl Resources {
    // TODO: fix macroquad error type here
    async fn new(map: &str) -> Result<Resources, macroquad::prelude::FileError> {
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

        let tiled_map_json = load_string(map).await.unwrap();
        let tiled_map = tiled::load_map(
            &tiled_map_json,
            &[("tileset.png", tileset), ("decorations1.png", decorations)],
            &[],
        )
        .unwrap();

        let mut static_colliders = vec![];
        for (_x, _y, tile) in tiled_map.tiles("main layer", None) {
            static_colliders.push(match tile {
                None => Tile::Empty,
                Some(tile) if tile.attrs.contains("jumpthrough") => Tile::JumpThrough,
                _ => Tile::Solid,
            });
        }
        let mut collision_world = CollisionWorld::new();
        collision_world.add_static_tiled_layer(
            static_colliders,
            32.,
            32.,
            tiled_map.raw_tiled_map.width as _,
            1,
        );

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

        #[allow(clippy::inconsistent_struct_constructor)]
        Ok(Resources {
            hit_fxses,
            explosion_fxses,
            life_ui_explosion_fxses,
            items_fxses,
            tiled_map,
            collision_world,
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
        })
    }
}

async fn game(map: &str, game_type: GameType) {
    use nodes::{Camera, Decoration, Fxses, LevelBackground, LocalNetwork, Network, Player};

    let resources_loading = start_coroutine({
        let map = map.to_string();
        async move {
            let resources = Resources::new(&map).await.unwrap();
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

    let battle_music = if map == "assets/map.json" {
        load_sound("assets/music/across the pond.ogg")
            .await
            .unwrap()
    } else {
        load_sound("assets/music/fish tide.ogg").await.unwrap()
    };

    audio::play_sound(
        battle_music,
        audio::PlaySoundParams {
            looped: true,
            volume: 0.6,
        },
    );

    let bounds = {
        let resources = storage::get::<Resources>();

        let w =
            resources.tiled_map.raw_tiled_map.tilewidth * resources.tiled_map.raw_tiled_map.width;
        let h =
            resources.tiled_map.raw_tiled_map.tileheight * resources.tiled_map.raw_tiled_map.height;
        Rect::new(0., 0., w as f32, h as f32)
    };

    let resources = storage::get::<Resources>();

    let _level_background = scene::add_node(LevelBackground::new());

    for object in &resources.tiled_map.layers["decorations"].objects {
        scene::add_node(Decoration::new(
            vec2(object.world_x, object.world_y),
            &object.name,
        ));
    }

    let objects = resources.tiled_map.layers["items"].objects.clone();

    drop(resources);

    let player1 = scene::add_node(Player::new(0, 0));
    let player2 = scene::add_node(Player::new(1, 1));

    match game_type {
        GameType::Local(players_input) => {
            assert!(
                players_input.len() == 2,
                "Only 2 player games are supported now"
            );
            scene::add_node(LocalNetwork::new(players_input, player1, player2));
        }
        GameType::Network {
            id,
            ref self_addr,
            ref other_addr,
            input_scheme,
        } => {
            scene::add_node(Network::new(
                input_scheme,
                player1,
                player2,
                id,
                self_addr,
                other_addr,
            ));
        }
    }

    scene::add_node(Camera::new(bounds));

    for object in &objects {
        for item_desc in items::ITEMS {
            if object.name == item_desc.tiled_name {
                (item_desc.constructor)(vec2(
                    object.world_x + item_desc.tiled_offset.0,
                    object.world_y + item_desc.tiled_offset.1,
                ));
            }
        }
    }

    scene::add_node(Fxses {});

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
        window_title: "FISH".to_owned(),
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

    loop {
        let game_type = gui::main_menu::game_type().await;

        let map = match game_type {
            GameType::Local(..) => gui::main_menu::location_select().await,
            GameType::Network { .. } => "assets/levels/lev01.json".to_string(),
        };

        game(&map, game_type).await;

        scene::clear();
    }
}
