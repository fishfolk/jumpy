use macroquad::audio::{play_sound_once, Sound};
use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use hecs::{World, Entity};

use crate::{AnimatedSpriteSet, PhysicsBody, Result};
use crate::effects::ActiveEffectMetadata;
use crate::items::{ATTACK_ANIMATION_ID, EFFECT_ANIMATED_SPRITE_ID, ItemDepleteBehavior, ItemDropBehavior, SPRITE_ANIMATED_SPRITE_ID};
use crate::{json, QueuedAnimationAction, Transform};
use crate::particles::{ParticleEmitter, ParticleEmitterParams};
use crate::AnimatedSpriteMetadata;
use crate::effects::active::spawn_active_effect;
use crate::player::{IDLE_ANIMATION_ID, PlayerInventory, PlayerState};

pub struct WeaponParams {
    pub effects: Vec<ActiveEffectMetadata>,
    pub uses: Option<u32>,
    pub sound_effect: Option<Sound>,
    pub mount_offset: Vec2,
    pub effect_offset: Vec2,
    pub drop_behavior: ItemDropBehavior,
    pub deplete_behavior: ItemDepleteBehavior,
}

impl Default for WeaponParams {
    fn default() -> Self {
        WeaponParams {
            effects: Vec::new(),
            uses: None,
            sound_effect: None,
            mount_offset: Vec2::ZERO,
            effect_offset: Vec2::ZERO,
            drop_behavior: Default::default(),
            deplete_behavior: Default::default(),
        }
    }
}

pub struct Weapon {
    pub id: String,
    pub effects: Vec<ActiveEffectMetadata>,
    pub sound_effect: Option<Sound>,
    pub recoil: f32,
    pub cooldown: f32,
    pub attack_duration: f32,
    pub uses: Option<u32>,
    pub mount_offset: Vec2,
    pub effect_offset: Vec2,
    pub drop_behavior: ItemDropBehavior,
    pub deplete_behavior: ItemDepleteBehavior,
    pub cooldown_timer: f32,
    pub use_cnt: u32,
}

impl Weapon {
    pub fn new(id: &str, recoil: f32, cooldown: f32, attack_duration: f32, params: WeaponParams) -> Self {
        Weapon {
            id: id.to_string(),
            effects: params.effects,
            recoil,
            cooldown,
            uses: params.uses,
            attack_duration,
            sound_effect: params.sound_effect,
            mount_offset: params.mount_offset,
            effect_offset: params.effect_offset,
            drop_behavior: params.drop_behavior,
            deplete_behavior: params.deplete_behavior,
            cooldown_timer: cooldown,
            use_cnt: 0,
        }
    }
}

pub fn fire_weapon(world: &mut World, entity: Entity, owner: Entity) -> Result<()> {
    let mut effects = Vec::new();

    let mut origin = Vec2::ZERO;

    {
        let mut weapon = world.get_mut::<Weapon>(entity).unwrap();

        if weapon.cooldown_timer >= weapon.cooldown {
            let mut owner_state = world.get_mut::<PlayerState>(owner).unwrap();

            {
                let mut owner_body = world.get_mut::<PhysicsBody>(owner).unwrap();

                if owner_state.is_facing_left {
                    owner_body.velocity.x = weapon.recoil;
                } else {
                    owner_body.velocity.x = -weapon.recoil;
                }

                let owner_transform = world.get::<Transform>(owner).unwrap();
                let owner_inventory = world.get::<PlayerInventory>(owner).unwrap();

                origin = owner_transform.position
                    + owner_inventory.get_weapon_mount(owner_state.is_facing_left, owner_state.is_upside_down);

                let mut offset = weapon.mount_offset + weapon.effect_offset;
                if owner_state.is_facing_left {
                    offset.x = -offset.x;
                }

                origin += offset;
            }

            owner_state.attack_timer = weapon.attack_duration;

            weapon.use_cnt += 1;

            weapon.cooldown_timer = 0.0;

            if let Some(sound) = weapon.sound_effect {
                play_sound_once(sound);
            }

            let mut sprite_set = world.get_mut::<AnimatedSpriteSet>(entity).unwrap();

            if let Some(sprite) = sprite_set.map.get_mut(SPRITE_ANIMATED_SPRITE_ID) {
                let is_looping = sprite
                    .get_animation(ATTACK_ANIMATION_ID)
                    .map(|a| a.is_looping)
                    .unwrap_or_default();

                sprite.set_animation(ATTACK_ANIMATION_ID, !is_looping);
                sprite.queue_action(QueuedAnimationAction::Play(IDLE_ANIMATION_ID.to_string()));
            }

            if let Some(sprite) = sprite_set.map.get_mut(EFFECT_ANIMATED_SPRITE_ID) {
                sprite.is_deactivated = false;

                let is_looping = sprite
                    .get_animation(ATTACK_ANIMATION_ID)
                    .map(|a| a.is_looping)
                    .unwrap_or_default();

                sprite.set_animation(ATTACK_ANIMATION_ID, !is_looping);
                sprite.queue_action(QueuedAnimationAction::Deactivate);
            }

            if let Ok(mut particle_emitters) = world.get_mut::<Vec<ParticleEmitter>>(entity) {
                for emitter in particle_emitters.iter_mut() {
                    emitter.activate();
                }
            }

            effects = weapon.effects.clone();
        }
    }

    for params in effects {
        spawn_active_effect(world, owner, origin, params)?;
    }

    Ok(())
}

/// This holds the parameters for the `AnimationPlayer` components of an equipped `Weapon`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponAnimationMetadata {
    /// This holds the parameters of the main `AnimationPlayer` component, holding the main
    /// animations, like `"idle"` and `"attack"`.
    /// At a minimum, an animation with the id `"idle"` must be specified. If no animation is
    /// required, an animation with one frame can be used to just display a sprite.
    #[serde(rename = "animation")]
    pub sprite: AnimatedSpriteMetadata,
    /// This can hold the parameters of the effect `AnimationPlayer` component, holding the
    /// animations used for effects like `"attack_effect"`.
    /// At a minimum, if this is specified, an animation with the id `"attack_effect"` must be
    /// specified. If no animation is required, an animation with one frame can be used to just
    /// display a sprite.
    #[serde(
    default,
    rename = "effect_animation",
    skip_serializing_if = "Option::is_none"
    )]
    pub effect: Option<AnimatedSpriteMetadata>,
}

/// This holds parameters specific to the `Weapon` variant of `ItemKind`, used to instantiate a
/// `Weapon` struct instance, when an `Item` of type `Weapon` is picked up.
#[derive(Clone, Serialize, Deserialize)]
pub struct WeaponMetadata {
    /// This specifies the effects to instantiate when the weapon is used to attack
    #[serde(default)]
    pub effects: Vec<ActiveEffectMetadata>,
    /// Particle effects that will be activated when using the weapon
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub particles: Vec<ParticleEmitterParams>,
    /// This can specify an id of a sound effect that is played when the weapon is used to attack
    #[serde(
    default,
    rename = "sound_effect",
    skip_serializing_if = "Option::is_none"
    )]
    pub sound_effect_id: Option<String>,
    /// This specifies the offset between the upper left corner of the weapon's sprite to the
    /// position that will serve as the origin of the weapon's effects
    #[serde(default, with = "json::vec2_def")]
    pub effect_offset: Vec2,
    /// This can specify a maximum amount of weapon uses. If no value is specified, the weapon
    /// will have unlimited uses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    /// This specifies the minimum interval of attacks with the weapon
    #[serde(default)]
    pub cooldown: f32,
    /// This specifies the amount of time the player will be locked in an attack state when using
    /// the weapon
    #[serde(default)]
    pub attack_duration: f32,
    /// This specifies the force applied to the `Player` velocity, in the opposite direction of the
    /// attack, when the weapon is activated.
    #[serde(default)]
    pub recoil: f32,
    /// This can hold the parameters of the effect `AnimationPlayer` component, holding the
    /// animations used for effects.
    /// At a minimum, if this is specified, an animation with the id `"attack"` must be
    /// specified.
    #[serde(
    default,
    skip_serializing_if = "Option::is_none"
    )]
    pub effect_sprite: Option<AnimatedSpriteMetadata>,
}

impl Default for WeaponMetadata {
    fn default() -> Self {
        WeaponMetadata {
            effects: Vec::new(),
            particles: Vec::new(),
            sound_effect_id: None,
            uses: None,
            effect_offset: Vec2::ZERO,
            cooldown: 0.0,
            attack_duration: 0.0,
            recoil: 0.0,
            effect_sprite: None,
        }
    }
}