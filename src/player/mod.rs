use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use hecs::{Entity, World};

use crate::{
    AnimatedSprite, AnimatedSpriteMetadata, AnimatedSpriteParams, CollisionWorld, Drawable,
    PhysicsBody, Resources, Transform,
};

mod animation;
mod character;
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

use crate::physics::PhysicsBodyParams;

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

pub const JUMP_SOUND_ID: &str = "jump";
pub const LAND_SOUND_ID: &str = "land";

pub const RESPAWN_DELAY: f32 = 2.5;
pub const PICKUP_GRACE_TIME: f32 = 0.25;

#[derive(Debug, Clone)]
pub struct PlayerParams {
    pub index: u8,
    pub controller: PlayerControllerKind,
    pub character: PlayerCharacterMetadata,
}

#[derive(Debug, Copy, Clone)]
pub struct Player(pub u8);

#[derive(Debug, Clone)]
pub struct PlayerAttributes {
    pub head_threshold: f32,
    pub legs_threshold: f32,
    pub weapon_mount: Vec2,
    pub jump_force: f32,
    pub move_speed: f32,
    pub slide_speed_factor: f32,
    pub incapacitation_duration: f32,
    pub float_gravity_factor: f32,
}

impl From<&PlayerCharacterMetadata> for PlayerAttributes {
    fn from(params: &PlayerCharacterMetadata) -> Self {
        PlayerAttributes {
            head_threshold: params.head_threshold,
            legs_threshold: params.legs_threshold,
            weapon_mount: params.weapon_mount,
            jump_force: params.jump_force,
            move_speed: params.move_speed,
            slide_speed_factor: params.slide_speed_factor,
            incapacitation_duration: params.incapacitation_duration,
            float_gravity_factor: params.float_gravity_factor,
        }
    }
}

impl From<PlayerCharacterMetadata> for PlayerAttributes {
    fn from(params: PlayerCharacterMetadata) -> Self {
        PlayerAttributes::from(&params)
    }
}

pub struct PlayerEffects {
    pub passive_effects: Vec<Entity>,
}

pub fn spawn_player(
    world: &mut World,
    index: u8,
    position: Vec2,
    controller: PlayerControllerKind,
    character: PlayerCharacterMetadata,
) -> Entity {
    let weapon_mount = character.weapon_mount;

    let offset = storage::get::<Resources>()
        .textures
        .get(&character.sprite.texture_id)
        .map(|t| {
            let frame_size = t.frame_size();
            character.sprite.offset
                - vec2(frame_size.x / 2.0, frame_size.y - character.collider_size.y)
        })
        .unwrap();

    let animations = character
        .sprite
        .animations
        .to_vec()
        .into_iter()
        .map(|a| a.into())
        .collect::<Vec<_>>();

    let texture_id = character.sprite.texture_id.clone();

    let params = {
        let meta: AnimatedSpriteMetadata = character.sprite.clone().into();

        AnimatedSpriteParams {
            offset,
            ..meta.into()
        }
    };

    let sprites = vec![(
        BODY_ANIMATED_SPRITE_ID,
        AnimatedSprite::new(&texture_id, animations.as_slice(), params),
    )];

    let draw_order = (index as u32 + 1) * 10;

    let size = character.collider_size.as_i32();
    let actor = storage::get_mut::<CollisionWorld>().add_actor(position, size.x, size.y);

    let body_params = PhysicsBodyParams {
        offset: vec2(-character.collider_size.x / 2.0, 0.0),
        size: character.collider_size,
        has_friction: false,
        can_rotate: false,
        ..Default::default()
    };

    world.spawn((
        Player(index),
        Transform::from(position),
        PlayerController::from(controller),
        PlayerAttributes::from(&character),
        PlayerState::from(position),
        PlayerInventory::from(weapon_mount),
        PlayerEventQueue::new(),
        Drawable::new_animated_sprite_set(draw_order, &sprites),
        PhysicsBody::new(actor, None, body_params),
    ))
}
