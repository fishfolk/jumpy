//! Things available to spawn from the level editor
//! Proto-mods, eventually some of the items will move to some sort of a wasm runtime

use hecs::{Entity, World};
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{
    json, AnimatedSprite, AnimatedSpriteMetadata, AnimatedSpriteSet, CollisionWorld, DrawOrder,
    PassiveEffectMetadata, PhysicsBody, Resources, Transform,
};

mod weapon;

pub use weapon::*;

use crate::particles::ParticleEmitter;
use crate::physics::PhysicsBodyParams;
use crate::Result;

pub const ITEMS_DRAW_ORDER: u32 = 1;

pub const SPRITE_ANIMATED_SPRITE_ID: &str = "sprite";
pub const EFFECT_ANIMATED_SPRITE_ID: &str = "effect";

pub const GROUND_ANIMATION_ID: &str = "ground";
pub const ATTACK_ANIMATION_ID: &str = "attack";

/// This dictates what happens to an item when it is dropped, either manually or on death.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemDropBehavior {
    /// Clear all state and restore default parameters
    ClearState,
    /// Keep all state between pickups
    PersistState,
    /// Destroy the item on drop
    Destroy,
}

impl Default for ItemDropBehavior {
    fn default() -> Self {
        ItemDropBehavior::ClearState
    }
}

/// This dictates what happens to an item when it is depleted, either by exceeding its duration,
/// in the case of `Equipment`, or by depleting `uses`, if specified, in the case of a `Weapon`
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemDepleteBehavior {
    /// Keep the item on depletion (do nothing)
    Keep,
    /// Drop the item on depletion
    Drop,
    /// Destroy the item on depletion
    Destroy,
}

impl Default for ItemDepleteBehavior {
    fn default() -> Self {
        ItemDepleteBehavior::Keep
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MapItemKind {
    Weapon {
        #[serde(flatten)]
        meta: WeaponMetadata,
    },
    Item {
        #[serde(flatten)]
        meta: ItemMetadata,
    },
}

pub struct ItemParams {
    pub name: String,
    pub effects: Vec<PassiveEffectMetadata>,
    pub uses: Option<u32>,
    pub duration: Option<f32>,
    pub mount_offset: Vec2,
    pub drop_behavior: ItemDropBehavior,
    pub deplete_behavior: ItemDepleteBehavior,
}

pub struct Item {
    pub id: String,
    pub name: String,
    pub effects: Vec<PassiveEffectMetadata>,
    pub uses: Option<u32>,
    pub duration: Option<f32>,
    pub mount_offset: Vec2,
    pub drop_behavior: ItemDropBehavior,
    pub deplete_behavior: ItemDepleteBehavior,
    pub duration_timer: f32,
    pub use_cnt: u32,
}

impl Item {
    pub fn new(id: &str, params: ItemParams) -> Self {
        Item {
            id: id.to_string(),
            name: params.name,
            effects: params.effects,
            uses: params.uses,
            duration: params.duration,
            mount_offset: params.mount_offset,
            drop_behavior: params.drop_behavior,
            deplete_behavior: params.deplete_behavior,
            duration_timer: 0.0,
            use_cnt: 0,
        }
    }
}

/// This holds the parameters used when constructing an `Equipment`
#[derive(Clone, Serialize, Deserialize)]
pub struct ItemMetadata {
    /// The effects that will be instantiated when the item is equipped
    #[serde(default)]
    pub effects: Vec<PassiveEffectMetadata>,
    /// The items duration, after being equipped. This will also be the default duration of
    /// passive effects that are added to the player, when equipping the item
    #[serde(default)]
    pub duration: Option<f32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MapItemMetadata {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub kind: MapItemKind,
    #[serde(with = "json::vec2_def")]
    pub collider_size: Vec2,
    #[serde(default, with = "json::vec2_def")]
    pub collider_offset: Vec2,
    #[serde(default)]
    pub uses: Option<u32>,
    #[serde(default)]
    pub drop_behavior: ItemDropBehavior,
    #[serde(default)]
    pub deplete_behavior: ItemDepleteBehavior,
    /// This specifies the offset from the player position to where the equipped item is drawn
    #[serde(default, with = "json::vec2_def")]
    pub mount_offset: Vec2,
    /// The parameters for the `AnimationPlayer` that will be used to draw the item
    #[serde(alias = "animation")]
    pub sprite: AnimatedSpriteMetadata,
}

pub fn spawn_item(world: &mut World, position: Vec2, meta: MapItemMetadata) -> Result<Entity> {
    let mut sprites = Vec::new();

    let MapItemMetadata {
        collider_size,
        collider_offset,
        drop_behavior,
        deplete_behavior,
        mount_offset,
        ..
    } = meta;

    let actor = storage::get_mut::<CollisionWorld>().add_actor(
        position,
        collider_size.x as i32,
        collider_size.y as i32,
    );

    let res = storage::get::<Resources>()
        .textures
        .get(&meta.sprite.texture_id)
        .cloned();

    if let Some(texture_res) = res {
        let (texture, frame_size) = (texture_res.texture, texture_res.frame_size());
        let animations = meta
            .sprite
            .animations
            .clone()
            .into_iter()
            .map(|a| a.into())
            .collect::<Vec<_>>();

        let params = meta.sprite.into();

        let sprite = AnimatedSprite::new(texture, frame_size, animations.as_slice(), params);
        sprites.push((SPRITE_ANIMATED_SPRITE_ID, sprite));
    }

    let id = meta.id.as_str();

    let entity = world.spawn((
        Transform::from(position),
        PhysicsBody::new(
            actor,
            None,
            PhysicsBodyParams {
                size: collider_size,
                offset: collider_offset,
                has_mass: true,
                has_friction: true,
                can_rotate: false,
                ..Default::default()
            },
        ),
        DrawOrder(ITEMS_DRAW_ORDER),
    ));

    let uses = meta.uses;

    let name = meta.name.clone();

    match meta.kind {
        MapItemKind::Item { meta } => {
            let ItemMetadata { effects, duration } = meta;

            world.insert_one(
                entity,
                Item::new(
                    id,
                    ItemParams {
                        name,
                        effects,
                        uses,
                        duration,
                        mount_offset,
                        drop_behavior,
                        deplete_behavior,
                    },
                ),
            )?;

            if !sprites.is_empty() {
                world.insert_one(entity, AnimatedSpriteSet::from(sprites))?;
            }
        }
        MapItemKind::Weapon { meta } => {
            let effect_offset = meta.effect_offset;

            let mut sound_effect = None;
            if let Some(id) = meta.sound_effect_id.as_ref() {
                sound_effect = storage::get::<Resources>().sounds.get(id).copied();
            }

            if let Some(effect_sprite) = meta.effect_sprite {
                if let Some(texture_res) = storage::get::<Resources>()
                    .textures
                    .get(&effect_sprite.texture_id)
                {
                    let animations = effect_sprite
                        .animations
                        .clone()
                        .into_iter()
                        .map(|a| a.into())
                        .collect::<Vec<_>>();

                    let params = effect_sprite.into();

                    let mut sprite = AnimatedSprite::new(
                        texture_res.texture,
                        texture_res.frame_size(),
                        animations.as_slice(),
                        params,
                    );

                    sprite.is_deactivated = true;

                    sprites.push((EFFECT_ANIMATED_SPRITE_ID, sprite));
                }
            }

            let particle_emitters = meta
                .particles
                .clone()
                .into_iter()
                .map(ParticleEmitter::new)
                .collect::<Vec<_>>();

            if !particle_emitters.is_empty() {
                world.insert_one(entity, particle_emitters).unwrap();
            }

            let params = WeaponParams {
                name,
                effects: meta.effects,
                uses,
                sound_effect,
                mount_offset,
                effect_offset,
                drop_behavior,
                deplete_behavior,
            };

            world.insert_one(
                entity,
                Weapon::new(id, meta.recoil, meta.cooldown, meta.attack_duration, params),
            )?;

            if !sprites.is_empty() {
                world.insert_one(entity, AnimatedSpriteSet::from(sprites))?;
            }
        }
    }

    Ok(entity)
}
