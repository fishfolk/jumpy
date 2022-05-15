use ff_core::ecs::{Entity, World};

use ff_core::prelude::*;

use crate::{
    AnimatedSprite, AnimatedSpriteMetadata, AnimatedSpriteParams, Camera, Drawable, PassiveEffect,
    PhysicsBody,
};

mod animation;
pub mod character;
mod controller;
mod events;
mod inventory;
mod state;

pub use animation::*;
pub use character::*;
pub use controller::*;
pub use events::*;
pub use inventory::*;
pub use state::*;

pub const BODY_ANIMATED_SPRITE_ID: &str = "body";
pub const LEFT_FIN_ANIMATED_SPRITE_ID: &str = "left_fin";
pub const RIGHT_FIN_ANIMATED_SPRITE_ID: &str = "right_fin";

pub const IDLE_ANIMATION_ID: &str = "idle";
pub const MOVE_ANIMATION_ID: &str = "move";
pub const JUMP_ANIMATION_ID: &str = "jump";
pub const FALL_ANIMATION_ID: &str = "fall";
pub const CROUCH_ANIMATION_ID: &str = "crouch";
pub const SLIDE_ANIMATION_ID: &str = "slide";
pub const DEATH_BACK_ANIMATION_ID: &str = "death_back";
pub const DEATH_FORWARD_ANIMATION_ID: &str = "death_forward";

pub const WEAPON_MOUNT_TWEEN_ID: &str = "weapon_mount";
pub const ITEM_MOUNT_TWEEN_ID: &str = "item_mount";
pub const HAT_MOUNT_TWEEN_ID: &str = "hat_mount";

pub const JUMP_SOUND_ID: &str = "jump";
pub const LAND_SOUND_ID: &str = "land";

pub const RESPAWN_DELAY: f32 = 2.5;
pub const PICKUP_GRACE_TIME: f32 = 0.25;

#[derive(Debug, Clone)]
pub struct PlayerParams {
    pub index: u8,
    pub controller: PlayerControllerKind,
    pub character: CharacterMetadata,
}

pub struct Player {
    pub index: u8,
    pub state: PlayerState,
    pub damage_from: Option<DamageDirection>,
    pub is_facing_left: bool,
    pub is_upside_down: bool,
    pub is_attacking: bool,
    pub jump_frame_counter: u16,
    pub pickup_grace_timer: f32,
    pub incapacitation_timer: f32,
    pub attack_timer: f32,
    pub respawn_timer: f32,
    pub camera_box: Rect,
    pub passive_effects: Vec<PassiveEffect>,
}

impl Player {
    pub fn new(index: u8, position: Vec2) -> Self {
        let camera_box = Rect::new(position.x - 30.0, position.y - 150.0, 100.0, 210.0);

        Player {
            index,
            state: PlayerState::None,
            damage_from: None,
            is_facing_left: false,
            is_upside_down: false,
            is_attacking: false,
            jump_frame_counter: 0,
            pickup_grace_timer: 0.0,
            attack_timer: 0.0,
            incapacitation_timer: 0.0,
            respawn_timer: 0.0,
            camera_box,
            passive_effects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerAttributes {
    pub head_threshold: f32,
    pub legs_threshold: f32,
    pub weapon_mount: Vec2,
    pub jump_force: f32,
    pub move_speed: f32,
    pub slide_speed: f32,
    pub incapacitation_duration: f32,
    pub float_gravity_factor: f32,
    base_jump_force: f32,
    base_move_speed: f32,
    slide_speed_factor: f32,
}

impl PlayerAttributes {
    pub fn clear_mods(&mut self) {
        self.jump_force = self.base_jump_force;
        self.move_speed = self.base_move_speed;
        self.slide_speed = self.move_speed * self.slide_speed_factor;
    }

    pub fn apply_mods(&mut self, effect: &PassiveEffect) {
        if let Some(factor) = effect.move_speed_factor {
            self.move_speed *= factor;
        }
        if let Some(factor) = effect.jump_force_factor {
            self.jump_force *= factor;
        }
        if let Some(factor) = effect.slide_speed_factor {
            self.slide_speed *= factor;
        }
    }
}

impl From<&CharacterMetadata> for PlayerAttributes {
    fn from(params: &CharacterMetadata) -> Self {
        PlayerAttributes {
            head_threshold: params.head_threshold,
            legs_threshold: params.legs_threshold,
            weapon_mount: params.weapon_mount,
            base_jump_force: params.jump_force,
            jump_force: params.jump_force,
            base_move_speed: params.move_speed,
            move_speed: params.move_speed,
            slide_speed: params.move_speed * params.slide_speed_factor,
            incapacitation_duration: params.incapacitation_duration,
            float_gravity_factor: params.float_gravity_factor,
            slide_speed_factor: params.slide_speed_factor,
        }
    }
}

impl From<CharacterMetadata> for PlayerAttributes {
    fn from(params: CharacterMetadata) -> Self {
        PlayerAttributes::from(&params)
    }
}

pub fn spawn_player(
    world: &mut World,
    index: u8,
    position: Vec2,
    controller: PlayerControllerKind,
    character: CharacterMetadata,
) -> Entity {
    let weapon_mount = character.weapon_mount;
    let item_mount = character.item_mount;
    let hat_mount = character.hat_mount;

    let texture = get_texture(&character.sprite.texture_id);

    let offset = {
        let frame_size = texture.frame_size();
        character.sprite.offset
            - vec2(
                frame_size.width / 2.0,
                frame_size.height - character.collider_size.height,
            )
    };

    let animations = character
        .sprite
        .animations
        .to_vec()
        .into_iter()
        .map(|a| a.into())
        .collect::<Vec<_>>();

    let params = {
        let meta: AnimatedSpriteMetadata = character.sprite.clone().into();

        AnimatedSpriteParams {
            offset,
            ..meta.into()
        }
    };

    let sprites = vec![(
        BODY_ANIMATED_SPRITE_ID,
        AnimatedSprite::new(texture, texture.frame_size(), animations.as_slice(), params),
    )];

    let draw_order = (index as u32 + 1) * 10;

    let actor = physics_world().add_actor(position, character.collider_size);

    let body_params = PhysicsBodyParams {
        offset: vec2(-character.collider_size.width / 2.0, 0.0),
        size: character.collider_size,
        has_friction: false,
        can_rotate: false,
        gravity: character.gravity,
        ..Default::default()
    };

    world.spawn((
        Player::new(index, position),
        Transform::from(position),
        PlayerController::from(controller),
        PlayerAttributes::from(&character),
        PlayerInventory::new(weapon_mount, item_mount, hat_mount),
        PlayerEventQueue::new(),
        Drawable::new_animated_sprite_set(draw_order, &sprites),
        PhysicsBody::new(actor, None, body_params),
    ))
}
