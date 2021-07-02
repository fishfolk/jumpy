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
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ItemType {
    Gun = 1,
    Sword = 2,
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
            self.on_ground = collision_world.collide_check(collider, self.pos + vec2(0., 1.));
            if self.on_ground == false {
                self.speed.y += Self::GRAVITY * get_frame_time();
            }
            collision_world.move_h(collider, self.speed.x * get_frame_time());

            if !collision_world.move_v(collider, self.speed.y * get_frame_time()) {
                self.speed.y = 0.0;
            }
        }
    }

    pub fn update_throw(&mut self) {
        if self.on_ground == false {
            self.angle += self.speed.x.abs() * 0.00015 + self.speed.y.abs() * 0.00005;

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

    pub fn pick_weapon(&mut self, item_type: ItemType) {
        let resources = storage::get_mut::<Resources>();
        play_sound_once(resources.pickup_sound);

        self.drop_weapon();

        match item_type {
            ItemType::Gun => {
                let node = scene::add_node(Muscet::new(self.body.facing, self.body.pos));
                self.weapon = Some((
                    node.untyped(),
                    node.lens(|node| &mut node.body),
                    Muscet::gun_capabilities(),
                ));
            }
            ItemType::Sword => {
                let node = scene::add_node(Sword::new(self.body.facing, self.body.pos));
                self.weapon = Some((
                    node.untyped(),
                    node.lens(|node| &mut node.body),
                    Sword::gun_capabilities(),
                ));
            }
        }
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

    pub fn draw(&mut self, id: i32) {
        let resources = storage::get::<Resources>();

        draw_texture_ex(
            if id == 0 {
                resources.whale
            } else {
                resources.whale_red
            },
            self.body.pos.x - 25.,
            self.body.pos.y - 10.,
            color::WHITE,
            DrawTextureParams {
                source: Some(self.fish_sprite.frame().source_rect),
                dest_size: Some(self.fish_sprite.frame().dest_size),
                flip_x: !self.body.facing,
                ..Default::default()
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
    pub state_machine: StateMachine<RefMut<Player>>,
    pub controller_id: i32,
}

impl Player {
    pub const ST_NORMAL: usize = 0;
    pub const ST_DEATH: usize = 1;
    pub const ST_SHOOT: usize = 2;
    pub const ST_AFTERMATCH: usize = 4;

    pub const JUMP_SPEED: f32 = 700.0;
    pub const RUN_SPEED: f32 = 250.0;
    pub const JUMP_GRACE_TIME: f32 = 0.15;
    pub const MAP_BOTTOM: f32 = 630.0;

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
            state_machine,
            controller_id,
        }
    }

    pub fn kill(&mut self, direction: bool) {
        self.body.facing = direction;
        self.state_machine.set_state(Self::ST_DEATH);
    }

    fn death_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        let handle = node.handle();
        let coroutine = async move {
            {
                let mut node = scene::get_node(handle);
                node.body.speed.x = -300. * node.body.facing_dir();
                node.body.speed.y = -150.;

                node.dead = true;
                node.fish_sprite.set_animation(2);
            }
            // give some take for a dead fish to take off the ground
            wait_seconds(0.1).await;

            // wait until it lands (or fall down the map)
            while {
                let node = scene::get_node(handle);

                (node.body.on_ground || node.body.pos.y > Self::MAP_BOTTOM) == false
            } {
                next_frame().await;
            }

            {
                let mut node = scene::get_node(handle);
                node.fish_sprite.set_animation(3);
                node.body.speed = vec2(0., 0.);
            }

            wait_seconds(0.5).await;

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
            node.fish_sprite.set_animation(1);
            node.body.speed.x = Self::RUN_SPEED;
            node.body.facing = true;
        } else if node.input.left {
            node.fish_sprite.set_animation(1);
            node.body.speed.x = -Self::RUN_SPEED;
            node.body.facing = false;
        } else {
            node.fish_sprite.set_animation(0);
            node.body.speed.x = 0.;
        }

        if node.input.jump {
            if node.jump_grace_timer > 0. {
                node.jump_grace_timer = 0.0;
                node.jump();
            }
        }

        if node.input.down {
            node.body.descent();
        }

        if node.input.throw {
            if let Some((weapon, _, gun)) = node.weapon.as_mut() {
                (gun.throw)(*weapon, node.input.down == false);
                node.weapon = None;
            } else {
                let mut picked = false;

                for sword in scene::find_nodes_by_type::<Sword>() {
                    if picked {
                        break;
                    }
                    if sword.thrown && sword.body.pos.distance(node.body.pos) < 80. {
                        picked = true;
                        sword.delete();
                        node.pick_weapon(ItemType::Sword);
                    }
                }
                for muscet in scene::find_nodes_by_type::<Muscet>() {
                    if picked {
                        break;
                    }
                    if muscet.thrown && muscet.body.pos.distance(node.body.pos) < 80. {
                        picked = true;
                        muscet.delete();
                        node.pick_weapon(ItemType::Gun);
                    }
                }
            }
        }

        if node.input.fire {
            if node.weapon.is_some() {
                node.state_machine.set_state(Self::ST_SHOOT);
            }
        }
    }
}

impl scene::Node for Player {
    fn draw(mut node: RefMut<Self>) {
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
        let id = node.controller_id;
        node.draw(id);
    }

    fn update(mut node: RefMut<Self>) {
        let game_started = true;

        node.fish_sprite.update();

        if node.body.pos.y > Self::MAP_BOTTOM {
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

        if game_started {
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

                const JUMP_BTN: usize = if cfg!(target_os = "macos") { 1 } else { 2 };
                const FIRE_BTN: usize = if cfg!(target_os = "macos") { 0 } else { 1 };
                const THROW_BTN: usize = if cfg!(target_os = "macos") { 3 } else { 3 };

                if state.digital_state[JUMP_BTN] && node.input.was_jump == false {
                    node.input.jump = true;
                } else {
                    node.input.jump = false;
                }
                node.input.was_jump = state.digital_state[JUMP_BTN];

                if state.digital_state[FIRE_BTN] && node.input.was_fire == false {
                    node.input.fire = true;
                } else {
                    node.input.fire = false;
                }
                node.input.was_fire = state.digital_state[FIRE_BTN];

                if state.digital_state[THROW_BTN] && node.input.was_throw == false {
                    node.input.throw = true;
                } else {
                    node.input.throw = false;
                }
                node.input.was_throw = state.digital_state[THROW_BTN];
            }
        }

        #[cfg(not(target_os = "macos"))]
        if game_started && node.controller_id == 0 {
            node.input.jump = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::W);
            node.input.throw = is_key_pressed(KeyCode::R) || is_key_pressed(KeyCode::K);

            node.input.fire = is_key_pressed(KeyCode::LeftControl)
                || is_key_pressed(KeyCode::F)
                || is_key_pressed(KeyCode::L);
            node.input.left = is_key_down(KeyCode::A);
            node.input.right = is_key_down(KeyCode::D);
            node.input.down = is_key_down(KeyCode::S);
        }

        // #[cfg(not(target_os = "macos"))]
        // if game_started && node.controller_id == 1 {
        //     node.input.jump = is_key_pressed(KeyCode::Up);
        //     node.input.throw = is_key_pressed(KeyCode::K);
        //     node.input.fire = is_key_pressed(KeyCode::L);
        //     node.input.left = is_key_down(KeyCode::Left);
        //     node.input.right = is_key_down(KeyCode::Right);
        //     node.input.down = is_key_down(KeyCode::Down);
        // }

        {
            let node = &mut *node;

            if node.body.on_ground {
                node.jump_grace_timer = Self::JUMP_GRACE_TIME;
            } else if node.jump_grace_timer > 0. {
                node.jump_grace_timer -= get_frame_time();
            }

            node.body.update();
        }
        StateMachine::update_detached(&mut node, |node| &mut node.state_machine);
    }
}
