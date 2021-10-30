use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, Handle, RefMut},
    },
    prelude::*,
};

use crate::{
    Decoration, GameCamera, GameWorld, Item, Map, MapLayerKind, MapObjectKind, ParticleControllers,
    ParticleEmitters, Player, Projectiles, Resources, Sproinger, TriggeredEffects,
};

#[derive(Default)]
pub struct GameScene;

impl GameScene {
    pub fn new() -> GameScene {
        GameScene {}
    }
}

fn parallax(texture: Texture2D, depth: f32, camera_pos: Vec2) -> Rect {
    let w = texture.width();
    let h = texture.height();

    let dest_rect = Rect::new(0., 0., w, h);
    let parallax_w = w as f32 * 0.5;

    let mut dest_rect2 = Rect::new(
        -parallax_w,
        -parallax_w,
        w + parallax_w * 2.,
        h + parallax_w * 2.,
    );

    let parallax_x = camera_pos.x / dest_rect.w - 0.3;
    let parallax_y = camera_pos.y / dest_rect.h * 0.6 - 0.5;

    dest_rect2.x += parallax_w * parallax_x * depth;
    dest_rect2.y += parallax_w * parallax_y * depth;

    dest_rect2
}

impl scene::Node for GameScene {
    fn draw(_: RefMut<Self>) {
        {
            let resources = storage::get::<Resources>();
            let pos = scene::camera_pos();

            clear_background(Color::from_rgba(126, 168, 166, 255));

            {
                let texture_entry = resources.textures.get("background_03").unwrap();
                let dest_rect = parallax(texture_entry.texture, 2.0, pos);
                draw_texture_ex(
                    texture_entry.texture,
                    dest_rect.x,
                    80.0 + dest_rect.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(dest_rect.w, dest_rect.h)),
                        ..Default::default()
                    },
                );
            }

            {
                let texture_entry = resources.textures.get("background_02").unwrap();
                let dest_rect = parallax(texture_entry.texture, 1.0, pos);
                draw_texture_ex(
                    texture_entry.texture,
                    dest_rect.x,
                    120.0 + dest_rect.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(dest_rect.w, dest_rect.h)),
                        ..Default::default()
                    },
                );
            }

            {
                let texture_entry = resources.textures.get("background_01").unwrap();
                let dest_rect = parallax(texture_entry.texture, 0.5, pos);
                draw_texture_ex(
                    texture_entry.texture,
                    dest_rect.x,
                    180.0 + dest_rect.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(dest_rect.w, dest_rect.h)),
                        ..Default::default()
                    },
                );
            }
        }

        let world = storage::get::<GameWorld>();
        world.map.draw(false, None);
    }
}

pub fn create_game_scene(map: Map, is_local_game: bool) -> Vec<Handle<Player>> {
    let bounds = {
        let w = map.grid_size.x as f32 * map.tile_size.x;
        let h = map.grid_size.y as f32 * map.tile_size.y;
        Rect::new(0., 0., w, h)
    };

    scene::add_node(GameCamera::new(bounds));

    scene::add_node(GameScene::new());

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
    scene::add_node(ParticleControllers::default());

    players
}
