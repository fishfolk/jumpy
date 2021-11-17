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

use crate::components::{ParticleController, ParticleControllerParams};
use crate::{
    components::{AnimationParams, AnimationPlayer},
    effects::{active_effect_coroutine, ActiveEffectParams},
    json::{self, OneOrMany},
    Player, Resources,
};

/// This holds the parameters for the `AnimationPlayer` components of an equipped `Weapon`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
    /// a single `ActiveEffectParams` or a vector of `ActiveEffectParams`.
    #[serde(alias = "effect")]
    pub effects: OneOrMany<ActiveEffectParams>,
    /// Particle effects that will be activated when using the weapon
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    particles: Vec<ParticleControllerParams>,
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

impl Default for WeaponParams {
    fn default() -> Self {
        WeaponParams {
            effects: OneOrMany::Many(Vec::new()),
            particles: Vec::new(),
            sound_effect_id: None,
            uses: None,
            is_destroyed_on_depletion: false,
            mount_offset: Vec2::ZERO,
            effect_offset: Vec2::ZERO,
            attack_duration: 0.0,
            cooldown: 0.0,
            recoil: 0.0,
            animation: Default::default(),
        }
    }
}

pub struct Weapon {
    pub id: String,
    pub particles: Vec<ParticleController>,
    pub sound_effect: Option<Sound>,
    pub effects: Vec<ActiveEffectParams>,
    pub cooldown: f32,
    pub recoil: f32,
    pub attack_duration: f32,
    pub uses: Option<u32>,
    pub sprite_animation: AnimationPlayer,
    pub effect_animation: Option<AnimationPlayer>,
    pub cooldown_timer: f32,
    mount_offset: Vec2,
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
        let particles = params
            .particles
            .into_iter()
            .map(ParticleController::new)
            .collect();

        let sound_effect = params.sound_effect_id.as_ref().map(|id| {
            let resources = storage::get::<Resources>();
            resources.sounds[id]
        });

        let sprite_animation = AnimationPlayer::new(params.animation.sprite);

        let effect_animation = params.animation.effect.map(|params| {
            AnimationPlayer::new(AnimationParams {
                is_deactivated: true,
                ..params
            })
        });

        Weapon {
            id: id.to_string(),
            particles,
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

    pub fn get_mount_offset(&self, flip_x: bool, flip_y: bool) -> Vec2 {
        let mut offset = self.mount_offset;

        if flip_x {
            offset.x = -offset.x;
        }

        if flip_y {
            offset.y = -offset.y;
        }

        offset
    }

    fn get_effect_offset(&self, flip_x: bool, flip_y: bool) -> Vec2 {
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
        self.cooldown_timer += dt;

        self.sprite_animation.update();

        if let Some(effect_animation) = &mut self.effect_animation {
            effect_animation.update();
        }

        for particles in &mut self.particles {
            particles.update(dt);
        }
    }

    pub fn draw(&mut self, position: Vec2, rotation: f32, flip_x: bool, flip_y: bool) {
        let position = position + self.get_mount_offset(flip_x, flip_y);

        {
            let mut offset = Vec2::ZERO;

            let size = self.sprite_animation.get_size();

            if flip_x {
                offset.x -= size.x;
            }

            if flip_y {
                offset.y -= size.y;
            }

            let position = position + offset;

            self.sprite_animation
                .draw(position, rotation, flip_x, flip_y);

            if let Some(effect_animation) = &mut self.effect_animation {
                effect_animation.draw(position, rotation, flip_x, flip_y);
            }

            #[cfg(debug_assertions)]
            self.sprite_animation.debug_draw(position);
        }

        {
            let position = position + self.get_effect_offset(flip_x, flip_y);

            for particles in &mut self.particles {
                particles.draw(position, flip_x, flip_y);
            }
        }
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
                    let (flip_x, flip_y) =
                        (!player.body.is_facing_right, player.body.is_upside_down);

                    if let Some(weapon) = &mut player.weapon {
                        for particles in &mut weapon.particles {
                            particles.activate();
                        }

                        let origin = weapon_mount
                            + weapon.get_mount_offset(flip_x, flip_y)
                            + weapon.get_effect_offset(flip_x, flip_y);

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
