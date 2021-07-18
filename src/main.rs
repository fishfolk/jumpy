use macroquad::prelude::*;

use macroquad_particles as particles;
use macroquad_tiled as tiled;

use macroquad::{
    audio::{self, load_sound},
    experimental::{
        collections::storage,
        coroutines::start_coroutine,
        scene::{self, Handle},
    },
};

use macroquad_platformer::{Tile, World as CollisionWorld};
use particles::EmittersCache;

mod gui;
mod nodes;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameType {
    // No wining conditions, game going forever
    // Used for quick game
    Deathmatch,
    // Killed players got removed from the game, the last one wins
    LastFishStanding {
        // match was created as a private match for friend,
        // not as a matchmaking match
        private: bool,
    },
}

struct Resources {
    hit_fxses: EmittersCache,
    explosion_fxses: EmittersCache,
    life_explosion_fxses: EmittersCache,
    tiled_map: tiled::Map,
    collision_world: CollisionWorld,
    whale: Texture2D,
    whale_red: Texture2D,
    grenades: Texture2D,
    gun: Texture2D,
    mines: Texture2D,
    sword: Texture2D,
    sproinger: Texture2D,
    fish_sword: Texture2D,
    background_01: Texture2D,
    background_02: Texture2D,
    background_03: Texture2D,
    decorations: Texture2D,
    jump_sound: audio::Sound,
    shoot_sound: audio::Sound,
    sword_sound: audio::Sound,
    pickup_sound: audio::Sound,
}

pub const HIT_FX: &'static str = r#"{"local_coords":false,"emission_shape":"Point","one_shot":true,"lifetime":0.2,"lifetime_randomness":0,"explosiveness":0.65,"amount":41,"shape":{"Circle":{"subdivisions":10}},"emitting":false,"initial_direction":{"x":0,"y":-1},"initial_direction_spread":6.2831855,"initial_velocity":73.9,"initial_velocity_randomness":0.2,"linear_accel":0,"size":5.6000004,"size_randomness":0.4,"blend_mode":"Alpha","colors_curve":{"start":{"r":0.8200004,"g":1,"b":0.31818175,"a":1},"mid":{"r":0.71000004,"g":0.36210018,"b":0,"a":1},"end":{"r":0.02,"g":0,"b":0.000000007152557,"a":1}},"gravity":{"x":0,"y":0},"post_processing":{}}
"#;

pub const EXPLOSION_FX: &'static str = r#"{"local_coords":false,"emission_shape":{"Sphere":{"radius":0.6}},"one_shot":true,"lifetime":0.35,"lifetime_randomness":0,"explosiveness":0.6,"amount":131,"shape":{"Circle":{"subdivisions":10}},"emitting":false,"initial_direction":{"x":0,"y":-1},"initial_direction_spread":6.2831855,"initial_velocity":316,"initial_velocity_randomness":0.6,"linear_accel":-7.4000025,"size":5.5,"size_randomness":0.3,"size_curve":{"points":[[0.005,1.48],[0.255,1.0799999],[1,0.120000005]],"interpolation":"Linear","resolution":30},"blend_mode":"Additive","colors_curve":{"start":{"r":0.9825908,"g":1,"b":0.13,"a":1},"mid":{"r":0.8,"g":0.19999999,"b":0.2000002,"a":1},"end":{"r":0.101,"g":0.099,"b":0.099,"a":1}},"gravity":{"x":0,"y":-500},"post_processing":{}}
"#;

pub const LIFE_EXPLOSION_FX: &'static str = r#"{"local_coords":false,"emission_shape":{"Sphere":{"radius":1.9}},"one_shot":true,"lifetime":1.6500002,"lifetime_randomness":0,"explosiveness":1,"amount":131,"shape":{"Circle":{"subdivisions":20}},"emitting":false,"initial_direction":{"x":0,"y":-1.2},"initial_direction_spread":0.5,"initial_velocity":316,"initial_velocity_randomness":0.70000005,"linear_accel":0,"size":5.5,"size_randomness":0.3,"size_curve":{"points":[[0.005,1.48],[0.255,1.0799999],[1,0.120000005]],"interpolation":"Linear","resolution":30},"blend_mode":"Additive","colors_curve":{"start":{"r":0.9825908,"g":1,"b":0.13,"a":1},"mid":{"r":0.8,"g":0.19999999,"b":0.2000002,"a":1},"end":{"r":0.101,"g":0.099,"b":0.099,"a":1}},"gravity":{"x":0,"y":1000},"post_processing":{}}"#;

impl Resources {
    // TODO: fix macroquad error type here
    async fn new(map: &str) -> Result<Resources, macroquad::prelude::FileError> {
        let tileset = load_texture("assets/tileset.png").await?;
        tileset.set_filter(FilterMode::Nearest);

        let decorations = load_texture("assets/decorations1.png").await?;
        decorations.set_filter(FilterMode::Nearest);

        let whale_red = load_texture("assets/Whale/Whale(76x66)(Green).png").await?;
        whale_red.set_filter(FilterMode::Nearest);

        let whale = load_texture("assets/Whale/Whale(76x66)(Blue).png").await?;
        whale.set_filter(FilterMode::Nearest);

        let gun = load_texture("assets/Whale/Gun(92x32).png").await?;
        gun.set_filter(FilterMode::Nearest);

        let mines = load_texture("assets/Whale/Mines(30x15).png").await?;
        mines.set_filter(FilterMode::Nearest);

        let sword = load_texture("assets/Whale/Sword(65x93).png").await?;
        sword.set_filter(FilterMode::Nearest);

        let grenades = load_texture("assets/Whale/Grenades(15x15).png").await?;
        grenades.set_filter(FilterMode::Nearest);

        let sproinger = load_texture("assets/Whale/Sproinger(32x32).png").await?;
        sproinger.set_filter(FilterMode::Nearest);

        let fish_sword = load_texture("assets/Whale/FishSword.png").await?;
        fish_sword.set_filter(FilterMode::Nearest);

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

        let hit_fxses = EmittersCache::new(nanoserde::DeJson::deserialize_json(HIT_FX).unwrap());
        let explosion_fxses =
            EmittersCache::new(nanoserde::DeJson::deserialize_json(EXPLOSION_FX).unwrap());
        let life_explosion_fxses =
            EmittersCache::new(nanoserde::DeJson::deserialize_json(LIFE_EXPLOSION_FX).unwrap());

        Ok(Resources {
            hit_fxses,
            explosion_fxses,
            life_explosion_fxses,
            tiled_map,
            collision_world,
            whale,
            whale_red,
            grenades,
            gun,
            mines,
            sword,
            sproinger,
            fish_sword,
            background_01,
            background_02,
            background_03,
            decorations,
            jump_sound,
            shoot_sound,
            sword_sound,
            pickup_sound,
        })
    }
}

async fn game(game_type: GameType, map: &str) -> i32 {
    use nodes::{
        Bullets, Camera, Decoration, Fxses, GameState, Grenades,
        LevelBackground, Mines, Muscet, Player, ScoreCounter, Sword, Sproinger,
    };

    let resources_loading = start_coroutine({
        let map = map.to_string();
        async move {
            let resources = Resources::new(&map).await.unwrap();
            storage::store(resources);
        }
    });

    while resources_loading.is_done() == false {
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

    // let battle_music = if map == "assets/map.json" {
    //     load_sound("assets/music/across the pond.ogg")
    //         .await
    //         .unwrap()
    // } else {
    //     load_sound("assets/music/fish tide.ogg").await.unwrap()
    // };

    // audio::play_sound(
    //     battle_music,
    //     audio::PlaySoundParams {
    //         looped: true,
    //         volume: 0.6,
    //     },
    // );

    let bounds = {
        let resources = storage::get::<Resources>();

        let w =
            resources.tiled_map.raw_tiled_map.tilewidth * resources.tiled_map.raw_tiled_map.width;
        let h =
            resources.tiled_map.raw_tiled_map.tileheight * resources.tiled_map.raw_tiled_map.height;
        Rect::new(0., 0., w as f32, h as f32)
    };

    scene::add_node(Camera::new(bounds));

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

    let score_counter = scene::add_node(ScoreCounter::new());
    let player1 = scene::add_node(Player::new(
        game_type == GameType::Deathmatch,
        0,
        score_counter,
        Handle::null(),
    ));
    let player2 = scene::add_node(Player::new(
        game_type == GameType::Deathmatch,
        1,
        score_counter,
        Handle::null(),
    ));

    let mut wat_facing = false;
    for object in &objects {
        if object.name == "sword" {
            let mut sword =
                Sword::new(wat_facing, vec2(object.world_x - 35., object.world_y - 25.));
            sword.throw(false);
            scene::add_node(sword);
            wat_facing ^= true;
        }

        if object.name == "muscet" {
            let mut muscet =
                Muscet::new(wat_facing, vec2(object.world_x - 35., object.world_y - 25.));
            muscet.throw(false);
            scene::add_node(muscet);
            wat_facing ^= true;
        }

        if object.name == "mines" {
            let mut mines =
                Mines::new(wat_facing, vec2(object.world_x - 35., object.world_y - 25.));
            mines.throw(false);
            scene::add_node(mines);
            wat_facing ^= true;
        }

        if object.name == "grenades" {
            let mut grenade =
                Grenades::new(wat_facing, vec2(object.world_x - 35., object.world_y - 25.));
            grenade.throw(false);
            scene::add_node(grenade);
            wat_facing ^= true;
        }

        if object.name == "sproinger" {
            let sproinger =
                Sproinger::new(vec2(object.world_x - 35., object.world_y - 25.));
            scene::add_node(sproinger);
        }
    }

    scene::add_node(Bullets::new());

    //scene::add_node(Camera::new(player2));
    scene::add_node(Fxses {});

    let game_state = scene::add_node(GameState::new());

    scene::get_node(player1).game_state = game_state;
    scene::get_node(player2).game_state = game_state;

    loop {
        {
            let mut controller = storage::get_mut::<gamepad_rs::ControllerContext>();
            for i in 0..2 {
                controller.update(i);
            }
        }

        if scene::find_node_by_type::<GameState>().unwrap().want_quit {
            return -1;
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
        let controller = gamepad_rs::ControllerContext::new().unwrap();
        storage::store(controller);
    }

    {
        let gui_resources = gui::GuiResources::load().await;
        storage::store(gui_resources);
    }

    loop {
        let map = gui::main_menu::gui().await;
        //let map = "assets/levels/lev06.json";
        let res = game(GameType::Deathmatch, &map).await;

        scene::clear();

        if res != -1 {
            for _ in 0..100 {
                if res == 1 {
                    clear_background(Color::from_rgba(126, 168, 166, 255));
                } else {
                    clear_background(Color::from_rgba(126, 178, 126, 255));
                }

                let resources = storage::get::<Resources>();

                draw_texture_ex(
                    if res == 1 {
                        resources.whale
                    } else {
                        resources.whale_red
                    },
                    0.,
                    0.0,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(0.0, 0.0, 76., 66.)),
                        dest_size: Some(vec2(screen_width(), screen_height())),
                        ..Default::default()
                    },
                );
                next_frame().await;
            }
        }
    }
}
