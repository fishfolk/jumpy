use macroquad::{
    audio::Sound,
    color,
    experimental::{
        collections::storage,
        coroutines::{
            Coroutine,
            start_coroutine,
            wait_seconds,
        },
        scene::Handle,
    },
    audio::play_sound_once,
    prelude::*,
};

use serde::{Deserialize, Serialize};

pub use effects::{
    EffectTrigger,
    WeaponEffectKind,
    WeaponEffectParams,
    weapon_effect_coroutine,
};

use crate::{
    components::{AnimationParams, AnimationPlayer},
    json,
    Player, Resources,
};

pub mod effects;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound_effect_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    #[serde(flatten)]
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
    pub idle_animation: usize,
    pub attack_animation: Option<usize>,
    pub animation_player: AnimationPlayer,
    pub cooldown_timer: f32,
    mount_offset: Vec2,
    effect_offset: Vec2,
}

impl Weapon {
    pub const CONDENSED_USE_COUNT_THRESHOLD: u32 = 12;

    const IDLE_ANIMATION_NAME: &'static str = "idle";
    const ATTACK_ANIMATION_NAME: &'static str = "attack";

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

        assert!(
            !params.animation.animations.is_empty(),
            "Weapon: A minimum of one animation ({}) is required",
            Self::IDLE_ANIMATION_NAME
        );

        let mut idle_animation = 0;
        let mut attack_animation = None;

        for (i, animation) in params.animation.animations.iter().enumerate() {
            if animation.name == Self::IDLE_ANIMATION_NAME {
                idle_animation = i;
            } else if animation.name == Self::ATTACK_ANIMATION_NAME {
                attack_animation = Some(i);
            }
        }

        let animation_player = AnimationPlayer::new(
            AnimationParams {
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
            idle_animation,
            attack_animation,
            mount_offset: params.mount_offset,
            effect_offset: params.effect_offset,
        }
    }

    pub fn update(&mut self) {
        self.animation_player.update();

        self.cooldown_timer += get_frame_time();
    }

    pub fn draw(
        &self,
        position: Vec2,
        rotation: f32,
        scale: Option<Vec2>,
        flip_x: bool,
        flip_y: bool,
    ) {
        let rect = self.animation_player.get_rect(scale);
        let mut corrected_position = position + self.mount_offset;
        if flip_x {
            corrected_position.x -= rect.w;
        }
        if flip_y {
            corrected_position.y -= rect.h;
        }

        self.animation_player
            .draw(corrected_position, rotation, scale, flip_x, flip_y);

        draw_rectangle_lines(
            corrected_position.x,
            corrected_position.y,
            rect.w,
            rect.h,
            1.0,
            color::RED,
        );
    }

    pub fn get_effect_offset(&self, facing_direction: Vec2) -> Vec2 {
        vec2(
            facing_direction.x * self.effect_offset.x,
            facing_direction.y * self.effect_offset.y,
        )
    }

    pub fn is_ready(&self) -> bool {
        if self.cooldown_timer < self.cooldown {
            return false;
        } else if let Some(uses) = self.uses {
            if self.use_cnt >= uses {
                return false;
            }
        }

        true
    }

    pub fn animation_coroutine(player_handle: Handle<Player>) -> Coroutine {
        let animation_coroutine = async move {
            let attack_animation = {
                let player = &mut *scene::get_node(player_handle);
                let weapon = player.weapon.as_ref().unwrap();

                weapon.attack_animation
            };

            if let Some(index) = attack_animation {
                let animation = {
                    let player = &mut *scene::get_node(player_handle);
                    let weapon = player.weapon.as_mut().unwrap();

                    weapon.animation_player.set_animation(index);
                    weapon.animation_player.get_animation(index).clone()
                };

                let frame_interval = 1.0 / animation.fps as f32;

                for i in 0..animation.frames as usize {
                    {
                        let player = &mut *scene::get_node(player_handle);
                        let weapon = player.weapon.as_mut().unwrap();

                        weapon.animation_player.set_frame(i);
                    }

                    wait_seconds(frame_interval).await;
                }

                {
                    let player = &mut *scene::get_node(player_handle);
                    let weapon = player.weapon.as_mut().unwrap();
                    weapon.animation_player.set_animation(weapon.idle_animation);
                }
            }
        };

        start_coroutine(animation_coroutine)
    }

    pub fn attack_coroutine(player_handle: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let player = &mut *scene::get_node(player_handle);
                let weapon = player.weapon.as_mut().unwrap();

                weapon.use_cnt += 1;
                weapon.cooldown_timer = 0.0;

                if let Some(sound_effect) = weapon.sound_effect {
                    play_sound_once(sound_effect);
                }

                player.body.velocity.x = if player.body.facing { -weapon.recoil } else { weapon.recoil };

                let origin = player.body.pos + player.weapon_mount_offset + weapon.get_effect_offset(player.body.facing_dir());

                weapon_effect_coroutine(player_handle, origin, weapon.effect.clone());
            }

            Weapon::animation_coroutine(player_handle);

            let attack_duration = {
                let player = &*scene::get_node(player_handle);
                let weapon = player.weapon.as_ref().unwrap();
                weapon.attack_duration
            };

            wait_seconds(attack_duration).await;

            {
                let player = &mut *scene::get_node(player_handle);
                player.state_machine.set_state(Player::ST_NORMAL);
            }
        };

        start_coroutine(coroutine)
    }
}
