mod camera;
mod music;

pub use camera::GameCamera;

use fishsticks::{Button, GamepadContext};

use hecs::{Entity, World};

use core::prelude::*;
use core::Result;

use crate::{debug, macroquad, PlayerControllerKind};
use crate::physics::{debug_draw_physics_bodies, fixed_update_physics_bodies};
use crate::player::{
    draw_weapons_hud, spawn_player, update_player_animations, update_player_camera_box,
    update_player_controllers, update_player_events, update_player_inventory,
    update_player_passive_effects, update_player_states, PlayerParams,
};
use crate::{
    create_collision_world, debug_draw_drawables, debug_draw_rigid_bodies, draw_drawables,
    fixed_update_rigid_bodies, update_animated_sprites, Map,
    MapLayerKind, MapObjectKind, Resources,
};

use crate::effects::active::debug_draw_active_effects;
use crate::effects::active::projectiles::fixed_update_projectiles;
use crate::effects::active::triggered::fixed_update_triggered_effects;
use crate::items::spawn_item;
use crate::map::{fixed_update_sproingers, spawn_decoration, spawn_sproinger};
use crate::network::{
    fixed_update_network_client, fixed_update_network_host, update_network_client,
    update_network_host,
};
use crate::particles::{draw_particles, update_particle_emitters};
pub use music::{start_music, stop_music};
use crate::macroquad::time::get_frame_time;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GameMode {
    Local,
    NetworkHost,
    NetworkClient,
}

pub fn create_main_game_state(game_mode: GameMode) -> Box<dyn GameState> {
    let mut state_builder = GameStateBuilder::new();

    if matches!(game_mode, GameMode::NetworkClient) {
        state_builder.add_update(update_network_client);
        state_builder.add_fixed_update(fixed_update_network_client);
    } else if matches!(game_mode, GameMode::NetworkHost) {
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
            .add_update(update_player_states)
            .add_update(update_player_inventory)
            .add_update(update_player_passive_effects)
            .add_update(update_player_events);

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

    Box::new(state_builder.build())
}

pub fn spawn_map_objects(world: &mut World, map: &Map) -> Result<Vec<Entity>> {
    let mut objects = Vec::new();

    for layer in map.layers.values() {
        if layer.is_visible && layer.kind == MapLayerKind::ObjectLayer {
            for map_object in &layer.objects {
                match map_object.kind {
                    MapObjectKind::Decoration => {
                        let resources = storage::get::<Resources>();
                        let res = resources.decoration.get(&map_object.id).cloned();

                        if let Some(params) = res {
                            let decoration = spawn_decoration(world, map_object.position, params);
                            objects.push(decoration);
                        } else {
                            #[cfg(debug_assertions)]
                            println!("WARNING: Invalid decoration id '{}'", &map_object.id)
                        }
                    }
                    MapObjectKind::Item => {
                        let resources = storage::get::<Resources>();
                        let res = resources.items.get(&map_object.id).cloned();

                        if let Some(params) = res {
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

cfg_if! {
    if #[cfg(not(feature = "ultimate"))] {
        use macroquad::prelude::scene::{self, Node, RefMut};

        pub struct Game {
            world: Option<World>,
            state: Box<dyn GameState>,
        }

        impl Game {
            pub fn new(mode: GameMode, map: Map, players: &[PlayerParams]) -> Self {
                let state = create_main_game_state(mode);

                let mut world = World::new();

                {
                    let camera = GameCamera::new(map.get_size());
                    storage::store(camera);

                    let collision_world = create_collision_world(&map);
                    storage::store(collision_world);
                }

                spawn_map_objects(&mut world, &map).unwrap();

                for params in players {
                    let position = map.get_random_spawn_point();

                    spawn_player(
                        &mut world,
                        params.index,
                        position,
                        params.controller.clone(),
                        params.character.clone(),
                    );
                }

                storage::store(map);

                Game {
                    world: Some(world),
                    state,
                }
            }
        }

        impl Node for Game {
            fn ready(mut node: RefMut<Self>) where Self: Sized {
                let world = node.world.take();
                node.state.begin(world);
            }

            fn update(mut node: RefMut<Self>) where Self: Sized {
                node.state.update(get_frame_time());

                let mut camera = storage::get_mut::<GameCamera>();
                camera.update();
            }

            fn fixed_update(mut node: RefMut<Self>) where Self: Sized {
                node.state.fixed_update(get_frame_time(), 1.0);
            }

            fn draw(mut node: RefMut<Self>) where Self: Sized {
                let camera_position = {
                    let camera = storage::get::<GameCamera>();
                    camera.bounds.point() + (camera.bounds.size() / 2.0)
                };

                {
                    let map = storage::get::<Map>();
                    map.draw(None, camera_position);
                }

                node.state.draw()
            }
        }
    }
}