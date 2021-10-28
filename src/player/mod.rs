use std::collections::HashMap;

use macroquad::{
    audio::{self, play_sound_once},
    experimental::{
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, Handle, HandleUntyped, RefMut},
        state_machine::{State, StateMachine},
    },
    prelude::*,
};

use crate::{
    capabilities::{NetworkReplicate, PhysicsObject},
    components::PhysicsBody,
    items::{Item, ItemKind, Weapon},
    GameInput, GameWorld, ParticleEmitters, PassiveEffectInstance, PassiveEffectParams, Resources,
};

use crate::components::{Animation, AnimationParams, AnimationPlayer};

use crate::items::EquippedItem;

mod ai;
mod events;

pub use events::{PlayerEvent, PlayerEventParams};

pub struct Player {
    pub id: u8,

    pub body: PhysicsBody,
    animation_player: AnimationPlayer,

    pub is_dead: bool,

    pub weapon: Option<Weapon>,
    pub equipped_items: HashMap<String, EquippedItem>,

    pub passive_effects: HashMap<String, PassiveEffectInstance>,

    pub input: GameInput,
    pub last_frame_input: GameInput,
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
    pub const _LEGS_THRESHOLD: f32 = 42.0;

    pub const ST_NORMAL: usize = 0;
    pub const ST_DEATH: usize = 1;
    pub const ST_ATTACK: usize = 2;
    pub const ST_SLIDE: usize = 3;
    pub const ST_INCAPACITATED: usize = 4;
    pub const ST_AFTERMATCH: usize = 5;

    pub const JUMP_UPWARDS_SPEED: f32 = 600.0;
    pub const JUMP_HEIGHT_CONTROL_FRAMES: i32 = 8;
    pub const JUMP_RELEASE_GRAVITY_INCREASE: f32 = 35.0;

    pub const RUN_SPEED: f32 = 250.0;
    pub const SLIDE_SPEED: f32 = 800.0;
    pub const SLIDE_DURATION: f32 = 0.1;
    pub const JUMP_GRACE_TIME: f32 = 0.15;
    pub const PICK_GRACE_TIME: f32 = 0.30;
    pub const FLOAT_SPEED: f32 = 100.0;

    pub const INCAPACITATED_BREAK_FACTOR: f32 = 0.9;
    pub const INCAPACITATED_STOP_THRESHOLD: f32 = 20.0;

    const ITEM_THROW_FORCE: f32 = 600.0;

    const WEAPON_MOUNT_X_OFFSET: f32 = 0.0;
    const WEAPON_MOUNT_Y_OFFSET: f32 = 16.0;
    const _WEAPON_MOUNT_Y_OFFSET_CROUCHED: f32 = 32.0;

    const WEAPON_HUD_Y_OFFSET: f32 = -16.0;

    const COLLIDER_WIDTH: f32 = 20.0;
    const COLLIDER_HEIGHT: f32 = 54.0;
    const COLLIDER_HEIGHT_CROUCHED: f32 = 38.0;

    const IDLE_ANIMATION_ID: &'static str = "idle";
    const MOVE_ANIMATION_ID: &'static str = "move";
    const DEATH_ANIMATION_ID: &'static str = "death";
    const DEATH_ALT_ANIMATION_ID: &'static str = "death2";
    const FLOAT_ANIMATION_ID: &'static str = "float";
    const CROUCH_ANIMATION_ID: &'static str = "crouch";
    const SLIDE_ANIMATION_ID: &'static str = "slide";

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
            Self::ST_ATTACK,
            State::new()
                .update(Self::update_attack)
                .coroutine(Self::attack_coroutine),
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

            let size = vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT);
            let collider_offset = vec2(-Self::COLLIDER_WIDTH / 2.0, 0.0);

            PhysicsBody::new(
                &mut world.collision_world,
                spawn_point,
                0.0,
                size,
                false,
                false,
                collider_offset,
            )
        };

        let texture_id = if controller_id == 0 {
            "player_blue"
        } else {
            "player_green"
        };

        let frame_size = {
            let resources = storage::get::<Resources>();
            let texture_entry = resources.textures.get(texture_id).unwrap();
            texture_entry.meta.sprite_size.unwrap()
        };

        let animation_player = AnimationPlayer::new(AnimationParams {
            texture_id: texture_id.to_string(),
            offset: vec2(-(frame_size.x as f32 / 2.0), -10.0),
            frame_size: Some(frame_size),
            animations: vec![
                Animation {
                    id: Self::IDLE_ANIMATION_ID.to_string(),
                    row: 0,
                    frames: 7,
                    fps: 12,
                },
                Animation {
                    id: Self::MOVE_ANIMATION_ID.to_string(),
                    row: 2,
                    frames: 6,
                    fps: 10,
                },
                Animation {
                    id: Self::DEATH_ANIMATION_ID.to_string(),
                    row: 12,
                    frames: 3,
                    fps: 5,
                },
                Animation {
                    id: Self::DEATH_ALT_ANIMATION_ID.to_string(),
                    row: 14,
                    frames: 4,
                    fps: 8,
                },
                Animation {
                    id: Self::FLOAT_ANIMATION_ID.to_string(),
                    row: 6,
                    frames: 4,
                    fps: 8,
                },
                Animation {
                    id: Self::CROUCH_ANIMATION_ID.to_string(),
                    row: 16,
                    frames: 2,
                    fps: 8,
                },
                Animation {
                    id: Self::SLIDE_ANIMATION_ID.to_string(),
                    row: 18,
                    frames: 2,
                    fps: 8,
                },
            ],
            should_autoplay: true,
            ..Default::default()
        });

        Player {
            id: player_id,
            is_dead: false,
            weapon: None,
            equipped_items: HashMap::new(),
            passive_effects: HashMap::new(),
            input: Default::default(),
            last_frame_input: Default::default(),
            pick_grace_timer: 0.,
            body,
            animation_player,
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

    pub fn add_passive_effect(&mut self, item_id: Option<&str>, params: PassiveEffectParams) {
        let effect = PassiveEffectInstance::new(item_id, params);

        if let Some(particle_effect_id) = &effect.particle_effect_id {
            let mut particle_emitters = scene::find_node_by_type::<ParticleEmitters>().unwrap();
            particle_emitters.spawn(particle_effect_id, self.body.position);
        }

        self.passive_effects.insert(effect.id.clone(), effect);
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

            let mut item = Item::new(self.body.position, params);

            if is_thrown {
                item.body.velocity = self.body.facing_dir() * Self::ITEM_THROW_FORCE;
            }

            scene::add_node(item);
        }
    }

    pub fn pick_up_weapon(&mut self, weapon: Weapon) {
        let resources = storage::get::<Resources>();
        let sound = resources.sounds["pickup"];

        play_sound_once(sound);

        self.drop_weapon(false);

        self.weapon = Some(weapon);
    }

    pub fn pick_up_equipped_item(&mut self, equipped_item: EquippedItem) {
        let resources = storage::get::<Resources>();
        let sound = resources.sounds["pickup"];
        play_sound_once(sound);

        self.equipped_items
            .insert(equipped_item.id.clone(), equipped_item);
    }

    pub fn get_weapon_mount_position(&self) -> Vec2 {
        let mut offset = Vec2::ZERO;

        if self.body.is_upside_down {
            offset.y = -Self::WEAPON_MOUNT_Y_OFFSET;
        } else {
            offset.y = Self::WEAPON_MOUNT_Y_OFFSET;
        }

        if self.body.is_facing_right {
            offset.x = Self::WEAPON_MOUNT_X_OFFSET;
        } else {
            offset.x = -Self::WEAPON_MOUNT_X_OFFSET;
        }

        let size = self.animation_player.get_size();
        let mut position = self.body.position + offset;
        position.y -= size.y - self.body.size.y;

        if self.body.is_upside_down {
            position.y += size.y;
        }

        // TODO: Implement rotation

        position + offset
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

    // This should only be used under special circumstances, when you want to override a damage
    // blocking effect, for example. To give damage to a player, use `Player::on_receive_damage`
    pub fn kill(&mut self, is_from_right: bool) {
        if self.state_machine.state() != Self::ST_DEATH {
            self.body.is_facing_right = is_from_right;

            self.drop_weapon(false);

            {
                let position = self.body.position;
                let resources = storage::get::<Resources>();
                for (_, item) in self.equipped_items.drain() {
                    if item.is_dropped_on_death {
                        let params = resources.items.get(&item.id).cloned().unwrap();
                        scene::add_node(Item::new(position, params));
                    }
                }
            }

            self.passive_effects.clear();

            self.state_machine.set_state(Self::ST_DEATH);

            {
                let resources = storage::get::<Resources>();
                let sound = resources.sounds["death"];
                play_sound_once(sound);
            }
        }
    }

    pub fn apply_input(&mut self, input: GameInput) {
        self.last_frame_input = self.input;
        self.input = input;
    }

    #[allow(dead_code)]
    pub fn incapacitate(&mut self, duration: f32, should_stop: bool, should_fall: bool) {
        if should_stop {
            self.body.velocity.x = 0.0;
        }

        self.incapacitated_duration = duration;
        self.incapacitated_timer = 0.0;
        self.state_machine.set_state(Self::ST_INCAPACITATED);
        if should_fall {
            self.animation_player
                .set_animation(Self::SLIDE_ANIMATION_ID);
        }
    }

    fn incapacitated_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        let player_handle = node.handle();

        let position = node.body.position;

        for effect in node.passive_effects.values_mut() {
            // TODO: Move this to where the player is incapacitated and collect the handle of the other player
            let event = PlayerEventParams::Incapacitated {
                incapacitated_by: None,
            };
            effect.on_player_event(player_handle, position, event);
        }

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

                node.is_dead = true;
                node.animation_player
                    .set_animation(Self::DEATH_ANIMATION_ID);

                // let mut score_counter = scene::get_node(node.score_counter);
                // score_counter.count_loss(node.controller_id)
            }

            let is_out_of_bounds = {
                let node = scene::get_node(handle);
                node.body.position.y < map_bottom
            };

            if is_out_of_bounds {
                // give some take for a dead fish to take off the ground
                wait_seconds(0.1).await;

                // wait until it lands (or fall down the map)
                let mut should_continue = false;
                while should_continue {
                    next_frame().await;

                    should_continue = {
                        let node = scene::get_node(handle);
                        !(node.body.is_on_ground || node.body.position.y > map_bottom)
                    };
                }

                {
                    let mut node = scene::get_node(handle);
                    node.animation_player
                        .set_animation(Self::DEATH_ALT_ANIMATION_ID);
                    node.body.velocity = vec2(0., 0.);
                }

                wait_seconds(0.5).await;
            }

            {
                let mut node = scene::get_node(handle);
                let pos = node.body.position;

                node.animation_player.stop();
                node.body.velocity = vec2(0., 0.);

                let mut particles = scene::find_node_by_type::<ParticleEmitters>().unwrap();
                particles.spawn("explosion", pos + vec2(15.0, 33.0));
            }

            wait_seconds(0.5).await;

            let mut node = scene::get_node(handle);

            if node.can_head_boink {
                // Shoes::spawn(this.body.pos);
                node.can_head_boink = false;
            }

            node.body.position = {
                let world = storage::get_mut::<GameWorld>();
                world.get_random_spawn_point()
            };

            node.animation_player.play();

            // in deathmatch we can just get back to normal after death
            {
                let mut world = storage::get_mut::<GameWorld>();

                node.state_machine.set_state(Self::ST_NORMAL);
                node.is_dead = false;
                world
                    .collision_world
                    .set_actor_position(node.body.collider, node.body.position);
            }
        };

        start_coroutine(coroutine)
    }

    fn attack_coroutine(node: &mut RefMut<Player>) -> Coroutine {
        Weapon::use_coroutine(node.handle())
    }

    fn update_incapacitated(node: &mut RefMut<Player>, dt: f32) {
        node.incapacitated_timer += dt;
        if node.incapacitated_timer >= node.incapacitated_duration {
            node.incapacitated_timer = 0.0;
            node.incapacitated_duration = 0.0;
            node.state_machine.set_state(Player::ST_NORMAL);
        }
    }

    fn update_attack(node: &mut RefMut<Player>, _dt: f32) {
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
                node.body.velocity.x = if node.body.is_facing_right {
                    Self::SLIDE_SPEED
                } else {
                    -Self::SLIDE_SPEED
                };
                node.animation_player
                    .set_animation(Self::SLIDE_ANIMATION_ID);
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

        #[cfg(debug_assertions)]
        if is_key_pressed(KeyCode::Y) {
            Player::on_receive_damage(node.handle(), true, None);
        }

        //let node = &mut **node;

        if node.is_crouched {
            node.body.velocity.x = 0.0;

            if node.input.right {
                node.body.is_facing_right = true;
            } else if node.input.left {
                node.body.is_facing_right = false;
            }
        } else {
            //
            if node.input.right {
                node.body.velocity.x = Self::RUN_SPEED;
                node.body.is_facing_right = true;
            } else if node.input.left {
                node.body.velocity.x = -Self::RUN_SPEED;
                node.body.is_facing_right = false;
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
        if node.body.is_on_ground && !node.body.was_on_ground_last_frame {
            {
                let resources = storage::get::<Resources>();
                let land_sound = resources.sounds["land"];

                play_sound_once(land_sound);
            }
        }

        if node.floating {
            node.animation_player
                .set_animation(Self::FLOAT_ANIMATION_ID);
        } else if node.is_crouched {
            node.animation_player
                .set_animation(Self::CROUCH_ANIMATION_ID);
        } else {
            //
            if node.input.right || node.input.left {
                node.animation_player.set_animation(Self::MOVE_ANIMATION_ID);
            } else {
                node.animation_player.set_animation(Self::IDLE_ANIMATION_ID);
            }
        }

        // if in jump and want to jump again
        if !node.body.is_on_ground
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
        if node.body.is_on_ground {
            node.was_floating = false;
            node.floating = false;
        }
        if node.floating {
            node.body.velocity.y = Self::FLOAT_SPEED;
        } else {
            node.body.has_gravity = true;
        }

        node.is_crouched = node.body.is_on_ground && node.input.down;

        if node.body.is_on_ground {
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
                if !node.body.is_on_ground {
                    node.pick_grace_timer = Self::PICK_GRACE_TIME;
                }

                // when the flocating fish is throwing a weapon and keeps
                // floating it looks less cool than if its stop floating and
                // falls, but idk
                node.floating = false;
            } else if node.pick_grace_timer <= 0.0 {
                for item in scene::find_nodes_by_type::<Item>() {
                    if let ItemKind::Weapon { params } = &item.kind {
                        if node
                            .get_collider_rect()
                            .overlaps(&item.body.get_collider_rect())
                        {
                            let weapon = Weapon::new(&item.id, params.clone());
                            node.pick_up_weapon(weapon);
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
                node.state_machine.set_state(Self::ST_ATTACK);
                node.floating = false;
            }
        }

        for item in scene::find_nodes_by_type::<Item>() {
            if let ItemKind::EquippedItem { params } = &item.kind {
                if node
                    .get_collider_rect()
                    .overlaps(&item.body.get_collider_rect())
                {
                    let equipment = EquippedItem::new(&item.id, params.clone(), node);

                    node.pick_up_equipped_item(equipment);
                    item.delete();
                }
            }
        }
    }

    pub fn get_collider_rect(&self) -> Rect {
        let state = self.state_machine.state();

        let mut rect = self.body.get_collider_rect();

        if state == Self::ST_INCAPACITATED || state == Self::ST_SLIDE || self.is_crouched {
            rect.y += rect.h - Self::COLLIDER_HEIGHT_CROUCHED;
            rect.h = Self::COLLIDER_HEIGHT_CROUCHED;
        }

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

        node.animation_player.update();

        let dt = get_frame_time();

        if let Some(weapon) = &mut node.weapon {
            weapon.update(dt);
        }

        for item in node.equipped_items.values_mut() {
            item.update(dt);
        }

        node.equipped_items.retain(|_, item| !item.is_depleted());

        for effect in node.passive_effects.values_mut() {
            effect.update(dt);
        }

        node.passive_effects
            .retain(|_, effect| !effect.is_depleted());

        {
            let player_handle = node.handle();
            let position = node.body.position;

            for effect in node.passive_effects.values_mut() {
                let event = PlayerEventParams::Update { dt };
                effect.on_player_event(player_handle, position, event);
            }
        }

        let map_bottom = {
            let world = storage::get::<GameWorld>();

            world.map.grid_size.y as f32 * world.map.tile_size.y
        } as f32;

        if node.body.position.y > map_bottom {
            Player::on_receive_damage(node.handle(), false, None);
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

        // if is_key_pressed(KeyCode::Q) {
        //     //Will fail half of the time, because it is triggered by both players and it's a 50% chance that they counteract each other.
        //     scene::find_node_by_type::<crate::nodes::Camera>()
        //         .unwrap()
        //         .shake_rotational(1.0, 10);
        // }

        {
            let node = &mut *node;

            if node.body.is_on_ground && !node.input.jump {
                node.jump_grace_timer = Self::JUMP_GRACE_TIME;
            } else if node.jump_grace_timer > 0. {
                node.jump_grace_timer -= get_frame_time();
            }

            node.body.update();
        }

        if node.can_head_boink && node.body.velocity.y > 0.0 {
            let hitbox = node.get_collider_rect();
            for player in scene::find_nodes_by_type::<Player>() {
                let other_hitbox = player.get_collider_rect();
                let is_overlapping = hitbox.overlaps(&other_hitbox);
                if is_overlapping && hitbox.y + 60.0 < other_hitbox.y + Self::HEAD_THRESHOLD {
                    let resources = storage::get::<Resources>();
                    let jump_sound = resources.sounds["jump"];

                    play_sound_once(jump_sound);
                    Player::on_receive_damage(
                        player.handle(),
                        !node.body.is_facing_right,
                        Some(node.handle()),
                    );
                }
            }
        }

        // update camera bound box
        {
            let fish_box = Rect::new(node.body.position.x, node.body.position.y, 32., 60.);

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

        if is_key_pressed(KeyCode::B) {
            node.body.is_upside_down = !node.body.is_upside_down;
        }

        StateMachine::update_detached(node, |node| &mut node.state_machine);
    }

    fn draw_player(&self) {
        for equipped in self.equipped_items.values() {
            let size = self.animation_player.get_size();
            let mut position = self.body.position;
            position.y -= size.y - self.body.size.y;

            if self.body.is_upside_down {
                position.y += size.y;
            }

            equipped.draw(
                position,
                self.body.rotation,
                !self.body.is_facing_right,
                self.body.is_upside_down,
            );
        }

        self.animation_player.draw(
            self.body.position,
            self.body.rotation,
            !self.body.is_facing_right,
            self.body.is_upside_down,
        );

        #[cfg(debug_assertions)]
        self.animation_player.debug_draw(self.body.position);

        #[cfg(debug_assertions)]
        self.body.debug_draw();
    }

    fn draw_weapon(&mut self) {
        let position = self.get_weapon_mount_position();
        if let Some(weapon) = &mut self.weapon {
            weapon.draw(
                position,
                self.body.rotation,
                !self.body.is_facing_right,
                self.body.is_upside_down,
            );

            let mut position = self.body.position;
            position.y += Self::WEAPON_HUD_Y_OFFSET;

            weapon.draw_hud(position);
        }
    }
}

impl scene::Node for Player {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::physics_capabilities());
        node.provides(Self::network_capabilities());
    }

    fn draw(mut node: RefMut<Self>) {
        if node.body.is_facing_right {
            node.draw_player();
            node.draw_weapon();
        } else {
            node.draw_weapon();
            node.draw_player();
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

            node.get_collider_rect()
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

impl Player {
    pub fn on_receive_damage(
        player_handle: Handle<Player>,
        is_from_right: bool,
        damage_from: Option<Handle<Player>>,
    ) -> Coroutine {
        let coroutine = async move {
            if let Some(mut node) = scene::try_get_node(player_handle) {
                if node.state_machine.state() != Self::ST_DEATH {
                    let position = node.body.position;

                    let mut is_damage_blocked = false;

                    for effect in node.passive_effects.values_mut() {
                        if effect.blocks_damage
                            && effect.events.contains(&PlayerEvent::ReceiveDamage)
                        {
                            is_damage_blocked = true;
                        }

                        let params = PlayerEventParams::ReceiveDamage {
                            is_from_right,
                            damage_from,
                            is_damage_blocked,
                        };

                        effect.on_player_event(player_handle, position, params);
                    }

                    if !is_damage_blocked {
                        node.kill(is_from_right);
                    }

                    if let Some(damage_from) = damage_from {
                        Player::on_give_damage(damage_from, player_handle, is_damage_blocked);
                    }
                }
            }
        };

        start_coroutine(coroutine)
    }

    pub fn on_give_damage(
        player_handle: Handle<Player>,
        damage_to: Handle<Player>,
        is_damage_blocked: bool,
    ) -> Coroutine {
        let coroutine = async move {
            if let Some(mut node) = scene::try_get_node(player_handle) {
                let position = node.body.position;

                for effect in node.passive_effects.values_mut() {
                    let params = PlayerEventParams::GiveDamage {
                        damage_to,
                        is_damage_blocked,
                    };
                    effect.on_player_event(player_handle, position, params);
                }
            }
        };

        start_coroutine(coroutine)
    }

    pub fn on_collision(
        player_handle: Handle<Player>,
        collision_with: Handle<Player>,
    ) -> Coroutine {
        let coroutine = async move {
            if let Some(mut node) = scene::try_get_node(player_handle) {
                let position = node.body.position;

                for effect in node.passive_effects.values_mut() {
                    let params = PlayerEventParams::Collision { collision_with };
                    effect.on_player_event(player_handle, position, params);
                }
            }
        };

        start_coroutine(coroutine)
    }
}
