mod effects;
mod projectiles;

use macroquad::{
    experimental::{
        scene::{
            Handle,
        },
        animation::{
            Animation,
            AnimatedSprite,
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
    json,
};

pub use effects::{
    EffectDeliveryKind,
    Effect,
};

pub use projectiles::Projectiles;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponParams {
    pub id: String,
    pub cooldown: f32,
    pub damage: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    pub effect: Effect,
    pub effect_delivery: EffectDeliveryKind,
    pub animation: AnimationParams,
}

#[derive(Clone)]
pub struct Weapon {
    pub id: String,
    pub cooldown: f32,
    pub damage: f32,
    pub uses: Option<u32>,
    pub effect: Effect,
    pub effect_delivery: EffectDeliveryKind,
    pub sprite: AnimatedSprite,
    use_cnt: u32,
    cooldown_timer: f32,
}

impl Weapon {
    pub fn new(params: WeaponParams) -> Self {
        let sprite = AnimatedSprite::new(
            params.animation.tile_size.x,
            params.animation.tile_size.y,
            &params.animation.animations,
            false,
        );

        Weapon {
            id: params.id,
            cooldown: params.cooldown,
            damage: params.damage,
            uses: params.uses,
            effect: params.effect,
            effect_delivery: params.effect_delivery,
            sprite,
            use_cnt: 0,
            cooldown_timer: params.cooldown,
        }
    }

    pub fn update(&mut self) {
        self.sprite.update();

        if self.cooldown_timer < self.cooldown {
            self.cooldown_timer += get_frame_time();
        }
    }

    pub fn attack(&mut self, player: Handle<Player>, position: Vec2, direction: Vec2) {
        if self.cooldown >= self.cooldown_timer {
            if let Some(uses) = self.uses {
                if self.use_cnt >= uses {
                    return;
                } else {
                    self.use_cnt += 1;
                }
            }

            self.cooldown_timer = 0.0;


        }
    }
}