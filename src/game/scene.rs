use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, Handle, RefMut},
    },
    prelude::*,
};

use crate::player::PlayerCharacterParams;
use crate::{
    Decoration, GameCamera, GameWorld, Item, Map, MapLayerKind, MapObjectKind, ParticleEmitters,
    Player, Projectiles, Resources, Sproinger, TriggeredEffects,
};

#[derive(Default)]
pub struct GameScene;

impl GameScene {
    pub fn new() -> GameScene {
        GameScene {}
    }
}

impl scene::Node for GameScene {
    fn draw(_: RefMut<Self>) {
        let world = storage::get::<GameWorld>();
        world.map.draw(None, true);
    }
}

pub fn create_game_scene(
    map: Map,
    player_characters: Vec<PlayerCharacterParams>,
    is_local_game: bool,
) -> Vec<Handle<Player>> {
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
        if layer.is_visible && layer.kind == MapLayerKind::ObjectLayer {
            map_objects.append(&mut layer.objects.clone());
        }
    }

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

    storage::store(GameWorld::new(map));

    for (position, params) in items {
        scene::add_node(Item::new(position, params));
    }

    drop(resources);

    let players = vec![
        scene::add_node(Player::new(0, player_characters[0].clone())),
        scene::add_node(Player::new(1, player_characters[1].clone())),
    ];

    scene::add_node(TriggeredEffects::new());
    scene::add_node(Projectiles::new());
    scene::add_node(ParticleEmitters::new());

    players
}
