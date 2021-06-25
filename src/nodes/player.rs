use macroquad::{
    audio::{self, play_sound_once},
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, Handle, RefMut},
        state_machine::{State, StateMachine},
    },
    prelude::*,
};
use macroquad_platformer::Actor;

use crate::{
    consts,
    nodes::{Muscet, Sword},
    Resources,
};

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
}

pub struct Fish {
    fish_sprite: AnimatedSprite,
    pub collider: Actor,
    pub pos: Vec2,
    pub speed: Vec2,
    pub on_ground: bool,
    pub dead: bool,
    pub facing: bool,
    pub sword: Option<Handle<Sword>>,
    pub muscet: Option<Handle<Muscet>>,
    input: Input,
}

impl Fish {
    pub fn new(spawner_pos: Vec2) -> Fish {
        let mut resources = storage::get_mut::<Resources>();

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

        Fish {
            fish_sprite,
            collider: resources.collision_world.add_actor(spawner_pos, 30, 54),
            on_ground: false,
            dead: false,
            pos: spawner_pos,
            speed: vec2(0., 0.),
            facing: true,
            sword: None,
            muscet: None,
            input: Default::default(),
        }
    }

    pub fn disarm(&mut self) {
        if let Some(muscet) = self.muscet {
            if let Some(muscet) = scene::try_get_node(muscet) {
                muscet.delete();
            }
            self.muscet = None;
        }

        if let Some(sword) = self.sword {
            if let Some(sword) = scene::try_get_node(sword) {
                sword.delete();
            }
            self.sword = None;
        }
    }

    pub fn pick_weapon(&mut self, item_type: ItemType) {
        let resources = storage::get_mut::<Resources>();
        play_sound_once(resources.pickup_sound);

        self.disarm();

        match item_type {
            ItemType::Gun => {
                self.muscet = Some(scene::add_node(Muscet::new(self.facing, self.pos)));
            }
            ItemType::Sword => {
                self.sword = Some(scene::add_node(Sword::new(self.facing, self.pos)));
            }
        }
    }

    pub fn facing_dir(&self) -> f32 {
        if self.facing {
            1.
        } else {
            -1.
        }
    }

    pub fn jump(&mut self) {
        let resources = storage::get::<Resources>();

        self.speed.y = -consts::JUMP_SPEED;
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
            self.pos.x - 25.,
            self.pos.y - 10.,
            color::WHITE,
            DrawTextureParams {
                source: Some(self.fish_sprite.frame().source_rect),
                dest_size: Some(self.fish_sprite.frame().dest_size),
                flip_x: !self.facing,
                ..Default::default()
            },
        );
    }
}

pub struct Player {
    pub fish: Fish,

    deathmatch: bool,
    jump_grace_timer: f32,
    pub state_machine: StateMachine<RefMut<Player>>,
    pub controller_id: i32,
}

impl Player {
    pub const ST_NORMAL: usize = 0;
    pub const ST_DEATH: usize = 1;
    pub const ST_SHOOT: usize = 2;
    pub const ST_SWORD_SHOOT: usize = 3;
    pub const ST_AFTERMATCH: usize = 4;

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
            Self::ST_SWORD_SHOOT,
            State::new()
                .update(Self::update_sword_shoot)
                .coroutine(Self::sword_shoot_coroutine),
        );
        state_machine.add_state(
            Self::ST_AFTERMATCH,
            State::new().update(Self::update_aftermatch),
        );

        Player {
            fish: Fish::new(spawner_pos),
            deathmatch,
            jump_grace_timer: 0.,
            state_machine,
            controller_id,
        }
    }

    pub fn pos(&self) -> Vec2 {
        self.fish.pos
    }

    pub fn is_dead(&self) -> bool {
        self.fish.dead
    }

    pub fn kill(&mut self, direction: bool) {
        self.fish.facing = direction;
        self.state_machine.set_state(Self::ST_DEATH);
    }

    fn death_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        let handle = node.handle();
        let coroutine = async move {
            {
                let mut node = scene::get_node(handle);
                node.fish.speed.x = -300. * node.fish.facing_dir();
                node.fish.speed.y = -150.;

                node.fish.dead = true;
                node.fish.fish_sprite.set_animation(2);
            }
            // give some take for a dead fish to take off the ground
            wait_seconds(0.1).await;

            // wait until it lands
            while scene::get_node(handle).fish.on_ground == false {
                next_frame().await;
            }

            {
                let mut node = scene::get_node(handle);
                node.fish.fish_sprite.set_animation(3);
                node.fish.speed = vec2(0., 0.);
            }

            wait_seconds(0.5).await;

            {
                let mut resources = storage::get_mut::<Resources>();
                let mut node = scene::get_node(handle);
                let pos = node.fish.pos;

                node.fish.fish_sprite.playing = false;
                node.fish.speed = vec2(0., 0.);
                resources.explosion_fxses.spawn(pos + vec2(15., 33.));
            }

            wait_seconds(0.5).await;

            let mut resources = storage::get_mut::<Resources>();
            let mut this = scene::get_node(handle);

            this.fish.pos = {
                let objects = &resources.tiled_map.layers["logic"].objects;
                let macroquad_tiled::Object {
                    world_x, world_y, ..
                } = objects[rand::gen_range(0, objects.len()) as usize];

                vec2(world_x, world_y)
            };
            this.fish.fish_sprite.playing = true;
            this.fish.disarm();

            // in deathmatch we can just get back to normal after death
            if this.deathmatch {
                this.state_machine.set_state(Self::ST_NORMAL);
                this.fish.dead = false;
                resources
                    .collision_world
                    .set_actor_position(this.fish.collider, this.fish.pos);
            }
        };

        start_coroutine(coroutine)
    }

    fn shoot_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        if let Some(muscet) = node.fish.muscet {
            Muscet::shot(muscet, node.handle())
        } else {
            let handle = node.handle();

            start_coroutine(async move {
                println!("thats weird");
                let player = &mut *scene::get_node(handle);
                player.state_machine.set_state(Player::ST_NORMAL);
            })
        }
    }

    fn update_shoot(node: &mut RefMut<Player>, _dt: f32) {
        node.fish.speed.x *= 0.9;
    }

    fn sword_shoot_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        if let Some(sword) = node.fish.sword {
            Sword::shot(sword, node.handle())
        } else {
            let handle = node.handle();

            start_coroutine(async move {
                println!("thats weird");
                let player = &mut *scene::get_node(handle);
                player.state_machine.set_state(Player::ST_NORMAL);
            })
        }
    }

    fn update_sword_shoot(node: &mut RefMut<Player>, _dt: f32) {
        node.fish.speed.x *= 0.9;
    }

    fn update_aftermatch(node: &mut RefMut<Player>, _dt: f32) {
        node.fish.speed.x = 0.0;
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
        let fish = &mut node.fish;

        if fish.input.right {
            fish.fish_sprite.set_animation(1);
            fish.speed.x = consts::RUN_SPEED;
            fish.facing = true;
        } else if fish.input.left {
            fish.fish_sprite.set_animation(1);
            fish.speed.x = -consts::RUN_SPEED;
            fish.facing = false;
        } else {
            fish.fish_sprite.set_animation(0);
            fish.speed.x = 0.;
        }

        if fish.input.jump {
            if node.jump_grace_timer > 0. {
                node.jump_grace_timer = 0.0;
                fish.jump();
            }
        }

        if fish.input.throw {
            if let Some(sword) = fish.sword {
                if let Some(mut sword) = scene::try_get_node(sword) {
                    sword.throw(true);
                }
                fish.sword = None;
            } else if let Some(muscet) = fish.muscet {
                if let Some(mut muscet) = scene::try_get_node(muscet) {
                    muscet.throw(true);
                }
                fish.muscet = None;
            } else {
                let mut picked = false;

                for sword in scene::find_nodes_by_type::<Sword>() {
                    if picked {
                        break;
                    }
                    if sword.thrown && sword.pos.distance(fish.pos) < 80. {
                        picked = true;
                        sword.delete();
                        fish.pick_weapon(ItemType::Sword);
                    }
                }
                for muscet in scene::find_nodes_by_type::<Muscet>() {
                    if picked {
                        break;
                    }
                    if muscet.thrown && muscet.pos.distance(fish.pos) < 80. {
                        picked = true;
                        muscet.delete();
                        fish.pick_weapon(ItemType::Gun);
                    }
                }
            }
        }

        if fish.input.fire {
            if node.fish.muscet.is_some()
                && scene::try_get_node(node.fish.muscet.unwrap())
                    .map_or(false, |muscet| muscet.bullets > 0)
            {
                node.state_machine.set_state(Self::ST_SHOOT);
            }
            if node.fish.sword.is_some() {
                node.state_machine.set_state(Self::ST_SWORD_SHOOT);
            }
        }
    }

    fn draw_hud(&self) {
        if self.is_dead() {
            return;
        }
        if let Some(muscet) = self
            .fish
            .muscet
            .and_then(|muscet| scene::try_get_node(muscet))
        {
            let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
            let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);
            for i in 0..3 {
                let x = self.fish.pos.x + 15.0 * i as f32;

                if i >= muscet.bullets {
                    draw_circle_lines(x, self.fish.pos.y - 4.0, 4.0, 2., empty_color);
                } else {
                    draw_circle(x, self.fish.pos.y - 4.0, 4.0, full_color);
                };
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
        node.fish.draw(id);

        node.draw_hud();
    }

    fn update(mut node: RefMut<Self>) {
        let game_started = true;

        node.fish.fish_sprite.update();

        if let Some(sword) = node.fish.sword {
            let mut sword = scene::get_node(sword);
            sword.pos = node.fish.pos;
            sword.facing = node.fish.facing;
        }

        if let Some(mut muscet) = node
            .fish
            .muscet
            .and_then(|muscet| scene::try_get_node(muscet))
        {
            muscet.pos = node.fish.pos;
            muscet.facing = node.fish.facing;
        }

        #[cfg(target_os = "macos")]
        if game_started {
            let mut controller = storage::get_mut::<gamepad_rs::ControllerContext>();

            let status = controller.state(node.controller_id as _).status;

            if status == gamepad_rs::ControllerStatus::Connected {
                let state = controller.state(node.controller_id as _);

                let x = state.analog_state[0];

                node.fish.input.left = x < -0.5;
                node.fish.input.right = x > 0.5;

                const JUMP_BTN: usize = 2;
                const FIRE_BTN: usize = 1;
                const THROW_BTN: usize = 3;

                if state.digital_state[JUMP_BTN] && node.fish.input.was_jump == false {
                    node.fish.input.jump = true;
                } else {
                    node.fish.input.jump = false;
                }
                node.fish.input.was_jump = state.digital_state[JUMP_BTN];

                if state.digital_state[FIRE_BTN] && node.fish.input.was_fire == false {
                    node.fish.input.fire = true;
                } else {
                    node.fish.input.fire = false;
                }
                node.fish.input.was_fire = state.digital_state[FIRE_BTN];

                if state.digital_state[THROW_BTN] && node.fish.input.was_throw == false {
                    node.fish.input.throw = true;
                } else {
                    node.fish.input.throw = false;
                }
                node.fish.input.was_throw = state.digital_state[THROW_BTN];
            }
        }

        #[cfg(not(target_os = "macos"))]
        if game_started && node.controller_id == 0 {
            node.fish.input.jump = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::W);
            node.fish.input.throw = is_key_pressed(KeyCode::Z);

            node.fish.input.fire =
                is_key_pressed(KeyCode::LeftControl) || is_key_pressed(KeyCode::F);
            node.fish.input.left = is_key_down(KeyCode::A);
            node.fish.input.right = is_key_down(KeyCode::D);
        }

        #[cfg(not(target_os = "macos"))]
        if game_started && node.controller_id == 1 {
            node.fish.input.jump = is_key_pressed(KeyCode::Up);
            node.fish.input.throw = is_key_pressed(KeyCode::K);
            node.fish.input.fire = is_key_pressed(KeyCode::L);
            node.fish.input.left = is_key_down(KeyCode::Left);
            node.fish.input.right = is_key_down(KeyCode::Right);
        }

        {
            let node = &mut *node;
            let fish = &mut node.fish;

            let mut resources = storage::get_mut::<Resources>();
            fish.pos = resources.collision_world.actor_pos(fish.collider);

            fish.on_ground = resources
                .collision_world
                .collide_check(fish.collider, fish.pos + vec2(0., 1.));

            if fish.on_ground == false {
                fish.speed.y += consts::GRAVITY * get_frame_time();
            }

            if fish.on_ground {
                node.jump_grace_timer = consts::JUMP_GRACE_TIME;
            } else if node.jump_grace_timer > 0. {
                node.jump_grace_timer -= get_frame_time();
            }

            resources
                .collision_world
                .move_h(fish.collider, fish.speed.x * get_frame_time());
            if !resources
                .collision_world
                .move_v(fish.collider, fish.speed.y * get_frame_time())
            {
                fish.speed.y = 0.0;
            }
        }
        StateMachine::update_detached(&mut node, |node| &mut node.state_machine);
    }
}
