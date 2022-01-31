mod camera;
mod input;
mod music;

pub use camera::GameCamera;

use fishsticks::{Button, GamepadContext};

use macroquad::experimental::collections::storage;
use macroquad::experimental::scene::{Node, RefMut};
use macroquad::prelude::*;
use macroquad::ui::root_ui;

use hecs::{Entity, World};

use crate::debug;
use crate::ecs::Scheduler;
use crate::gui::{self, GAME_MENU_RESULT_MAIN_MENU, GAME_MENU_RESULT_QUIT};
use crate::physics::{debug_draw_physics_bodies, fixed_update_physics_bodies};
use crate::player::{
    draw_weapons_hud, spawn_player, update_player_animations, update_player_camera_box,
    update_player_controllers, update_player_events, update_player_inventory,
    update_player_passive_effects, update_player_states, PlayerParams,
};
use crate::Result;
use crate::{
    create_collision_world, debug_draw_drawables, debug_draw_rigid_bodies, draw_drawables,
    exit_to_main_menu, fixed_update_rigid_bodies, is_gamepad_btn_pressed, quit_to_desktop,
    update_animated_sprites, Map, MapLayerKind, MapObjectKind, Resources,
};

pub use input::{collect_local_input, GameInput, GameInputScheme};

use crate::effects::active::debug_draw_active_effects;
use crate::effects::active::projectiles::fixed_update_projectiles;
use crate::effects::active::triggered::fixed_update_triggered_effects;
use crate::items::spawn_item;
use crate::map::{fixed_update_sproingers, spawn_decoration, spawn_sproinger};
use crate::network::{
    fixed_update_network_client, fixed_update_network_host, update_network_client,
    update_network_host, AccountId, NetworkClient, NetworkHost,
};
use crate::particles::{draw_particles, update_particle_emitters};
pub use music::{start_music, stop_music};

#[derive(Debug, Copy, Clone)]
pub enum GameMode {
    Local,
    NetworkHost {
        port: Option<u16>,
    },
    NetworkClient {
        port: Option<u16>,
        host_id: AccountId,
    },
}

pub struct Game {
    world: World,
    #[allow(dead_code)]
    players: Vec<Entity>,
    updates: Scheduler,
    fixed_updates: Scheduler,
    draws: Scheduler,
    #[cfg(debug_assertions)]
    debug_draws: Scheduler,
}

impl Game {
    pub fn new(mode: GameMode, map: Map, player_params: &[PlayerParams]) -> Result<Game> {
        let mut world = World::default();

        {
            let camera = GameCamera::new(map.get_size());
            storage::store(camera);

            let collision_world = create_collision_world(&map);
            storage::store(collision_world);
        }

        spawn_map_objects(&mut world, &map).unwrap();

        let mut players = Vec::new();
        for PlayerParams {
            index,
            controller,
            character,
        } in player_params.iter().cloned()
        {
            let position = map.get_random_spawn_point();
            let player = spawn_player(&mut world, index, position, controller, character);

            players.push(player);
        }

        storage::store(map);

        let mut updates_builder = Scheduler::builder();

        let mut fixed_updates_builder = Scheduler::builder();

        match mode {
            GameMode::NetworkClient { port, host_id } => {
                let e = world.spawn(());
                world.insert_one(e, NetworkClient::new(port, host_id))?;

                updates_builder.add_system(update_network_client);

                fixed_updates_builder.add_system(fixed_update_network_client);
            }
            GameMode::NetworkHost { port } => {
                let e = world.spawn(());
                world.insert_one(e, NetworkHost::new(port))?;

                updates_builder.add_system(update_network_host);

                fixed_updates_builder.add_system(fixed_update_network_host);
            }
            _ => {}
        }

        updates_builder
            .add_system(update_player_controllers)
            .add_system(update_player_camera_box);

        if matches!(mode, GameMode::Local | GameMode::NetworkHost { .. }) {
            updates_builder
                .add_system(update_player_states)
                .add_system(update_player_inventory)
                .add_system(update_player_passive_effects)
                .add_system(update_player_events);

            fixed_updates_builder
                .add_system(fixed_update_physics_bodies)
                .add_system(fixed_update_rigid_bodies)
                .add_system(fixed_update_projectiles)
                .add_system(fixed_update_triggered_effects)
                .add_system(fixed_update_sproingers);
        }

        let updates = updates_builder
            .with_system(update_player_animations)
            .with_system(update_animated_sprites)
            .with_system(update_particle_emitters)
            .build();

        let fixed_updates = fixed_updates_builder.build();

        let draws = Scheduler::builder()
            .with_thread_local(draw_drawables)
            .with_thread_local(draw_weapons_hud)
            .with_thread_local(draw_particles)
            .build();

        #[cfg(debug_assertions)]
        let debug_draws = Scheduler::builder()
            .with_thread_local(debug_draw_drawables)
            .with_thread_local(debug_draw_physics_bodies)
            .with_thread_local(debug_draw_rigid_bodies)
            .with_thread_local(debug_draw_active_effects)
            .build();

        let res = Game {
            world,
            players,
            updates,
            fixed_updates,
            draws,
            #[cfg(debug_assertions)]
            debug_draws,
        };

        Ok(res)
    }

    fn on_update(&mut self) {
        self.updates.execute(&mut self.world);

        #[cfg(debug_assertions)]
        if is_key_pressed(macroquad::prelude::KeyCode::U) {
            crate::debug::toggle_debug_draw();
        }

        {
            let gamepad_context = storage::get::<GamepadContext>();
            if is_key_pressed(macroquad::prelude::KeyCode::Escape)
                || is_gamepad_btn_pressed(Some(&gamepad_context), Button::Start)
            {
                gui::toggle_game_menu();
            }
        }
    }

    fn on_fixed_update(&mut self) {
        self.fixed_updates.execute(&mut self.world);
    }

    fn on_draw(&mut self) {
        let mut camera = storage::get_mut::<GameCamera>();
        camera.update();

        {
            let map = storage::get::<Map>();
            map.draw(None, true);
        }

        self.draws.execute(&mut self.world);

        #[cfg(debug_assertions)]
        if debug::is_debug_draw_enabled() {
            self.debug_draws.execute(&mut self.world);
        }

        if gui::is_game_menu_open() {
            if let Some(res) = gui::draw_game_menu(&mut *root_ui()) {
                match res.into_usize() {
                    GAME_MENU_RESULT_MAIN_MENU => exit_to_main_menu(),
                    GAME_MENU_RESULT_QUIT => quit_to_desktop(),
                    _ => {}
                }
            }
        }
    }
}

impl Node for Game {
    fn update(mut node: RefMut<Self>) {
        node.on_update();
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.on_fixed_update();
    }

    fn draw(mut node: RefMut<Self>) {
        node.on_draw();
    }
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
