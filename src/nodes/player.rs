use macroquad::{
    audio::{self, play_sound_once},
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, HandleUntyped, NodeWith, RefMut},
        state_machine::{State, StateMachine},
    },
    prelude::*,
};

use crate::{
    capabilities::{NetworkReplicate, PhysicsObject, Weapon, WeaponTrait},
    components::PhysicsBody,
    Input, Resources,
};

use std::f32;

mod ai;

impl Player {
    pub fn drop_weapon(&mut self) {
        if let Some(weapon) = self.weapon.as_mut() {
            weapon.throw(false);
        }
        self.weapon = None;
    }

    pub fn pick_weapon(&mut self, weapon: NodeWith<Weapon>) {
        let resources = storage::get_mut::<Resources>();
        play_sound_once(resources.pickup_sound);

        self.drop_weapon();

        self.weapon = Some(weapon);
    }

    pub fn jump(&mut self) {
        let resources = storage::get::<Resources>();

        self.body.speed.y = -Self::JUMP_UPWARDS_SPEED;
        self.jump_frames_left = Self::JUMP_HEIGHT_CONTROL_FRAMES;

        audio::play_sound(
            resources.jump_sound,
            audio::PlaySoundParams {
                looped: false,
                volume: 0.6,
            },
        );
    }
    fn slide(&mut self) {
        self.state_machine.set_state(Self::ST_SLIDE);
    }
}

pub struct Player {
    pub id: u8,

    pub body: PhysicsBody,

    fish_sprite: AnimatedSprite,
    pub dead: bool,
    pub weapon: Option<NodeWith<Weapon>>,
    pub input: Input,
    pub last_frame_input: Input,

    jump_grace_timer: f32,
    jump_frames_left: i32,

    was_floating: bool,
    pub floating: bool,

    pub state_machine: StateMachine<Player>,
    pub controller_id: i32,
    pub remote_control: bool,

    ai_enabled: bool,
    ai: Option<ai::Ai>,

    pub camera_box: Rect,

    pub can_head_boink: bool,
    pub is_crouched: bool,

    pub incapacitated_duration: f32,
    pub incapacitated_timer: f32,

    pub back_armor: i32,
}

impl Player {
    pub const HEAD_THRESHOLD: f32 = 24.0;
    pub const LEGS_THRESHOLD: f32 = 42.0;

    pub const ST_NORMAL: usize = 0;
    pub const ST_DEATH: usize = 1;
    pub const ST_SHOOT: usize = 2;
    pub const ST_SLIDE: usize = 3;
    pub const ST_INCAPACITATED: usize = 4;
    pub const ST_AFTERMATCH: usize = 5;

    pub const JUMP_UPWARDS_SPEED: f32 = 600.0;
    pub const JUMP_HEIGHT_CONTROL_FRAMES: i32 = 8;
    pub const JUMP_RELEASE_GRAVITY_INCREASE: f32 = 35.0; // When up key is released and player is moving upwards, apply extra gravity to stop them fasterpub const JUMP_SPEED: f32 = 700.0;
    pub const RUN_SPEED: f32 = 250.0;
    pub const SLIDE_SPEED: f32 = 800.0;
    pub const SLIDE_DURATION: f32 = 0.1;
    pub const JUMP_GRACE_TIME: f32 = 0.15;
    pub const FLOAT_SPEED: f32 = 100.0;

    pub const INCAPACITATED_BREAK_FACTOR: f32 = 0.9;
    pub const INCAPACITATED_STOP_THRESHOLD: f32 = 20.0;

    pub fn new(player_id: u8, controller_id: i32) -> Player {
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
            Self::ST_INCAPACITATED,
            State::new()
                .update(Self::update_incapacitated)
                .coroutine(Self::incapacitated_coroutine),
        );
        state_machine.add_state(
            Self::ST_AFTERMATCH,
            State::new().update(Self::update_aftermatch),
        );

        state_machine.add_state(
            Self::ST_SLIDE,
            State::new()
                .update(Self::update_slide)
                .coroutine(Self::slide_coroutine),
        );

        let body = PhysicsBody {
            collider: {
                let mut resources = storage::get_mut::<Resources>();
                resources.collision_world.add_actor(spawner_pos, 30, 54)
            },
            on_ground: false,
            angle: 0.0,
            speed: vec2(0., 0.),
            pos: spawner_pos,
            size: vec2(30., 54.),
            facing: true,
            last_frame_on_ground: false,
            have_gravity: true,
            bouncyness: 0.0,
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
                Animation {
                    name: "crouch".to_string(),
                    row: 16,
                    frames: 2,
                    fps: 8,
                },
                Animation {
                    name: "slide".to_string(),
                    row: 18,
                    frames: 2,
                    fps: 8,
                },
            ],
            true,
        );

        Player {
            id: player_id,
            dead: false,
            weapon: None,
            input: Default::default(),
            last_frame_input: Default::default(),

            body,
            fish_sprite,
            jump_grace_timer: 0.,
            jump_frames_left: 0,
            floating: false,
            was_floating: false,
            state_machine,
            controller_id,
            remote_control: false,
            ai_enabled: false, //controller_id == 0,
            ai: Some(ai::Ai::new()),
            camera_box: Rect::new(spawner_pos.x - 30., spawner_pos.y - 150., 100., 210.),
            can_head_boink: false,
            back_armor: 0,
            is_crouched: false,
            incapacitated_timer: 0.0,
            incapacitated_duration: 0.0,
        }
    }

    pub fn apply_input(&mut self, input: Input) {
        self.last_frame_input = self.input;
        self.input = input;
    }

    pub fn incapacitate(&mut self, duration: f32, should_stop: bool, should_fall: bool) {
        if should_stop {
            self.body.speed.x = 0.0;
        }
        self.incapacitated_duration = duration;
        self.incapacitated_timer = 0.0;
        self.state_machine.set_state(Self::ST_INCAPACITATED);
        if should_fall {
            self.fish_sprite.set_animation(6);
        }
    }

    pub fn kill(&mut self, direction: bool) {
        // check if armor blocks the kill
        if direction != self.body.facing && self.back_armor > 0 {
            self.back_armor -= 1;
        } else {
            // set armor to 0
            self.back_armor = 0;
            self.body.facing = direction;
            if self.state_machine.state() != Self::ST_DEATH {
                self.state_machine.set_state(Self::ST_DEATH);
                {
                    let resources = storage::get::<Resources>();
                    play_sound_once(resources.player_die_sound);
                }
            }
        }
    }

    fn incapacitated_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        let _handle = node.handle();

        let coroutine = async move {
            // NOTHING HERE FOR NOW
        };

        start_coroutine(coroutine)
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
                node.body.speed.x = -300. * node.body.facing_dir().x;
                node.body.speed.y = -150.;
                node.body.have_gravity = true;

                node.dead = true;
                node.fish_sprite.set_animation(2);

                // let mut score_counter = scene::get_node(node.score_counter);
                // score_counter.count_loss(node.controller_id)
            }

            #[allow(clippy::blocks_in_if_conditions)]
            if {
                let node = scene::get_node(handle);
                node.body.pos.y < map_bottom
            } {
                // give some take for a dead fish to take off the ground
                wait_seconds(0.1).await;

                // wait until it lands (or fall down the map)
                while {
                    let node = scene::get_node(handle);

                    !(node.body.on_ground || node.body.pos.y > map_bottom)
                } {
                    next_frame().await;
                }

                {
                    let mut node = scene::get_node(handle);
                    node.fish_sprite.set_animation(3);
                    node.body.speed = vec2(0., 0.);
                }

                wait_seconds(0.5).await;
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
            {
                let mut resources = storage::get_mut::<Resources>();

                this.state_machine.set_state(Self::ST_NORMAL);
                this.dead = false;
                resources
                    .collision_world
                    .set_actor_position(this.body.collider, this.body.pos);
            }
        };

        start_coroutine(coroutine)
    }

    fn shoot_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        if let Some(ref weapon) = node.weapon {
            weapon.shoot(node.handle())
        } else {
            let handle = node.handle();

            start_coroutine(async move {
                let player = &mut *scene::get_node(handle);
                player.state_machine.set_state(Player::ST_NORMAL);
            })
        }
    }

    fn update_incapacitated(node: &mut RefMut<Player>, dt: f32) {
        node.incapacitated_timer += dt;
        if node.incapacitated_timer >= node.incapacitated_duration {
            node.incapacitated_timer = 0.0;
            node.incapacitated_duration = 0.0;
            node.state_machine.set_state(Player::ST_NORMAL);
        }
    }

    fn update_shoot(node: &mut RefMut<Player>, _dt: f32) {
        node.body.speed.x *= 0.9;
    }

    fn update_aftermatch(node: &mut RefMut<Player>, _dt: f32) {
        node.body.speed.x = 0.0;
    }

    fn update_slide(_node: &mut RefMut<Player>, _dt: f32) {}

    fn slide_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        let handle = node.handle();

        let coroutine = async move {
            {
                let mut node = scene::get_node(handle);
                node.body.speed.x = if node.body.facing {
                    Self::SLIDE_SPEED
                } else {
                    -Self::SLIDE_SPEED
                };
                node.fish_sprite.set_animation(6);
            }

            wait_seconds(Self::SLIDE_DURATION).await;

            {
                let mut node = scene::get_node(handle);
                node.state_machine.set_state(Self::ST_NORMAL);
            }
        };

        start_coroutine(coroutine)
    }
    fn update_normal(node: &mut RefMut<Player>, _dt: f32) {
        if node.remote_control {
            return;
        }

        // self destruct, for debugging only
        if is_key_pressed(KeyCode::Y) {
            node.kill(true);
        }
        if is_key_pressed(KeyCode::U) {
            node.kill(false);
        }

        let node_handle = node.handle();
        let node = &mut **node;

        if node.is_crouched {
            node.body.speed.x = 0.0;

            if node.input.right {
                node.body.facing = true;
            } else if node.input.left {
                node.body.facing = false;
            }
        } else {
            //
            if node.input.right {
                node.body.speed.x = Self::RUN_SPEED;
                node.body.facing = true;
            } else if node.input.left {
                node.body.speed.x = -Self::RUN_SPEED;
                node.body.facing = false;
            } else {
                node.body.speed.x = 0.;
            }
        }

        // shanke on fall
        // TODO: This needs to adjust magnitude depending on velocity on collision, it's weird and sickening otherwise
        /*if node.body.on_ground && node.body.last_frame_on_ground == false {
            scene::find_node_by_type::<crate::nodes::Camera>()
                .unwrap()
                .shake_sinusodial(0.3, 6, 0.5, f32::consts::PI / 2.);
        }*/

        // Just adding this here for the SFX for the time being
        // - Arc
        if node.body.on_ground && !node.body.last_frame_on_ground {
            {
                let resources = storage::get::<Resources>();
                play_sound_once(resources.player_landing_sound);
            }
        }

        if node.floating {
            node.fish_sprite.set_animation(4);
        } else if node.is_crouched {
            node.fish_sprite.set_animation(5);
        } else {
            //
            if node.input.right || node.input.left {
                node.fish_sprite.set_animation(1);
            } else {
                node.fish_sprite.set_animation(0);
            }
        }

        // if in jump and want to jump again
        if !node.body.on_ground
            && node.input.jump
            && !node.last_frame_input.jump
            && node.jump_grace_timer <= 0.0
        {
            //
            if !node.was_floating {
                node.floating = true;
                node.was_floating = true;
                node.body.have_gravity = false;
            }
        }
        // jump button released, stop to float
        if node.floating && !node.input.jump && node.last_frame_input.jump {
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

        node.is_crouched = node.body.on_ground && node.input.down;

        if node.body.on_ground {
            if node.input.down {
                if node.input.jump && !node.last_frame_input.jump {
                    node.body.descent();
                } else if node.input.slide && !node.last_frame_input.slide {
                    node.slide();
                }
            }
        } else if node.input.down {
            node.body.descent();
        }

        if !node.input.down
            && node.input.jump
            && !node.last_frame_input.jump
            && node.jump_grace_timer > 0.
        {
            node.jump_grace_timer = 0.0;

            node.jump();
        }

        if node.input.throw && !node.last_frame_input.throw {
            if let Some(weapon) = node.weapon.as_mut() {
                weapon.throw(!node.input.down);
                node.weapon = None;
                {
                    let resources = storage::get::<Resources>();
                    play_sound_once(resources.player_throw_sound);
                }

                // when the flocating fish is throwing a weapon and keeps
                // floating it looks less cool than if its stop floating and
                // falls, but idk
                node.floating = false;
            } else {
                let mut picked = false;

                for weapon in scene::find_nodes_with::<Weapon>() {
                    if picked {
                        break;
                    }
                    let gun_rect = weapon.collider();
                    // make it little bigger to simplify picking up
                    let gun_rect = Rect::new(
                        gun_rect.x - 50.,
                        gun_rect.y - 50.,
                        gun_rect.w + 100.,
                        gun_rect.h + 100.,
                    );
                    if weapon.is_thrown() && node.get_hitbox().overlaps(&gun_rect) {
                        picked = true;
                        weapon.pick_up(node_handle);
                        node.pick_weapon(weapon);
                    }
                }
            }
        }

        if node.input.fire {
            //
            if node.weapon.is_some() {
                node.state_machine.set_state(Self::ST_SHOOT);
                node.floating = false;
            }
        }
    }

    pub fn get_hitbox(&self) -> Rect {
        let state = self.state_machine.state();
        Rect::new(
            self.body.pos.x,
            if state == Self::ST_INCAPACITATED || state == Self::ST_SLIDE || self.is_crouched {
                self.body.pos.y + 32.0
            } else {
                self.body.pos.y
            },
            if state == Self::ST_INCAPACITATED || state == Self::ST_SLIDE {
                40.0
            } else {
                20.0
            },
            if state == Self::ST_INCAPACITATED || state == Self::ST_SLIDE || self.is_crouched {
                32.0
            } else {
                64.0
            },
        )
    }

    fn network_update(mut node: RefMut<Self>) {
        // Break incapacitated
        if node.state_machine.state() == Player::ST_INCAPACITATED && node.body.speed.x != 0.0 {
            if node.body.speed.x > Player::INCAPACITATED_STOP_THRESHOLD
                || node.body.speed.x < -Player::INCAPACITATED_STOP_THRESHOLD
            {
                node.body.speed.x *= Player::INCAPACITATED_BREAK_FACTOR;
            } else {
                node.body.speed.x = 0.0;
            }
        }

        {
            // if scene::get_node(node.game_state).game_paused {
            //     return;
            // }
        }

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
            if let Some(weapon) = node.weapon.as_mut() {
                weapon.mount(node.body.pos, node.body.facing);
            }
        }

        if node.input.jump {
            if node.jump_frames_left > 0 {
                node.body.speed.y = -Player::JUMP_UPWARDS_SPEED;
                node.jump_frames_left -= 1;
            }
        } else {
            if node.body.speed.y < 0.0 {
                node.body.speed.y += Player::JUMP_RELEASE_GRAVITY_INCREASE;
            }
            node.jump_frames_left = 0;
        }

        if node.ai_enabled {
            let mut ai = node.ai.take().unwrap();
            let input = ai.update(&mut *node);
            node.input = input;
            node.ai = Some(ai);
        }

        if is_key_pressed(KeyCode::Q) {
            //Will fail half of the time, because it is triggered by both players and it's a 50% chance that they counteract each other.
            scene::find_node_by_type::<crate::nodes::Camera>()
                .unwrap()
                .shake_rotational(1.0, 10);
        }

        {
            let node = &mut *node;

            if node.body.on_ground && !node.input.jump {
                node.jump_grace_timer = Self::JUMP_GRACE_TIME;
            } else if node.jump_grace_timer > 0. {
                node.jump_grace_timer -= get_frame_time();
            }

            node.body.update();
        }

        if node.can_head_boink && node.body.speed.y > 0.0 {
            let hitbox = node.get_hitbox();
            for mut other in scene::find_nodes_by_type::<Player>() {
                let other_hitbox = other.get_hitbox();
                let is_overlapping = hitbox.overlaps(&other_hitbox);
                if is_overlapping {
                    //
                    if hitbox.y + 60.0 < other_hitbox.y + Self::HEAD_THRESHOLD {
                        let resources = storage::get_mut::<Resources>();
                        play_sound_once(resources.jump_sound);
                        other.kill(!node.body.facing);
                    }
                }
            }
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

        StateMachine::update_detached(node, |node| &mut node.state_machine);
    }
}

impl scene::Node for Player {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::physics_capabilities());
        node.provides(Self::network_capabilities());
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get::<Resources>();

        draw_texture_ex(
            if node.controller_id == 0 {
                if node.can_head_boink {
                    resources.whale_boots_blue
                } else {
                    resources.whale_blue
                }
            } else {
                //
                if node.can_head_boink {
                    resources.whale_boots_green
                } else {
                    resources.whale_green
                }
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

        // draw turtle shell on player if the player has back armor
        if node.back_armor > 0 {
            draw_texture_ex(
                if node.back_armor == 1 {
                    resources.broken_turtleshell
                } else {
                    resources.turtleshell
                },
                node.body.pos.x + if node.body.facing { -20.0 } else { 15.0 },
                node.body.pos.y,
                color::WHITE,
                DrawTextureParams {
                    flip_y: node.body.facing,
                    rotation: std::f32::consts::PI / 2.0,
                    ..Default::default()
                },
            )
        }
    }

    fn update(mut node: RefMut<Self>) {
        if is_key_pressed(KeyCode::Key0) && node.controller_id == 0 {
            node.ai_enabled ^= true;
        }

        if is_key_pressed(KeyCode::Key1) && node.controller_id == 1 {
            node.ai_enabled ^= true;
        }
    }
}

impl Player {
    fn physics_capabilities() -> PhysicsObject {
        fn active(_: HandleUntyped) -> bool {
            true
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Player>();

            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                node.body.size.x,
                node.body.size.y,
            )
        }
        fn set_speed_x(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Player>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Player>();
            node.body.speed.y = speed;
        }

        PhysicsObject {
            active,
            collider,
            set_speed_x,
            set_speed_y,
        }
    }

    fn network_capabilities() -> NetworkReplicate {
        fn network_update(handle: HandleUntyped) {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Player>();
            Player::network_update(node);
        }

        NetworkReplicate { network_update }
    }
}
