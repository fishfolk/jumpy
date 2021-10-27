use macroquad::{experimental::scene::RefMut, prelude::*};

use serde::{Deserialize, Serialize};

use crate::{
    components::{AnimationParams, AnimationPlayer},
    json::{self, OneOrMany},
    Player,
};

use crate::effects::{active_effect_coroutine, AnyEffectParams};

/// This holds the parameters used when constructing an `EquippedItem`
#[derive(Clone, Serialize, Deserialize)]
pub struct EquippedItemParams {
    /// The effects that will be instantiated when the item is equipped
    pub effects: OneOrMany<AnyEffectParams>,
    /// This specifies the offset from the player position to where the equipped item is drawn
    #[serde(default, with = "json::vec2_def")]
    pub mount_offset: Vec2,
    /// The parameters for the `AnimationPlayer` that will be used to draw the item when it is
    /// equipped by a player.
    #[serde(default)]
    pub animation: Option<AnimationParams>,
    /// The items duration, after being equipped. This will also be the default duration of
    /// passive effects that are added to the player, when equipping the item
    #[serde(default)]
    pub duration: Option<f32>,
    /// If this is true the item will be dropped if the player holding it dies
    #[serde(default)]
    pub is_dropped_on_death: bool,
}

#[allow(dead_code)]
pub struct EquippedItem {
    pub id: String,
    mount_offset: Vec2,
    sprite_animation: Option<AnimationPlayer>,
    duration: Option<f32>,
    duration_timer: f32,
    pub is_dropped_on_death: bool,
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

        let sprite_animation = params.animation.map(AnimationPlayer::new);

        EquippedItem {
            id: id.to_string(),
            mount_offset: params.mount_offset,
            sprite_animation,
            duration: params.duration,
            duration_timer: 0.0,
            is_dropped_on_death: params.is_dropped_on_death,
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
            let size = sprite.get_size();
            let mut offset = Vec2::ZERO;

            if flip_x {
                offset.x = -(self.mount_offset.x + size.x);
            } else {
                offset.x = self.mount_offset.x;
            }

            if flip_y {
                offset.y = -(self.mount_offset.y + size.y);
            } else {
                offset.y = self.mount_offset.y;
            }

            sprite.draw(position + offset, rotation, flip_x, flip_y);
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
