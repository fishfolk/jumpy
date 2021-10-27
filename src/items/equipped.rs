use macroquad::{
    experimental::{
        scene::RefMut,
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

use crate::{
    components::{AnimationParams, AnimationPlayer},
    json::OneOrMany,
    Player,
};

use crate::effects::{PassiveEffect, PassiveEffectParams, AnyEffectParams, active_effect_coroutine};

/// This holds the parameters used when constructing an `EquippedItem`
#[derive(Clone, Serialize, Deserialize)]
pub struct EquippedItemParams {
    /// The effects that will be instantiated when the item is equipped
    pub effects: OneOrMany<AnyEffectParams>,
    /// The parameters for the `AnimationPlayer` that will be used to draw the item when it is
    /// equipped by a player.
    #[serde(default)]
    pub animation: Option<AnimationParams>,
    /// The items duration, after being equipped. This will also be the default duration of
    /// passive effects that are added to the player, when equipping the item
    #[serde(default)]
    pub duration: Option<f32>,
}

#[allow(dead_code)]
pub struct EquippedItem {
    id: String,
    sprite_animation: Option<AnimationPlayer>,
    duration: Option<f32>,
    duration_timer: f32,
}

impl EquippedItem {
    #[allow(dead_code)]
    pub fn new(id: &str, params: EquippedItemParams, player: &mut RefMut<Player>) -> Self {
        for params in params.effects.into_vec() {
            match params {
                AnyEffectParams::Active(params) => {
                    active_effect_coroutine(player.handle(), player.body.position, params);
                }
                AnyEffectParams::Passive(params) => {
                    player.add_passive_effect(params);
                }
            }
        }

        let sprite_animation = params.animation
            .map(|params| AnimationPlayer::new(params));

        EquippedItem {
            id: id.to_string(),
            sprite_animation,
            duration: params.duration,
            duration_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(sprite) = &mut self.sprite_animation {
            sprite.update();
        }

        self.duration_timer += dt;
    }

    pub fn draw(&self, position: Vec2, rotation: f32, flip_x: bool, flip_y: bool) {
        if let Some(sprite) = &self.sprite_animation {
            sprite.draw(position, rotation, flip_x, flip_y);
        }
    }

    pub fn is_depleted(&self) -> bool {
        if let Some(duration) = self.duration {
            if self.duration_timer >= duration {
                return true;
            }
        }

        false
    }
}
