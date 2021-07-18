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

const FLYING_CURSE_COUNTDOWN_DURATION: f32 = 0.75;

const FLYING_CURSE_WIDTH: f32 = 32.;
pub const FLYING_CURSE_HEIGHT: f32 = 32.;
const FLYING_CURSE_ANIMATION_FLYING: &'static str = "flying";
const FLYING_CURSE_SPEED_X: f32 = 600.;
const FLYING_CURSE_MAX_AMPLITUDE: f32 = 100.;
const FLYING_CURSE_Y_FREQ_SLOWDOWN: f32 = 75.; // the higher, the slower the frequency is
const FLYING_CURSE_MOUNT_X_REL: f32 = -35.;

/// The FlyingCurse doesn't have a body, as it has a non-conventional (sinuisodal) motion.
pub struct FlyingCurse {
    flying_curse_sprite: AnimatedSprite,
    reference_pos: Vec2,
    current_x: f32,
    speed_x: f32,
    facing: bool,
    lived: f32,
    countdown: f32,
}

impl FlyingCurse {
    pub fn new(curse_body: &PhysicsBody) -> Self {
        // This can be easily turned into a single sprite, rotated via DrawTextureParams.
        //
        let flying_curse_sprite = AnimatedSprite::new(
            FLYING_CURSE_WIDTH as u32,
            FLYING_CURSE_HEIGHT as u32,
            &[Animation {
                name: FLYING_CURSE_ANIMATION_FLYING.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            true,
        );

        let facing_x_factor = if curse_body.facing { 1. } else { -1. };

        let speed_x = facing_x_factor * FLYING_CURSE_SPEED_X;
        let current_x = curse_body.pos.x - facing_x_factor * FLYING_CURSE_MOUNT_X_REL;

        Self {
            flying_curse_sprite,
            current_x,
            reference_pos: vec2(curse_body.pos.x, curse_body.pos.y),
            speed_x,
            facing: curse_body.facing,
            lived: 0.0,
            countdown: FLYING_CURSE_COUNTDOWN_DURATION,
        }
    }

    fn current_y(&self) -> f32 {
        // Start always from the negative value, so that the motion always starts upwards.
        let diff_x = -(self.current_x - self.reference_pos.x).abs();
        let displacement =
            (diff_x / FLYING_CURSE_Y_FREQ_SLOWDOWN).sin() * (FLYING_CURSE_MAX_AMPLITUDE / 2.);
        self.reference_pos.y + displacement
    }
}

pub struct FlyingCurses {
    flying_curses: Vec<FlyingCurse>,
}

impl FlyingCurses {
    pub fn new() -> Self {
        FlyingCurses {
            flying_curses: vec![],
        }
    }

    pub fn spawn_flying_curse(&mut self, curse_body: &PhysicsBody) {
        self.flying_curses.push(FlyingCurse::new(curse_body));
    }
}

impl scene::Node for FlyingCurses {
    fn fixed_update(mut node: RefMut<Self>) {
        for flying_curse in &mut node.flying_curses {
            flying_curse.lived += get_frame_time();
            flying_curse.current_x += flying_curse.speed_x * get_frame_time();
        }

        node.flying_curses.retain(|flying_curse| {
            let hit_fxses = &mut storage::get_mut::<Resources>().hit_fxses;
            let explosion_position = vec2(
                flying_curse.current_x + FLYING_CURSE_WIDTH / 2.,
                flying_curse.current_y() + FLYING_CURSE_HEIGHT / 2.,
            );

            if flying_curse.lived < flying_curse.countdown {
                let flying_curse_hitbox = Rect::new(
                    flying_curse.current_x,
                    flying_curse.current_y(),
                    FLYING_CURSE_WIDTH,
                    FLYING_CURSE_HEIGHT,
                );

                for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                    let player_hitbox = Rect::new(
                        player.body.pos.x,
                        player.body.pos.y,
                        PLAYER_HITBOX_WIDTH,
                        PLAYER_HITBOX_HEIGHT,
                    );
                    if player_hitbox.intersect(flying_curse_hitbox).is_some() {
                        hit_fxses.spawn(explosion_position);

                        scene::find_node_by_type::<crate::nodes::Camera>()
                            .unwrap()
                            .shake();

                        let direction =
                            flying_curse.current_x > (player.body.pos.x + PLAYER_HITBOX_WIDTH / 2.);
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
        for flying_curse in &mut node.flying_curses {
            flying_curse.flying_curse_sprite.update();

            draw_texture_ex(
                resources.flying_curses,
                flying_curse.current_x,
                flying_curse.current_y(),
                color::WHITE,
                DrawTextureParams {
                    source: Some(flying_curse.flying_curse_sprite.frame().source_rect),
                    dest_size: Some(flying_curse.flying_curse_sprite.frame().dest_size),
                    flip_x: flying_curse.facing,
                    rotation: 0.0,
                    ..Default::default()
                },
            );
        }
    }
}
