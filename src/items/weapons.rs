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
    effects::{active_effect_coroutine, ActiveEffectParams},
    json::{self, OneOrMany},
    Player, Resources,
};

/// This holds the parameters for the `AnimationPlayer` components of an equipped `Weapon`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponAnimationParams {
    /// This holds the parameters of the main `AnimationPlayer` component, holding the main
    /// animations, like `"idle"` and `"attack"`.
    /// At a minimum, an animation with the id `"idle"` must be specified. If no animation is
    /// required, an animation with one frame can be used to just display a sprite.
    #[serde(rename = "animation")]
    pub sprite: AnimationParams,
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
    pub effect: Option<AnimationParams>,
}

/// This holds parameters specific to the `Weapon` variant of `ItemKind`, used to instantiate a
/// `Weapon` struct instance, when an `Item` of type `Weapon` is picked up.
#[derive(Clone, Serialize, Deserialize)]
pub struct WeaponParams {
    /// This specifies the effects to instantiate when the weapon is used to attack. Can be either
    /// a single `ActiveEffectParams` or a vector of `ActiveEffectParams`-
    pub effects: OneOrMany<ActiveEffectParams>,
    /// This can specify an id of a sound effect that is played when the weapon is used to attack
    #[serde(
        default,
        rename = "sound_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub sound_effect_id: Option<String>,
    /// This can specify a maximum amount of weapon uses. If no value is specified, the weapon
    /// will have unlimited uses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    /// If this is set to `true` the weapon will be destroyed when it is out of uses
    #[serde(default)]
    pub is_destroyed_on_depletion: bool,
    /// This specifies the offset from the `Player` weapon mount
    #[serde(default, with = "json::vec2_def")]
    pub mount_offset: Vec2,
    /// This specifies the offset between the upper left corner of the weapon's sprite to the
    /// position that will serve as the origin of the weapon's effects
    #[serde(default, with = "json::vec2_def")]
    pub effect_offset: Vec2,
    /// This specifies the duration of the weapons attack, ie. the amount of time the player will
    /// be locked in the attack state, before control is regained
    #[serde(default)]
    pub attack_duration: f32,
    /// This specifies the minimum interval of attacks with the weapon
    #[serde(default)]
    pub cooldown: f32,
    /// This specifies the force applied to the `Player` velocity, in the opposite direction of the
    /// attack, when the weapon is activated.
    #[serde(default)]
    pub recoil: f32,
    /// This holds the parameters for the `AnimationPlayer` components that will be used when
    /// the weapon is equipped by a player. It is flattened into this struct, so when defining
    /// weapons in JSON files, the members of `WeaponAnimationParams` will be treated as members
    /// of this struct.
    /// Note that the sprite that is used when the weapon is on the ground is specified on the
    /// `Item` node's `sprite` member.
    #[serde(flatten)]
    pub animation: WeaponAnimationParams,
}

pub struct Weapon {
    pub id: String,
    pub sound_effect: Option<Sound>,
    pub effects: Vec<ActiveEffectParams>,
    pub cooldown: f32,
    pub recoil: f32,
    pub attack_duration: f32,
    pub uses: Option<u32>,
    pub sprite_animation: AnimationPlayer,
    pub effect_animation: Option<AnimationPlayer>,
    pub cooldown_timer: f32,
    pub mount_offset: Vec2,
    pub effect_offset: Vec2,
    is_destroyed_on_depletion: bool,
    use_cnt: u32,
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
    const ATTACK_EFFECT_ANIMATION_ID: &'static str = "attack_effect";

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
                .sprite
                .animations
                .iter()
                .find(|a| a.id == Self::IDLE_ANIMATION_ID);

            assert!(
                res.is_some(),
                "Weapon: An animation with id '{}' is required",
                Self::IDLE_ANIMATION_ID
            );
        }

        let sprite_animation = AnimationPlayer::new(params.animation.sprite);

        let mut effect_animation = None;
        if let Some(animation_params) = params.animation.effect {
            let animation_player = AnimationPlayer::new(AnimationParams {
                is_deactivated: true,
                ..animation_params
            });

            effect_animation = Some(animation_player);
        }

        Weapon {
            id: id.to_string(),
            sound_effect,
            effects: params.effects.into(),
            cooldown: params.cooldown,
            recoil: params.recoil,
            attack_duration: params.attack_duration,
            uses: params.uses,
            sprite_animation,
            effect_animation,
            cooldown_timer: params.cooldown,
            mount_offset: params.mount_offset,
            effect_offset: params.effect_offset,
            is_destroyed_on_depletion: params.is_destroyed_on_depletion,
            use_cnt: 0,
        }
    }

    fn get_mount_offset(&self, flip_x: bool, flip_y: bool) -> Vec2 {
        let size = self.sprite_animation.get_size();
        let mut offset = Vec2::ZERO;

        if flip_x {
            offset.x = -(self.mount_offset.x + size.x);
        } else {
            offset.x = self.mount_offset.x
        }

        if flip_y {
            offset.y = -(self.mount_offset.y + size.y);
        } else {
            offset.y = self.mount_offset.y
        }

        offset
    }

    pub fn get_effect_offset(&self, flip_x: bool, flip_y: bool) -> Vec2 {
        let mut offset = Vec2::ZERO;

        if flip_x {
            offset.x = -self.effect_offset.x;
        } else {
            offset.x = self.effect_offset.x;
        }

        if flip_y {
            offset.y = -self.effect_offset.y;
        } else {
            offset.y = self.effect_offset.y;
        }

        offset
    }

    pub fn update(&mut self, dt: f32) {
        self.sprite_animation.update();

        if let Some(effect_animation) = &mut self.effect_animation {
            effect_animation.update();
        }

        self.cooldown_timer += dt;
    }

    pub fn draw(&mut self, position: Vec2, rotation: f32, flip_x: bool, flip_y: bool) {
        let position = position + self.get_mount_offset(flip_x, flip_y);

        self.sprite_animation
            .draw(position, rotation, flip_x, flip_y);

        if let Some(effect_animation) = &mut self.effect_animation {
            effect_animation.draw(position, rotation, flip_x, flip_y);
        }

        #[cfg(debug_assertions)]
        self.sprite_animation.debug_draw(position);
    }

    pub fn draw_hud(&self, position: Vec2) {
        if let Some(uses) = self.uses {
            if !self.is_destroyed_on_depletion || uses > 1 {
                let remaining = uses - self.use_cnt;

                if uses >= Self::HUD_CONDENSED_USE_COUNT_THRESHOLD {
                    let x = position.x - ((4.0 * uses as f32) / 2.0);

                    for i in 0..uses {
                        draw_rectangle(
                            x + 4.0 * i as f32,
                            position.y - 12.0,
                            2.0,
                            12.0,
                            if i >= remaining {
                                Self::HUD_USE_COUNT_COLOR_EMPTY
                            } else {
                                Self::HUD_USE_COUNT_COLOR_FULL
                            },
                        )
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

    fn animation_coroutine(
        player_handle: Handle<Player>,
        animation_id: &str,
        is_effect: bool,
    ) -> Coroutine {
        let animation_id = animation_id.to_string();

        let coroutine = async move {
            let mut animation = None;

            {
                let player = &mut *scene::get_node(player_handle);
                if let Some(weapon) = &mut player.weapon {
                    if is_effect {
                        if let Some(animation_player) = &mut weapon.effect_animation {
                            animation = animation_player.set_animation(&animation_id).cloned();

                            if animation.is_some() {
                                animation_player.stop();
                                animation_player.is_deactivated = false;
                            }
                        }
                    } else {
                        animation = weapon
                            .sprite_animation
                            .set_animation(&animation_id)
                            .cloned();

                        if animation.is_some() {
                            weapon.sprite_animation.stop();
                        }
                    }
                }
            }

            if let Some(animation) = animation {
                let frame_interval = 1.0 / animation.fps as f32;

                for i in 0..animation.frames as usize {
                    {
                        let player = &mut *scene::get_node(player_handle);
                        if let Some(weapon) = player.weapon.as_mut() {
                            if is_effect {
                                let animation_player = weapon.effect_animation.as_mut().unwrap();

                                animation_player.set_frame(i);
                            } else {
                                weapon.sprite_animation.set_frame(i);
                            }
                        }
                    }

                    wait_seconds(frame_interval).await;
                }

                {
                    let player = &mut *scene::get_node(player_handle);
                    if let Some(weapon) = player.weapon.as_mut() {
                        if is_effect {
                            let animation_player = weapon.effect_animation.as_mut().unwrap();
                            animation_player.stop();
                            animation_player.is_deactivated = true;
                        } else {
                            weapon
                                .sprite_animation
                                .set_animation(Self::IDLE_ANIMATION_ID);

                            weapon.sprite_animation.play();
                        }
                    }
                }
            }
        };

        start_coroutine(coroutine)
    }

    /// This will start a `Coroutine` that performs an attack with the `Weapon` equipped by the
    /// `Player` fetched with `player_handle`, id one is equipped and ready for use.
    pub fn use_coroutine(player_handle: Handle<Player>) -> Coroutine {
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
                let mut should_destroy = false;

                {
                    let player = &mut *scene::get_node(player_handle);
                    if let Some(weapon) = player.weapon.as_mut() {
                        if let Some(uses) = weapon.uses {
                            weapon.use_cnt += 1;

                            if weapon.is_destroyed_on_depletion && weapon.use_cnt >= uses {
                                should_destroy = true;
                            }
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
                    let weapon_mount = player.get_weapon_mount_position();

                    if let Some(weapon) = player.weapon.as_mut() {
                        let origin = weapon_mount
                            + weapon.mount_offset
                            + weapon.get_effect_offset(!player.body.is_facing_right, false);
                        for params in weapon.effects.clone() {
                            active_effect_coroutine(player_handle, origin, params);
                        }
                    }
                }

                {
                    Weapon::animation_coroutine(player_handle, Self::ATTACK_ANIMATION_ID, false);
                    Weapon::animation_coroutine(
                        player_handle,
                        Self::ATTACK_EFFECT_ANIMATION_ID,
                        true,
                    );
                }

                let attack_duration = {
                    let player = &*scene::get_node(player_handle);
                    player.weapon.as_ref().map(|weapon| weapon.attack_duration)
                };

                if let Some(attack_duration) = attack_duration {
                    wait_seconds(attack_duration).await;
                }

                if should_destroy {
                    let player = &mut *scene::get_node(player_handle);
                    player.weapon = None;
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
