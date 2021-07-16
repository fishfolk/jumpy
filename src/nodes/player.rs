use macroquad::{
    audio::{self, play_sound_once},
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, HandleUntyped, Lens, RefMut},
        state_machine::{State, StateMachine},
    },
    prelude::*,
};
use macroquad_platformer::Actor;

use crate::{
    nodes::{Muscet, Sword},
    Resources,
};
use crate::nodes::Grenades;

mod ai;

pub mod capabilities {
    use crate::nodes::Player;
    use macroquad::experimental::{
        coroutines::Coroutine,
        scene::{Handle, HandleUntyped},
    };

    #[derive(Clone)]
    pub struct Gun {
        pub throw: fn(node: HandleUntyped, force: bool),
        pub shoot: fn(node: HandleUntyped, player: Handle<Player>) -> Coroutine,
    }
}

#[derive(Default, Debug, Clone)]
pub struct Input {
    jump: bool,
    was_jump: bool,
    throw: bool,
    was_throw: bool,
    fire: bool,
    was_fire: bool,
    left: bool,
    right: bool,
    down: bool,
}

pub struct PhysicsBody {
    pub pos: Vec2,
    pub speed: Vec2,
    pub facing: bool,
    pub angle: f32,
    pub collider: Option<Actor>,
    pub on_ground: bool,
    pub last_frame_on_ground: bool,
    pub have_gravity: bool,
}

impl PhysicsBody {
    pub const GRAVITY: f32 = 1800.0;

    pub fn facing_dir(&self) -> f32 {
        if self.facing {
            1.
        } else {
            -1.
        }
    }

    pub fn descent(&mut self) {
        let collision_world = &mut storage::get_mut::<Resources>().collision_world;

        if let Some(collider) = self.collider {
            collision_world.descent(collider);
        }
    }

    pub fn update(&mut self) {
        if let Some(collider) = self.collider {
            let collision_world = &mut storage::get_mut::<Resources>().collision_world;

            self.pos = collision_world.actor_pos(collider);
            self.last_frame_on_ground = self.on_ground;
            self.on_ground = collision_world.collide_check(collider, self.pos + vec2(0., 1.));
            if self.on_ground == false && self.have_gravity {
                self.speed.y += Self::GRAVITY * get_frame_time();
            }
            if !collision_world.move_h(collider, self.speed.x * get_frame_time()) {
                self.speed.x = 0.0;
            }
            if !collision_world.move_v(collider, self.speed.y * get_frame_time()) {
                self.speed.y = 0.0;
            }
            self.pos = collision_world.actor_pos(collider);
        }
    }

    pub fn update_throw(&mut self) {
        if self.on_ground == false {
            self.angle += self.speed.x.abs() * 0.00045 + self.speed.y.abs() * 0.00015;

            self.speed.y += Self::GRAVITY * get_frame_time();
        } else {
            self.angle %= std::f32::consts::PI * 2.;
            let goal = if self.angle <= std::f32::consts::PI {
                std::f32::consts::PI
            } else {
                std::f32::consts::PI * 2.
            };

            let rest = goal - self.angle;
            if rest.abs() >= 0.1 {
                self.angle += (rest * 0.1).max(0.1);
            }
        }

        self.speed.x *= 0.98;
        if self.speed.x.abs() <= 1. {
            self.speed.x = 0.0;
        }
    }
}

impl Player {
    pub fn drop_weapon(&mut self) {
        if let Some((weapon, _, gun)) = self.weapon.as_mut() {
            (gun.throw)(*weapon, false);
        }
        self.weapon = None;
    }

    pub fn pick_weapon(&mut self, weapon: (HandleUntyped, Lens<PhysicsBody>, capabilities::Gun)) {
        let resources = storage::get_mut::<Resources>();
        play_sound_once(resources.pickup_sound);

        self.drop_weapon();

        self.weapon = Some(weapon);
    }

    pub fn jump(&mut self) {
        let resources = storage::get::<Resources>();

        self.body.speed.y = -Self::JUMP_SPEED;
        audio::play_sound(
            resources.jump_sound,
            audio::PlaySoundParams {
                looped: false,
                volume: 0.6,
            },
        );
    }
}

pub struct Player {
    pub body: PhysicsBody,

    fish_sprite: AnimatedSprite,
    pub dead: bool,
    pub weapon: Option<(HandleUntyped, Lens<PhysicsBody>, capabilities::Gun)>,
    input: Input,

    deathmatch: bool,
    jump_grace_timer: f32,

    was_floating: bool,
    floating: bool,

    pub state_machine: StateMachine<RefMut<Player>>,
    pub controller_id: i32,

    ai_enabled: bool,
    ai: Option<ai::Ai>,

    pub camera_box: Rect,
    pub loses: i32,
}

impl Player {
    pub const ST_NORMAL: usize = 0;
    pub const ST_DEATH: usize = 1;
    pub const ST_SHOOT: usize = 2;
    pub const ST_AFTERMATCH: usize = 4;

    pub const JUMP_SPEED: f32 = 700.0;
    pub const RUN_SPEED: f32 = 250.0;
    pub const JUMP_GRACE_TIME: f32 = 0.15;
    pub const FLOAT_SPEED: f32 = 100.0;

    pub fn new(deathmatch: bool, controller_id: i32) -> Player {
        let spawner_pos = {
            let resources = storage::get_mut::<Resources>();
            let objects = &resources.tiled_map.layers["logic"].objects;
            let macroquad_tiled::Object {
                world_x, world_y, ..
            } = objects[rand::gen_range(0, objects.len()) as usize];
            vec2(world_x, world_y)
        };

        let mut state_machine = StateMachine::new();
        state_machine.add_state(Self::ST_NORMAL, State::new().update(Self::update_normal));
        state_machine.add_state(
            Self::ST_DEATH,
            State::new().coroutine(Self::death_coroutine),
        );
        state_machine.add_state(
            Self::ST_SHOOT,
            State::new()
                .update(Self::update_shoot)
                .coroutine(Self::shoot_coroutine),
        );
        state_machine.add_state(
            Self::ST_AFTERMATCH,
            State::new().update(Self::update_aftermatch),
        );

        let body = PhysicsBody {
            collider: {
                let mut resources = storage::get_mut::<Resources>();
                Some(resources.collision_world.add_actor(spawner_pos, 30, 54))
            },
            on_ground: false,
            angle: 0.0,
            speed: vec2(0., 0.),
            pos: spawner_pos,
            facing: true,
            last_frame_on_ground: false,
            have_gravity: true,
        };

        let fish_sprite = AnimatedSprite::new(
            76,
            66,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 7,
                    fps: 12,
                },
                Animation {
                    name: "run".to_string(),
                    row: 2,
                    frames: 6,
                    fps: 10,
                },
                Animation {
                    name: "death".to_string(),
                    row: 12,
                    frames: 3,
                    fps: 5,
                },
                Animation {
                    name: "death2".to_string(),
                    row: 14,
                    frames: 4,
                    fps: 8,
                },
                Animation {
                    name: "float".to_string(),
                    row: 6,
                    frames: 4,
                    fps: 8,
                },
            ],
            true,
        );

        Player {
            dead: false,
            weapon: None,
            input: Default::default(),

            body,
            fish_sprite,
            deathmatch,
            jump_grace_timer: 0.,
            floating: false,
            was_floating: false,
            state_machine,
            controller_id,
            ai_enabled: false,
            ai: Some(ai::Ai::new()),
            camera_box: Rect::new(spawner_pos.x - 30., spawner_pos.y - 150., 100., 210.),
            loses: 0,
        }
    }

    pub fn kill(&mut self, direction: bool) {
        self.body.facing = direction;
        self.state_machine.set_state(Self::ST_DEATH);
    }

    fn death_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        let handle = node.handle();

        let map_bottom = {
            let resources = storage::get::<Resources>();

            resources.tiled_map.raw_tiled_map.tileheight * resources.tiled_map.raw_tiled_map.height
        } as f32;

        let coroutine = async move {
            {
                let mut node = scene::get_node(handle);
                node.body.speed.x = -300. * node.body.facing_dir();
                node.body.speed.y = -150.;
                node.body.have_gravity = true;

                node.dead = true;
                node.fish_sprite.set_animation(2);
            }

            if {
                let node = scene::get_node(handle);
                node.body.pos.y < map_bottom
            } {
                // give some take for a dead fish to take off the ground
                wait_seconds(0.1).await;

                // wait until it lands (or fall down the map)
                while {
                    let node = scene::get_node(handle);

                    (node.body.on_ground || node.body.pos.y > map_bottom) == false
                } {
                    next_frame().await;
                }

                {
                    let mut node = scene::get_node(handle);
                    node.fish_sprite.set_animation(3);
                    node.body.speed = vec2(0., 0.);

                    node.loses += 1;
                }

                wait_seconds(0.5).await;
            } else {
                let mut node = scene::get_node(handle);
                node.loses += 1;
            }

            {
                let mut resources = storage::get_mut::<Resources>();
                let mut node = scene::get_node(handle);
                let pos = node.body.pos;

                node.fish_sprite.playing = false;
                node.body.speed = vec2(0., 0.);
                resources.explosion_fxses.spawn(pos + vec2(15., 33.));
            }

            wait_seconds(0.5).await;

            let mut this = scene::get_node(handle);

            this.body.pos = {
                let resources = storage::get_mut::<Resources>();
                let objects = &resources.tiled_map.layers["logic"].objects;
                let macroquad_tiled::Object {
                    world_x, world_y, ..
                } = objects[rand::gen_range(0, objects.len()) as usize];

                vec2(world_x, world_y)
            };
            this.fish_sprite.playing = true;
            this.drop_weapon();

            // in deathmatch we can just get back to normal after death
            if this.deathmatch {
                let mut resources = storage::get_mut::<Resources>();

                this.state_machine.set_state(Self::ST_NORMAL);
                this.dead = false;
                resources
                    .collision_world
                    .set_actor_position(this.body.collider.unwrap(), this.body.pos);
            }
        };

        start_coroutine(coroutine)
    }

    fn shoot_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        if let Some((weapon, _, gun)) = node.weapon.as_mut() {
            (gun.shoot)(*weapon, node.handle())
        } else {
            let handle = node.handle();

            start_coroutine(async move {
                let player = &mut *scene::get_node(handle);
                player.state_machine.set_state(Player::ST_NORMAL);
            })
        }
    }

    fn update_shoot(node: &mut RefMut<Player>, _dt: f32) {
        node.body.speed.x *= 0.9;
    }

    fn update_aftermatch(node: &mut RefMut<Player>, _dt: f32) {
        node.body.speed.x = 0.0;
    }

    fn update_normal(node: &mut RefMut<Player>, _dt: f32) {
        // self destruct, for debugging only
        if is_key_pressed(KeyCode::Y) {
            node.kill(true);
        }
        if is_key_pressed(KeyCode::U) {
            node.kill(false);
        }

        let node = &mut **node;

        if node.input.right {
            node.body.speed.x = Self::RUN_SPEED;
            node.body.facing = true;
        } else if node.input.left {
            node.body.speed.x = -Self::RUN_SPEED;
            node.body.facing = false;
        } else {
            node.body.speed.x = 0.;
        }

        // shanke on fall
        if node.body.on_ground && node.body.last_frame_on_ground == false {
            // scene::find_node_by_type::<crate::nodes::Camera>()
            //     .unwrap()
            //     .shake();
        }

        if node.floating {
            node.fish_sprite.set_animation(4);
        } else {
            if node.input.right || node.input.left {
                node.fish_sprite.set_animation(1);
            } else {
                node.fish_sprite.set_animation(0);
            }
        }

        if node.input.down {
            node.body.descent();
        }

        // if in jump and want to jump again
        if node.body.on_ground == false && node.input.jump && node.jump_grace_timer <= 0.0 {
            if node.was_floating == false {
                node.floating = true;
                node.was_floating = true;
                node.body.have_gravity = false;
            }
        }
        // jump button released, stop to float
        if node.input.was_jump == false {
            node.floating = false;
        }
        if node.body.on_ground {
            node.was_floating = false;
            node.floating = false;
        }
        if node.floating {
            node.body.speed.y = Self::FLOAT_SPEED;
        } else {
            node.body.have_gravity = true;
        }

        if node.input.jump {
            if node.jump_grace_timer > 0. {
                node.jump_grace_timer = 0.0;
                node.jump();
            }
        }

        if node.input.throw {
            if let Some((weapon, _, gun)) = node.weapon.as_mut() {
                (gun.throw)(*weapon, node.input.down == false);
                node.weapon = None;

                // when the floating fish is throwing a weapon and keeps
                // floating it looks less cool than if its stop floating and
                // falls, but idk
                node.floating = false;
            } else {
                let mut picked = false;

                for mut sword in scene::find_nodes_by_type::<Sword>() {
                    if picked {
                        break;
                    }
                    if sword.thrown && sword.body.pos.distance(node.body.pos) < 80. {
                        picked = true;
                        sword.body.angle = std::f32::consts::PI / 4. + 0.3;
                        sword.thrown = false;
                        let sword = (
                            sword.handle().untyped(),
                            sword.handle().lens(|node| &mut node.body),
                            Sword::gun_capabilities(),
                        );
                        node.pick_weapon(sword);
                    }
                }
                for mut muscet in scene::find_nodes_by_type::<Muscet>() {
                    if picked {
                        break;
                    }
                    if muscet.thrown && muscet.body.pos.distance(node.body.pos) < 80. {
                        picked = true;
                        muscet.body.angle = 0.;
                        muscet.thrown = false;
                        muscet.bullets = 3;

                        let muscet = (
                            muscet.handle().untyped(),
                            muscet.handle().lens(|node| &mut node.body),
                            Muscet::gun_capabilities(),
                        );
                        node.pick_weapon(muscet);
                    }
                }
                for mut grenades in scene::find_nodes_by_type::<Grenades>() {
                    if picked {
                        break;
                    }
                    if grenades.thrown && grenades.body.pos.distance(node.body.pos) < 80. {
                        picked = true;
                        grenades.body.angle = 0.;
                        grenades.thrown = false;
                        grenades.amount = 3;

                        let grenades = (
                            grenades.handle().untyped(),
                            grenades.handle().lens(|node| &mut node.body),
                            Grenades::gun_capabilities(),
                        );
                        node.pick_weapon(grenades);
                    }
                }
            }
        }

        if node.input.fire {
            if node.weapon.is_some() {
                node.state_machine.set_state(Self::ST_SHOOT);
                node.floating = false;
            }
        }
    }
}

impl scene::Node for Player {
    fn draw(node: RefMut<Self>) {
        //     let sword_hit_box = if node.fish.facing {
        //         Rect::new(node.pos().x + 35., node.pos().y - 5., 40., 60.)
        //     } else {
        //         Rect::new(node.pos().x - 50., node.pos().y - 5., 40., 60.)
        //     };
        //     draw_rectangle(
        //         sword_hit_box.x,
        //         sword_hit_box.y,
        //         sword_hit_box.w,
        //         sword_hit_box.h,
        //         RED,
        //     );

        // draw_rectangle_lines(
        //     node.camera_box.x,
        //     node.camera_box.y,
        //     node.camera_box.w,
        //     node.camera_box.h,
        //     5.,
        //     RED,
        // );

        //draw_rectangle_lines(fish_box.x, fish_box.y, fish_box.w, fish_box.h, 5., BLUE);

        let resources = storage::get::<Resources>();

        draw_texture_ex(
            if node.controller_id == 0 {
                resources.whale
            } else {
                resources.whale_red
            },
            node.body.pos.x - 25.,
            node.body.pos.y - 10.,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.fish_sprite.frame().source_rect),
                dest_size: Some(node.fish_sprite.frame().dest_size),
                flip_x: !node.body.facing,
                ..Default::default()
            },
        );
    }

    fn update(mut node: RefMut<Self>) {
        if is_key_pressed(KeyCode::B) && node.controller_id == 0 {
            node.ai_enabled ^= true;
        }

        if is_key_pressed(KeyCode::N) && node.controller_id == 1 {
            node.ai_enabled ^= true;
        }
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.fish_sprite.update();

        let map_bottom = {
            let resources = storage::get::<Resources>();

            resources.tiled_map.raw_tiled_map.tileheight * resources.tiled_map.raw_tiled_map.height
        } as f32;

        if node.body.pos.y > map_bottom {
            node.kill(false);
        }

        {
            let node = &mut *node;
            if let Some((_, weapon_body, _)) = node.weapon.as_mut() {
                let body = weapon_body.get().unwrap();
                body.pos = node.body.pos;
                body.facing = node.body.facing;
            }
        }

        if false {
            let controller = storage::get_mut::<gamepad_rs::ControllerContext>();

            let status = controller.state(node.controller_id as _).status;

            if status == gamepad_rs::ControllerStatus::Connected {
                let state = controller.state(node.controller_id as _);

                let x = state.analog_state[0];
                let y = state.analog_state[1];

                node.input.left = x < -0.5;
                node.input.right = x > 0.5;

                if cfg!(target_os = "macos") {
                    node.input.down = y < -0.5;
                } else {
                    node.input.down = y > 0.5;
                };

                let jump_btn: usize = if cfg!(target_os = "macos") {
                    1
                } else {
                    if node.controller_id == 1 {
                        2
                    } else {
                        1
                    }
                };
                let fire_btn: usize = if cfg!(target_os = "macos") {
                    0
                } else {
                    if node.controller_id == 1 {
                        1
                    } else {
                        0
                    }
                };
                let throw_btn: usize = if cfg!(target_os = "macos") {
                    3
                } else {
                    if node.controller_id == 1 {
                        3
                    } else {
                        4
                    }
                };

                if state.digital_state[jump_btn] && node.input.was_jump == false {
                    node.input.jump = true;
                } else {
                    node.input.jump = false;
                }
                node.input.was_jump = state.digital_state[jump_btn];

                if state.digital_state[fire_btn] && node.input.was_fire == false {
                    node.input.fire = true;
                } else {
                    node.input.fire = false;
                }
                node.input.was_fire = state.digital_state[fire_btn];

                if state.digital_state[throw_btn] && node.input.was_throw == false {
                    node.input.throw = true;
                } else {
                    node.input.throw = false;
                }
                node.input.was_throw = state.digital_state[throw_btn];
            }
        }

        if node.ai_enabled {
            let mut ai = node.ai.take().unwrap();
            let input = ai.update(&mut *node);
            node.input = input;
            node.ai = Some(ai);
        }

        if is_key_pressed(KeyCode::Q) {
            scene::find_node_by_type::<crate::nodes::Camera>()
                .unwrap()
                .shake();
        }

        #[cfg(not(target_os = "macos"))]
        if node.ai_enabled == false && node.controller_id == 1 {
            let jump = is_key_down(KeyCode::Space) || is_key_down(KeyCode::W);
            node.input.jump = jump && node.input.was_jump == false;
            node.input.was_jump = jump;

            let throw = is_key_down(KeyCode::R) || is_key_down(KeyCode::K);
            node.input.throw = throw && node.input.was_throw == false;
            node.input.was_throw = throw;

            node.input.fire = is_key_down(KeyCode::LeftControl)
                || is_key_down(KeyCode::F)
                || is_key_down(KeyCode::L);
            node.input.left = is_key_down(KeyCode::A);
            node.input.right = is_key_down(KeyCode::D);
            node.input.down = is_key_down(KeyCode::S);
        }

        #[cfg(not(target_os = "macos"))]
        if node.ai_enabled == false && node.controller_id == 0 {
            let jump = is_key_down(KeyCode::Up);
            node.input.jump = jump && node.input.was_jump == false;
            node.input.was_jump = jump;
            node.input.left = is_key_down(KeyCode::Left);
            node.input.right = is_key_down(KeyCode::Right);
            node.input.down = is_key_down(KeyCode::Down);
        }

        {
            let node = &mut *node;

            if node.body.on_ground && node.input.was_jump == false {
                node.jump_grace_timer = Self::JUMP_GRACE_TIME;
            } else if node.jump_grace_timer > 0. {
                node.jump_grace_timer -= get_frame_time();
            }

            node.body.update();
        }

        // update camera bound box
        {
            let fish_box = Rect::new(node.body.pos.x, node.body.pos.y, 32., 60.);

            if fish_box.x < node.camera_box.x {
                node.camera_box.x = fish_box.x;
            }
            if fish_box.x + fish_box.w > node.camera_box.x + node.camera_box.w {
                node.camera_box.x = fish_box.x + fish_box.w - node.camera_box.w;
            }
            if fish_box.y < node.camera_box.y {
                node.camera_box.y = fish_box.y;
            }
            if fish_box.y + fish_box.h > node.camera_box.y + node.camera_box.h {
                node.camera_box.y = fish_box.y + fish_box.h - node.camera_box.h;
            }
        }
        StateMachine::update_detached(&mut node, |node| &mut node.state_machine);
    }
}
