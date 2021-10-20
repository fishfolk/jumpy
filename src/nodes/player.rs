use std::thread::spawn;
use macroquad::{
    audio::{self, play_sound_once},
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, HandleUntyped, RefMut},
        state_machine::{State, StateMachine},
    },
    prelude::*,
};

use crate::{
    capabilities::{NetworkReplicate, PhysicsObject},
    components::PhysicsBody,
    items::{
        weapons::{
            Weapon,
            WeaponEffectKind,
        },
        Item, ItemKind,
    },
    math::{deg_to_rad, rotate_vector},
    nodes::ParticleEmitters,
    GameWorld, Input, Resources,
};

mod ai;

pub struct Player {
    pub id: u8,

    pub body: PhysicsBody,
    sprite: AnimatedSprite,

    pub dead: bool,

    pub weapon: Option<Weapon>,

    pub input: Input,
    pub last_frame_input: Input,
    pub pick_grace_timer: f32,

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

    pub is_crouched: bool,

    pub incapacitated_duration: f32,
    pub incapacitated_timer: f32,

    pub back_armor: i32,
    pub can_head_boink: bool,
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
    pub const PICK_GRACE_TIME: f32 = 0.30;
    pub const FLOAT_SPEED: f32 = 100.0;

    pub const INCAPACITATED_BREAK_FACTOR: f32 = 0.9;
    pub const INCAPACITATED_STOP_THRESHOLD: f32 = 20.0;

    const ITEM_THROW_FORCE: f32 = 800.0;
    const WEAPON_HUD_Y_OFFSET: f32 = 16.0;

    // TODO: Fix offsetting of player sprite and collider, so that origin is always on the middle
    // of the sprite x-axis
    const SPRITE_X_OFFSET: f32 = 20.0;

    pub fn new(player_id: u8, controller_id: i32) -> Player {
        let spawn_point = {
            let world = storage::get_mut::<GameWorld>();
            world.get_random_spawn_point()
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

        let body = {
            let mut world = storage::get_mut::<GameWorld>();

            PhysicsBody::new(
                &mut world.collision_world,
                spawn_point,
                0.0,
                vec2(30.0, 54.0),
                false,
            )
        };

        let sprite = AnimatedSprite::new(
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
            pick_grace_timer: 0.,
            body,
            sprite,
            jump_grace_timer: 0.,
            jump_frames_left: 0,
            floating: false,
            was_floating: false,
            state_machine,
            controller_id,
            remote_control: false,
            ai_enabled: false, //controller_id == 0,
            ai: Some(ai::Ai::new()),
            camera_box: Rect::new(spawn_point.x - 30., spawn_point.y - 150., 100., 210.),
            can_head_boink: false,
            back_armor: 0,
            is_crouched: false,
            incapacitated_timer: 0.0,
            incapacitated_duration: 0.0,
        }
    }

    pub fn drop_weapon(&mut self, is_thrown: bool) {
        if let Some(weapon) = self.weapon.take() {
            let params = {
                let resources = storage::get::<Resources>();
                resources
                    .items
                    .get(&weapon.id)
                    .cloned()
                    .unwrap_or_else(|| panic!("Player: Invalid weapon ID '{}'", &weapon.id))
            };

            let mut item = Item::new(self.body.pos, params);

            if is_thrown {
                item.body.velocity = self.body.facing_dir() * Self::ITEM_THROW_FORCE;
            }

            scene::add_node(item);
        }
    }

    pub fn pick_weapon(&mut self, weapon: Weapon) {
        let resources = storage::get::<Resources>();
        let pickup_sound = resources.sounds["pickup"];

        play_sound_once(pickup_sound);

        self.drop_weapon(false);

        self.weapon = Some(weapon);
    }

    pub fn get_weapon_mount(&self) -> Vec2 {
        let mut offset = vec2(Self::SPRITE_X_OFFSET, 0.0);

        if self.is_crouched {
            offset.y = 32.0;
        } else {
            offset.y = 16.0;
        }

        offset
    }

    pub fn jump(&mut self) {
        let resources = storage::get::<Resources>();
        let jump_sound = resources.sounds["jump"];

        self.body.velocity.y = -Self::JUMP_UPWARDS_SPEED;
        self.jump_frames_left = Self::JUMP_HEIGHT_CONTROL_FRAMES;

        audio::play_sound(
            jump_sound,
            audio::PlaySoundParams {
                looped: false,
                volume: 0.6,
            },
        );
    }

    fn slide(&mut self) {
        self.state_machine.set_state(Self::ST_SLIDE);
    }

    pub fn apply_input(&mut self, input: Input) {
        self.last_frame_input = self.input;
        self.input = input;
    }

    pub fn incapacitate(&mut self, duration: f32, should_stop: bool, should_fall: bool) {
        if should_stop {
            self.body.velocity.x = 0.0;
        }
        self.incapacitated_duration = duration;
        self.incapacitated_timer = 0.0;
        self.state_machine.set_state(Self::ST_INCAPACITATED);
        if should_fall {
            self.sprite.set_animation(6);
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
                    let die_sound = resources.sounds["death"];

                    play_sound_once(die_sound);
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
            let world = storage::get::<GameWorld>();

            world.map.grid_size.y as f32 * world.map.tile_size.y
        } as f32;

        let coroutine = async move {
            {
                let mut node = scene::get_node(handle);
                node.body.velocity.x = -300. * node.body.facing_dir().x;
                node.body.velocity.y = -150.;
                node.body.has_gravity = true;

                node.dead = true;
                node.sprite.set_animation(2);

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
                    node.sprite.set_animation(3);
                    node.body.velocity = vec2(0., 0.);
                }

                wait_seconds(0.5).await;
            }

            {
                let mut node = scene::get_node(handle);
                let pos = node.body.pos;

                node.sprite.playing = false;
                node.body.velocity = vec2(0., 0.);

                let mut particles = scene::find_node_by_type::<ParticleEmitters>().unwrap();
                particles.explosions.spawn(pos + vec2(15., 33.));
            }

            wait_seconds(0.5).await;

            let mut this = scene::get_node(handle);

            if this.can_head_boink {
                // Shoes::spawn(this.body.pos);
                this.can_head_boink = false;
            }

            this.body.pos = {
                let world = storage::get_mut::<GameWorld>();
                world.get_random_spawn_point()
            };
            this.sprite.playing = true;
            this.drop_weapon(false);

            // in deathmatch we can just get back to normal after death
            {
                let mut world = storage::get_mut::<GameWorld>();

                this.state_machine.set_state(Self::ST_NORMAL);
                this.dead = false;
                world
                    .collision_world
                    .set_actor_position(this.body.collider, this.body.pos);
            }
        };

        start_coroutine(coroutine)
    }

    fn shoot_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        let player = node.handle();

        if node.weapon.is_some() && node.weapon.as_ref().unwrap().is_ready() {
            Weapon::attack_coroutine(player)
        } else {
            let coroutine = async move {
                let player = &mut *scene::get_node(player);
                player.state_machine.set_state(Player::ST_NORMAL);
            };

            start_coroutine(coroutine)
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
        node.body.velocity.x *= 0.9;
    }

    fn update_aftermatch(node: &mut RefMut<Player>, _dt: f32) {
        node.body.velocity.x = 0.0;
    }

    fn update_slide(_node: &mut RefMut<Player>, _dt: f32) {}

    fn slide_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        let handle = node.handle();

        let coroutine = async move {
            {
                let mut node = scene::get_node(handle);
                node.body.velocity.x = if node.body.facing {
                    Self::SLIDE_SPEED
                } else {
                    -Self::SLIDE_SPEED
                };
                node.sprite.set_animation(6);
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

        let node = &mut **node;

        if node.is_crouched {
            node.body.velocity.x = 0.0;

            if node.input.right {
                node.body.facing = true;
            } else if node.input.left {
                node.body.facing = false;
            }
        } else {
            //
            if node.input.right {
                node.body.velocity.x = Self::RUN_SPEED;
                node.body.facing = true;
            } else if node.input.left {
                node.body.velocity.x = -Self::RUN_SPEED;
                node.body.facing = false;
            } else {
                node.body.velocity.x = 0.;
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
                let land_sound = resources.sounds["land"];

                play_sound_once(land_sound);
            }
        }

        if node.floating {
            node.sprite.set_animation(4);
        } else if node.is_crouched {
            node.sprite.set_animation(5);
        } else {
            //
            if node.input.right || node.input.left {
                node.sprite.set_animation(1);
            } else {
                node.sprite.set_animation(0);
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
                node.body.has_gravity = false;
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
            node.body.velocity.y = Self::FLOAT_SPEED;
        } else {
            node.body.has_gravity = true;
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

        if node.weapon.is_none() && node.pick_grace_timer > 0. {
            node.pick_grace_timer -= get_frame_time();
        }

        if node.input.pickup && !node.last_frame_input.pickup {
            if node.weapon.is_some() {
                node.drop_weapon(true);

                {
                    let resources = storage::get::<Resources>();
                    let throw_sound = resources.sounds["throw"];

                    play_sound_once(throw_sound);
                }

                // set a grace time for picking up the weapon again
                if !node.body.on_ground {
                    node.pick_grace_timer = Self::PICK_GRACE_TIME;
                }

                // when the flocating fish is throwing a weapon and keeps
                // floating it looks less cool than if its stop floating and
                // falls, but idk
                node.floating = false;
            } else if node.pick_grace_timer <= 0.0 {
                for item in scene::find_nodes_by_type::<Item>() {
                    if node.get_hitbox().overlaps(&item.get_collider()) {
                        let was_picked_up = match &item.kind {
                            ItemKind::Weapon { params } => {
                                let weapon = Weapon::new(&item.id, params.clone());
                                node.pick_weapon(weapon);
                                true
                            }
                            ItemKind::Misc => false,
                        };

                        if was_picked_up {
                            item.delete();
                            break;
                        }
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
        let mut rect = Rect::new(
            self.body.pos.x + Self::SPRITE_X_OFFSET,
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
        );

        rect.x -= rect.w / 2.0;
        rect
    }

    fn network_update(mut node: RefMut<Self>) {
        // Break incapacitated
        if node.state_machine.state() == Player::ST_INCAPACITATED && node.body.velocity.x != 0.0 {
            if node.body.velocity.x > Player::INCAPACITATED_STOP_THRESHOLD
                || node.body.velocity.x < -Player::INCAPACITATED_STOP_THRESHOLD
            {
                node.body.velocity.x *= Player::INCAPACITATED_BREAK_FACTOR;
            } else {
                node.body.velocity.x = 0.0;
            }
        }

        {
            // if scene::get_node(node.game_state).game_paused {
            //     return;
            // }
        }

        node.sprite.update();
        if let Some(weapon) = &mut node.weapon {
            weapon.update();
        }

        let map_bottom = {
            let world = storage::get::<GameWorld>();

            world.map.grid_size.y as f32 * world.map.tile_size.y
        } as f32;

        if node.body.pos.y > map_bottom {
            node.kill(false);
        }

        if node.input.jump {
            if node.jump_frames_left > 0 {
                node.body.velocity.y = -Player::JUMP_UPWARDS_SPEED;
                node.jump_frames_left -= 1;
            }
        } else {
            if node.body.velocity.y < 0.0 {
                node.body.velocity.y += Player::JUMP_RELEASE_GRAVITY_INCREASE;
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

        if node.can_head_boink && node.body.velocity.y > 0.0 {
            let hitbox = node.get_hitbox();
            for mut other in scene::find_nodes_by_type::<Player>() {
                let other_hitbox = other.get_hitbox();
                let is_overlapping = hitbox.overlaps(&other_hitbox);
                if is_overlapping && hitbox.y + 60.0 < other_hitbox.y + Self::HEAD_THRESHOLD {
                    let resources = storage::get::<Resources>();
                    let jump_sound = resources.sounds["jump"];

                    play_sound_once(jump_sound);
                    other.kill(!node.body.facing);
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

        let texture_id = if node.controller_id == 0 {
            if node.can_head_boink {
                "player_with_boots_blue"
            } else {
                "player_blue"
            }
        } else if node.can_head_boink {
            "player_with_boots_green"
        } else {
            "player_green"
        };

        let texture_entry = resources.textures
            .get(texture_id)
            .unwrap();

        let dest_size = node.sprite.frame().dest_size;

        draw_texture_ex(
            texture_entry.texture,
            node.body.pos.x - Self::SPRITE_X_OFFSET,
            node.body.pos.y - 10.0,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.sprite.frame().source_rect),
                dest_size: Some(dest_size),
                flip_x: !node.body.facing,
                ..Default::default()
            },
        );

        draw_rectangle_lines(
            node.body.pos.x - Self::SPRITE_X_OFFSET,
            node.body.pos.y - 10.0,
            dest_size.x,
            dest_size.y,
            2.0,
            color::BLUE,
        );

        {
            let hitbox = node.get_hitbox();

            draw_rectangle_lines(
                hitbox.x,
                hitbox.y,
                hitbox.w,
                hitbox.h,
                2.0,
                color::RED,
            );
        }

        // draw turtle shell on player if the player has back armor
        if node.back_armor > 0 {
            let texture_id = if node.back_armor == 1 {
                "turtle_shell_broken"
            } else {
                "turtle_shell"
            };

            let texture_entry = resources.textures.get(texture_id).unwrap();

            draw_texture_ex(
                texture_entry.texture,
                node.body.pos.x + if node.body.facing { -15.0 } else { 20.0 },
                node.body.pos.y,
                color::WHITE,
                DrawTextureParams {
                    flip_y: node.body.facing,
                    rotation: std::f32::consts::PI / 2.0,
                    ..Default::default()
                },
            )
        }

        if let Some(weapon) = &node.weapon {
            let position = node.body.pos + node.get_weapon_mount()
                + weapon.get_mount_offset(node.body.facing_dir());

            weapon.draw(
                position,
                node.body.angle,
                None,
                !node.body.facing,
                false,
            );

            if let Some(uses) = weapon.uses {
                let mut position = node.body.pos;
                position.y -= Self::WEAPON_HUD_Y_OFFSET;

                let remaining = uses - weapon.use_cnt;

                let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
                let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);

                if uses >= Weapon::CONDENSED_USE_COUNT_THRESHOLD {
                    let x = position.x + Self::SPRITE_X_OFFSET - ((4.0 * uses as f32) / 2.0);

                    for i in 0..uses {
                        let x = x + 4.0 * i as f32;

                        if i >= remaining {
                            draw_rectangle(x, position.y - 12.0, 2.0, 12.0, empty_color);
                        } else {
                            draw_rectangle(x, position.y - 12.0, 2.0, 12.0, full_color);
                        };
                    }
                } else {
                    for i in 0..uses {
                        let x = position.x + 15.0 * i as f32;

                        if i >= remaining {
                            draw_circle_lines(x, position.y - 12.0, 4.0, 2.0, empty_color);
                        } else {
                            draw_circle(x, position.y - 12.0, 4.0, full_color);
                        };
                    }
                }
            }
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
            node.body.velocity.x = speed;
        }

        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Player>();
            node.body.velocity.y = speed;
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
