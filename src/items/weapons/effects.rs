use macroquad::{
    experimental::{
        scene::{
            Node,
            Handle,
            HandleUntyped,
            RefMut,
        },
    },
    prelude::*,
};

use serde::{
    Serialize,
    Deserialize,
};

use crate::{
    Player,
    capabilities::NetworkReplicate,
    json,
};

use super::AnimationParams;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectDeliveryTrigger {
    Player,
    Ground,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EffectDeliveryKind {
    // Cone that delivers the effect to anyone that passes its collision check.
    // This would typically be used for things like a flame thrower or any melee weapons.
    Cone {
        #[serde(with = "json::def_vec2")]
        direction: Vec2,
        range: f32,
        angle: f32,
    },
    // Projectile should be used for anything that spawns an object that delivers an effect on hit.
    // This would typically be used for things like a gun.
    Projectile {
        speed: f32,
        spread: f32,
        size: f32,
    },
    // PhysicsBody will spawn a new object that has physics. It requires either a `timer` or a
    // `trigger`, or both, to function.
    // This would typically be used for things like grenades.
    PhysicsBody {
        texture_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        animation_params: Option<AnimationParams>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timer: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trigger: Option<EffectDeliveryTrigger>,
    }
}

#[derive(Debug, Clone)]
pub struct EffectDelivery {
    pub owner: Handle<Player>,
    pub position: Vec2,
    pub kind: EffectDeliveryKind,
    pub effect: Effect,
    pub ttl_timer: f32,
    pub is_finished: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
    DirectDamage {
        damage: f32,
    },
    AreaDamage {
        damage: f32,
        force: f32,
        is_single_target: bool,
    },
}