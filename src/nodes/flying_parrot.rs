use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::RefMut,
    },
    prelude::*,
};

use crate::{nodes::player::PhysicsBody, Resources};

use super::player::{PLAYER_HITBOX_HEIGHT, PLAYER_HITBOX_WIDTH};

const FLYING_PARROT_COUNTDOWN_DURATION: f32 = 0.75;

const FLYING_PARROT_WIDTH: f32 = 40.;
pub const FLYING_PARROT_HEIGHT: f32 = 18.;
const FLYING_PARROT_ANIMATION_FLYING: &'static str = "flying";
const FLYING_PARROT_SPEED_X: f32 = 600.;
const FLYING_PARROT_MAX_AMPLITUDE: f32 = 100.;
const FLYING_PARROT_Y_FREQ_SLOWDOWN: f32 = 75.; // the higher, the slower the frequency is
const FLYING_PARROT_MOUNT_X_REL: f32 = -35.;

/// The FlyingParrot doesn't have a body, as it has a non-conventional (sinuisodal) motion.
pub struct FlyingParrot {
    flying_parrot_sprite: AnimatedSprite,
    reference_pos: Vec2,
    current_x: f32,
    speed_x: f32,
    facing: bool,
    lived: f32,
    countdown: f32,
}

impl FlyingParrot {
    pub fn new(parrot_body: &PhysicsBody) -> Self {
        // This can be easily turned into a single sprite, rotated via DrawTextureParams.
        //
        let flying_parrot_sprite = AnimatedSprite::new(
            FLYING_PARROT_WIDTH as u32,
            FLYING_PARROT_HEIGHT as u32,
            &[Animation {
                name: FLYING_PARROT_ANIMATION_FLYING.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            true,
        );

        let facing_x_factor = if parrot_body.facing { 1. } else { -1. };

        let speed_x = facing_x_factor * FLYING_PARROT_SPEED_X;
        let current_x = parrot_body.pos.x - facing_x_factor * FLYING_PARROT_MOUNT_X_REL;

        Self {
            flying_parrot_sprite,
            current_x,
            reference_pos: vec2(parrot_body.pos.x, parrot_body.pos.y),
            speed_x,
            facing: parrot_body.facing,
            lived: 0.0,
            countdown: FLYING_PARROT_COUNTDOWN_DURATION,
        }
    }

    fn current_y(&self) -> f32 {
        // Start always from the negative value, so that the motion always starts upwards.
        let diff_x = -(self.current_x - self.reference_pos.x).abs();
        let displacement =
            (diff_x / FLYING_PARROT_Y_FREQ_SLOWDOWN).sin() * (FLYING_PARROT_MAX_AMPLITUDE / 2.);
        self.reference_pos.y + displacement
    }
}

pub struct FlyingParrots {
    flying_parrots: Vec<FlyingParrot>,
}

impl FlyingParrots {
    pub fn new() -> Self {
        FlyingParrots {
            flying_parrots: vec![],
        }
    }

    pub fn spawn_flying_parrot(&mut self, parrot_body: &PhysicsBody) {
        self.flying_parrots.push(FlyingParrot::new(parrot_body));
    }
}

impl scene::Node for FlyingParrots {
    fn fixed_update(mut node: RefMut<Self>) {
        for flying_parrot in &mut node.flying_parrots {
            flying_parrot.lived += get_frame_time();
            flying_parrot.current_x += flying_parrot.speed_x * get_frame_time();
        }

        node.flying_parrots.retain(|flying_parrot| {
            let hit_fxses = &mut storage::get_mut::<Resources>().hit_fxses;
            let explosion_position = vec2(
                flying_parrot.current_x + FLYING_PARROT_WIDTH / 2.,
                flying_parrot.current_y() + FLYING_PARROT_HEIGHT / 2.,
            );

            if flying_parrot.lived < flying_parrot.countdown {
                let flying_parrot_hitbox = Rect::new(
                    flying_parrot.current_x,
                    flying_parrot.current_y(),
                    FLYING_PARROT_WIDTH,
                    FLYING_PARROT_HEIGHT,
                );

                for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                    let player_hitbox = Rect::new(
                        player.body.pos.x,
                        player.body.pos.y,
                        PLAYER_HITBOX_WIDTH,
                        PLAYER_HITBOX_HEIGHT,
                    );
                    if player_hitbox.intersect(flying_parrot_hitbox).is_some() {
                        hit_fxses.spawn(explosion_position);

                        scene::find_node_by_type::<crate::nodes::Camera>()
                            .unwrap()
                            .shake();

                        let direction = flying_parrot.current_x
                            > (player.body.pos.x + PLAYER_HITBOX_WIDTH / 2.);
                        player.kill(direction);

                        return false;
                    }
                }

                return true;
            }

            hit_fxses.spawn(explosion_position);

            false
        });
    }

    fn draw(mut node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();
        for flying_parrot in &mut node.flying_parrots {
            flying_parrot.flying_parrot_sprite.update();

            draw_texture_ex(
                resources.flying_parrots,
                flying_parrot.current_x,
                flying_parrot.current_y(),
                color::WHITE,
                DrawTextureParams {
                    source: Some(flying_parrot.flying_parrot_sprite.frame().source_rect),
                    dest_size: Some(flying_parrot.flying_parrot_sprite.frame().dest_size),
                    flip_x: flying_parrot.facing,
                    rotation: 0.0,
                    ..Default::default()
                },
            );
        }
    }
}
