use macroquad::{
    audio::play_sound_once,
    audio::Sound,
    experimental::{
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::Handle,
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

use crate::{
    components::{AnimationParams, AnimationPlayer},
    json, Player, Resources,
};

pub mod effects;

pub use effects::{
    add_custom_weapon_effect, get_custom_weapon_effect, weapon_effect_coroutine,
    CustomWeaponEffectCoroutine, CustomWeaponEffectParam, Projectiles, WeaponEffectKind,
    WeaponEffectParams, WeaponEffectTriggerKind,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponParams {
    #[serde(
        default,
        rename = "sound_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub sound_effect_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    pub effect: WeaponEffectParams,
    #[serde(default, with = "json::vec2_def")]
    pub mount_offset: Vec2,
    #[serde(default, with = "json::vec2_def")]
    pub effect_offset: Vec2,
    #[serde(default)]
    pub attack_duration: f32,
    pub cooldown: f32,
    #[serde(default)]
    pub recoil: f32,
    pub animation: AnimationParams,
}

pub struct Weapon {
    pub id: String,
    pub sound_effect: Option<Sound>,
    pub effect: WeaponEffectParams,
    pub cooldown: f32,
    pub recoil: f32,
    pub attack_duration: f32,
    pub uses: Option<u32>,
    pub use_cnt: u32,
    pub animation_player: AnimationPlayer,
    pub cooldown_timer: f32,
    pub mount_offset: Vec2,
    pub effect_offset: Vec2,
}

impl Weapon {
    const HUD_CONDENSED_USE_COUNT_THRESHOLD: u32 = 12;

    const HUD_USE_COUNT_COLOR_FULL: Color = Color {
        r: 0.8,
        g: 0.9,
        b: 1.0,
        a: 1.0,
    };

    const HUD_USE_COUNT_COLOR_EMPTY: Color = Color {
        r: 0.8,
        g: 0.9,
        b: 1.0,
        a: 0.8,
    };

    const IDLE_ANIMATION_ID: &'static str = "idle";
    const ATTACK_ANIMATION_ID: &'static str = "attack";

    pub fn new(id: &str, params: WeaponParams) -> Self {
        let sound_effect = if let Some(sound_effect_id) = &params.sound_effect_id {
            let resources = storage::get::<Resources>();
            let sound_effect = resources
                .sounds
                .get(sound_effect_id)
                .copied()
                .unwrap_or_else(|| panic!("Weapon: Invalid sound effect ID '{}'", sound_effect_id));
            Some(sound_effect)
        } else {
            None
        };

        {
            let res = params
                .animation
                .animations
                .iter()
                .find(|a| a.id == Self::IDLE_ANIMATION_ID);

            assert!(
                res.is_some(),
                "Weapon: An animation with id '{}' is required",
                Self::IDLE_ANIMATION_ID
            );
        }

        let animation_player = AnimationPlayer::new(AnimationParams {
            pivot: Some(params.mount_offset),
            ..params.animation
        });

        Weapon {
            id: id.to_string(),
            sound_effect,
            effect: params.effect,
            cooldown: params.cooldown,
            recoil: params.recoil,
            attack_duration: params.attack_duration,
            uses: params.uses,
            use_cnt: 0,
            animation_player,
            cooldown_timer: params.cooldown,
            mount_offset: params.mount_offset,
            effect_offset: params.effect_offset,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.animation_player.update();

        self.cooldown_timer += dt;
    }

    pub fn draw(&mut self, position: Vec2, rotation: f32, flip_x: bool, flip_y: bool) {
        let size = self.animation_player.get_size();
        let mut position = position;

        if flip_x {
            position.x -= size.x;
        }

        if flip_y {
            position.y -= size.y;
        }

        self.animation_player
            .draw(position, rotation, flip_x, flip_y);
    }

    pub fn draw_hud(&self, position: Vec2) {
        if let Some(uses) = self.uses {
            let remaining = uses - self.use_cnt;

            if uses >= Self::HUD_CONDENSED_USE_COUNT_THRESHOLD {
                let x = position.x - ((4.0 * uses as f32) / 2.0);

                for i in 0..uses {
                    let x = x + 4.0 * i as f32;

                    if i >= remaining {
                        draw_rectangle(
                            x,
                            position.y - 12.0,
                            2.0,
                            12.0,
                            Self::HUD_USE_COUNT_COLOR_EMPTY,
                        );
                    } else {
                        draw_rectangle(
                            x,
                            position.y - 12.0,
                            2.0,
                            12.0,
                            Self::HUD_USE_COUNT_COLOR_FULL,
                        );
                    };
                }
            } else {
                let x = position.x - (uses as f32 * 14.0) / 2.0;

                for i in 0..uses {
                    let x = x + 14.0 * i as f32;

                    if i >= remaining {
                        draw_circle_lines(
                            x,
                            position.y - 4.0,
                            4.0,
                            2.0,
                            Self::HUD_USE_COUNT_COLOR_EMPTY,
                        );
                    } else {
                        draw_circle(x, position.y - 4.0, 4.0, Self::HUD_USE_COUNT_COLOR_FULL);
                    };
                }
            }
        }
    }

    fn is_ready(&self) -> bool {
        if self.cooldown_timer < self.cooldown {
            return false;
        } else if let Some(uses) = self.uses {
            if self.use_cnt >= uses {
                return false;
            }
        }

        true
    }

    fn animation_coroutine(player_handle: Handle<Player>, animation_id: &str) -> Coroutine {
        let animation_id = animation_id.to_string();

        let coroutine = async move {
            let mut animation = None;

            {
                let player = &mut *scene::get_node(player_handle);
                if let Some(weapon) = &mut player.weapon {
                    animation = weapon
                        .animation_player
                        .set_animation(&animation_id)
                        .cloned();

                    if animation.is_some() {
                        weapon.animation_player.stop();
                    }
                }
            }

            if let Some(animation) = animation {
                let frame_interval = 1.0 / animation.fps as f32;

                for i in 0..animation.frames as usize {
                    {
                        let player = &mut *scene::get_node(player_handle);
                        if let Some(weapon) = player.weapon.as_mut() {
                            weapon.animation_player.set_frame(i);
                        }
                    }

                    wait_seconds(frame_interval).await;
                }

                {
                    let player = &mut *scene::get_node(player_handle);
                    if let Some(weapon) = player.weapon.as_mut() {
                        weapon
                            .animation_player
                            .set_animation(Self::IDLE_ANIMATION_ID);
                        weapon.animation_player.play();
                    }
                }
            }
        };

        start_coroutine(coroutine)
    }

    /// This will start a `Coroutine` that performs an attack with the `Weapon` equipped by the
    /// `Player` fetched with `player_handle`, id one is equipped and ready for use.
    pub fn attack_coroutine(player_handle: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            let is_ready = {
                let player = &mut *scene::get_node(player_handle);
                if let Some(weapon) = &player.weapon {
                    weapon.is_ready()
                } else {
                    false
                }
            };

            if is_ready {
                {
                    let player = &mut *scene::get_node(player_handle);
                    if let Some(weapon) = player.weapon.as_mut() {
                        if weapon.uses.is_some() {
                            weapon.use_cnt += 1;
                        }

                        weapon.cooldown_timer = 0.0;

                        if let Some(sound_effect) = weapon.sound_effect {
                            play_sound_once(sound_effect);
                        }

                        player.body.velocity.x = if player.body.is_facing_right {
                            -weapon.recoil
                        } else {
                            weapon.recoil
                        };
                    } else {
                        return;
                    }
                }

                {
                    let player = &mut *scene::get_node(player_handle);
                    let origin = player.body.pos + player.get_weapon_effect_offset();

                    if let Some(weapon) = player.weapon.as_mut() {
                        weapon_effect_coroutine(player_handle, origin, weapon.effect.clone());
                    }
                }

                {
                    Weapon::animation_coroutine(player_handle, Self::ATTACK_ANIMATION_ID);
                }

                let attack_duration = {
                    let player = &*scene::get_node(player_handle);
                    player.weapon.as_ref().map(|weapon| weapon.attack_duration)
                };

                if let Some(attack_duration) = attack_duration {
                    wait_seconds(attack_duration).await;
                }
            }

            {
                let player = &mut *scene::get_node(player_handle);
                player.state_machine.set_state(Player::ST_NORMAL);
            }
        };

        start_coroutine(coroutine)
    }
}
