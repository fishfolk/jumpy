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
const FLYING_PARROT_INITIAL_SPEED_X_REL: f32 = 600.;
const FLYING_PARROT_INITIAL_SPEED_Y: f32 = -200.;
const FLYING_PARROT_MOUNT_X_REL: f32 = -35.;

pub struct FlyingParrot {
    flying_parrot_sprite: AnimatedSprite,
    body: PhysicsBody,
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
        let mut resources = storage::get_mut::<Resources>();

        let speed = vec2(
            facing_x_factor * FLYING_PARROT_INITIAL_SPEED_X_REL,
            FLYING_PARROT_INITIAL_SPEED_Y,
        );

        let flying_parrot_mount_pos = vec2(
            parrot_body.pos.x - facing_x_factor * FLYING_PARROT_MOUNT_X_REL,
            parrot_body.pos.y,
        );

        let actor = resources.collision_world.add_actor(
            flying_parrot_mount_pos,
            FLYING_PARROT_WIDTH as i32,
            FLYING_PARROT_HEIGHT as i32,
        );

        let body = PhysicsBody {
            pos: flying_parrot_mount_pos,
            facing: parrot_body.facing,
            angle: 0.0,
            speed,
            collider: Some(actor),
            on_ground: false,
            last_frame_on_ground: false,
            have_gravity: true,
        };

        Self {
            flying_parrot_sprite,
            body,
            lived: 0.0,
            countdown: FLYING_PARROT_COUNTDOWN_DURATION,
        }
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
            flying_parrot.body.update();
            flying_parrot.lived += get_frame_time();
        }

        node.flying_parrots.retain(|flying_parrot| {
            let hit_fxses = &mut storage::get_mut::<Resources>().hit_fxses;
            let explosion_position =
                flying_parrot.body.pos + vec2(FLYING_PARROT_WIDTH / 2., FLYING_PARROT_HEIGHT / 2.);

            if flying_parrot.lived < flying_parrot.countdown {
                let flying_parrot_hitbox = Rect::new(
                    flying_parrot.body.pos.x,
                    flying_parrot.body.pos.y,
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

                        let direction = flying_parrot.body.pos.x
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
                flying_parrot.body.pos.x,
                flying_parrot.body.pos.y,
                color::WHITE,
                DrawTextureParams {
                    source: Some(flying_parrot.flying_parrot_sprite.frame().source_rect),
                    dest_size: Some(flying_parrot.flying_parrot_sprite.frame().dest_size),
                    flip_x: flying_parrot.body.facing,
                    rotation: 0.0,
                    ..Default::default()
                },
            );
        }
    }
}
