use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::RefMut,
    },
    prelude::{scene::Handle, *},
};

use crate::{nodes::player::PhysicsBody, Resources};

use super::{
    player::{PLAYER_HITBOX_HEIGHT, PLAYER_HITBOX_WIDTH},
    Player,
};

const CANNONBALL_COUNTDOWN_DURATION: f32 = 0.5;
/// After shooting, the owner is safe for this amount of time. This is crucial, otherwise, given the
/// large hitbox, they will die immediately on shoot.
/// The formula is simplified (it doesn't include mount position, run speed and throwback).
const CANNONBALL_OWNER_SAFE_TIME: f32 =
    (EXPLOSION_HITBOX_WIDTH / 2.) / CANNONBALL_INITIAL_SPEED_X_REL;

const CANNONBALL_WIDTH: f32 = 32.;
pub const CANNONBALL_HEIGHT: f32 = 36.;
const CANNONBALL_ANIMATION_ROLLING: &'static str = "rolling";
const CANNONBALL_INITIAL_SPEED_X_REL: f32 = 600.;
const CANNONBALL_INITIAL_SPEED_Y: f32 = -200.;
const CANNONBALL_MOUNT_X_REL: f32 = 20.;
const CANNONBALL_MOUNT_Y: f32 = 40.;

const EXPLOSION_HITBOX_WIDTH: f32 = 4. * CANNONBALL_WIDTH;
const EXPLOSION_HITBOX_HEIGHT: f32 = 4. * CANNONBALL_HEIGHT;

pub struct Cannonball {
    cannonball_sprite: AnimatedSprite,
    body: PhysicsBody,
    lived: f32,
    countdown: f32,
    owner: Handle<Player>,
    owner_safe_countdown: f32,
}

impl Cannonball {
    pub fn new(pos: Vec2, facing: bool, owner: Handle<Player>) -> Self {
        // This can be easily turned into a single sprite, rotated via DrawTextureParams.
        //
        let cannonball_sprite = AnimatedSprite::new(
            CANNONBALL_WIDTH as u32,
            CANNONBALL_HEIGHT as u32,
            &[Animation {
                name: CANNONBALL_ANIMATION_ROLLING.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            true,
        );

        let speed = if facing {
            vec2(CANNONBALL_INITIAL_SPEED_X_REL, CANNONBALL_INITIAL_SPEED_Y)
        } else {
            vec2(-CANNONBALL_INITIAL_SPEED_X_REL, CANNONBALL_INITIAL_SPEED_Y)
        };

        let mut body = PhysicsBody {
            pos,
            facing,
            angle: 0.0,
            speed,
            collider: None,
            on_ground: false,
            last_frame_on_ground: false,
            have_gravity: true,
            bouncyness: 1.0,
        };

        let mut resources = storage::get_mut::<Resources>();

        let cannonball_mount_pos = if facing {
            vec2(CANNONBALL_MOUNT_X_REL, CANNONBALL_MOUNT_Y)
        } else {
            vec2(-CANNONBALL_MOUNT_X_REL, CANNONBALL_MOUNT_Y)
        };

        body.collider = Some(resources.collision_world.add_actor(
            body.pos + cannonball_mount_pos,
            CANNONBALL_WIDTH as i32,
            CANNONBALL_HEIGHT as i32,
        ));

        Self {
            cannonball_sprite,
            body,
            lived: 0.0,
            countdown: CANNONBALL_COUNTDOWN_DURATION,
            owner,
            owner_safe_countdown: CANNONBALL_OWNER_SAFE_TIME,
        }
    }
}

pub struct Cannonballs {
    cannonballs: Vec<Cannonball>,
}

impl Cannonballs {
    pub fn new() -> Self {
        Cannonballs {
            cannonballs: vec![],
        }
    }

    pub fn spawn_cannonball(&mut self, pos: Vec2, facing: bool, owner: Handle<Player>) {
        self.cannonballs.push(Cannonball::new(pos, facing, owner));
    }
}

impl scene::Node for Cannonballs {
    fn fixed_update(mut node: RefMut<Self>) {
        for cannonball in &mut node.cannonballs {
            cannonball.body.update();
            cannonball.lived += get_frame_time();
            cannonball.owner_safe_countdown -= get_frame_time();
        }

        node.cannonballs.retain(|cannonball| {
            let hit_fxses = &mut storage::get_mut::<Resources>().hit_fxses;
            let explosion_position =
                cannonball.body.pos + vec2(CANNONBALL_WIDTH / 2., CANNONBALL_HEIGHT / 2.);

            if cannonball.lived < cannonball.countdown {
                let cannonball_owner_id = scene::get_node(cannonball.owner).id;

                let cannonball_hitbox = Rect::new(
                    cannonball.body.pos.x + (CANNONBALL_WIDTH - EXPLOSION_HITBOX_WIDTH) / 2.,
                    cannonball.body.pos.y + (CANNONBALL_HEIGHT - EXPLOSION_HITBOX_HEIGHT) / 2.,
                    EXPLOSION_HITBOX_WIDTH,
                    EXPLOSION_HITBOX_HEIGHT,
                );

                for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                    if player.id != cannonball_owner_id || cannonball.owner_safe_countdown < 0. {
                        let player_hitbox = Rect::new(
                            player.body.pos.x,
                            player.body.pos.y,
                            PLAYER_HITBOX_WIDTH,
                            PLAYER_HITBOX_HEIGHT,
                        );
                        if player_hitbox.intersect(cannonball_hitbox).is_some() {
                            hit_fxses.spawn(explosion_position);

                            scene::find_node_by_type::<crate::nodes::Camera>()
                                .unwrap()
                                .shake();

                            let direction = cannonball.body.pos.x
                                > (player.body.pos.x + PLAYER_HITBOX_WIDTH / 2.);
                            player.kill(direction);

                            return false;
                        }
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
        for cannonball in &mut node.cannonballs {
            cannonball.cannonball_sprite.update();

            draw_texture_ex(
                resources.cannonballs,
                cannonball.body.pos.x,
                cannonball.body.pos.y,
                color::WHITE,
                DrawTextureParams {
                    source: Some(cannonball.cannonball_sprite.frame().source_rect),
                    dest_size: Some(cannonball.cannonball_sprite.frame().dest_size),
                    flip_x: cannonball.body.facing,
                    rotation: 0.0,
                    ..Default::default()
                },
            );
        }
    }
}
