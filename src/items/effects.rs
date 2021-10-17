use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use crate::json;

mod projectiles;

pub use projectiles::Projectiles;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectDeliveryTrigger {
    Player,
    Ground,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "effect_delivery", rename_all = "snake_case")]
pub enum EffectDelivery {
    // Cone that delivers the effect to anyone that passes its collision check.
    // This would typically be used for things like a flame thrower or any melee weapons.
    Cone {
        #[serde(default, rename = "cone_direction", with = "json::vec2_def")]
        direction: Vec2,
        #[serde(rename = "cone_length")]
        length: f32,
        #[serde(rename = "cone_angle")]
        angle: f32,
    },
    // Projectile should be used for anything that spawns an object that delivers an effect on hit.
    // This would typically be used for things like a gun.
    Projectile {
        #[serde(rename = "projectile_range")]
        range: f32,
        #[serde(rename = "projectile_speed")]
        speed: f32,
        #[serde(default, rename = "projectile_spread")]
        spread: f32,
        #[serde(rename = "projectile_size")]
        size: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "effect", rename_all = "snake_case")]
pub enum Effect {
    DirectDamage,
    AreaDamage {
        #[serde(default, rename = "aoe_force")]
        force: f32,
        #[serde(default, rename = "aoe_is_single_target")]
        is_single_target: bool,
    },
}
