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

use super::EruptedItem;
use crate::circle::Circle;

const CANNONBALL_COUNTDOWN_DURATION: f32 = 0.5;
/// After shooting, the owner is safe for this amount of time. This is crucial, otherwise, given the
/// large hitbox, they will die immediately on shoot.
/// The formula is simplified (it doesn't include mount position, run speed and throwback).
const CANNONBALL_OWNER_SAFE_TIME: f32 = EXPLOSION_RADIUS / CANNONBALL_INITIAL_SPEED_X_REL;

const CANNONBALL_WIDTH: f32 = 32.;
pub const CANNONBALL_HEIGHT: f32 = 36.;
const CANNONBALL_ANIMATION_ROLLING: &'static str = "rolling";
const CANNONBALL_INITIAL_SPEED_X_REL: f32 = 600.;
const CANNONBALL_INITIAL_SPEED_Y: f32 = -200.;
const CANNONBALL_MOUNT_X_REL: f32 = 20.;
const CANNONBALL_MOUNT_Y: f32 = 40.;

const EXPLOSION_RADIUS: f32 = 4. * CANNONBALL_WIDTH;

pub struct Cannonball {
    cannonball_sprite: AnimatedSprite,
    body: PhysicsBody,
    lived: f32,
    countdown: f32,
    owner_id: u8,
    owner_safe_countdown: f32,
    /// True if erupting from a volcano
    erupting: bool,
    /// When erupting, enable the collider etc. after passing this coordinate on the way down. Set/valid
    /// only when erupting.
    erupting_enable_on_y: Option<f32>,
}

impl Cannonball {
    // Use Cannonball::spawn(), which handles the scene graph.
    //
    fn new(pos: Vec2, facing: bool, owner_id: u8) -> Self {
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
            owner_id,
            owner_safe_countdown: CANNONBALL_OWNER_SAFE_TIME,
            erupting: false,
            erupting_enable_on_y: None,
        }
    }

    pub fn spawn(pos: Vec2, facing: bool, owner_id: u8) {
        let cannonball = Cannonball::new(pos, facing, owner_id);
        scene::add_node(cannonball);
    }
}

impl EruptedItem for Cannonball {
    fn spawn_for_volcano(pos: Vec2, speed: Vec2, enable_at_y: f32, owner_id: u8) {
        let mut cannonball = Self::new(pos, true, owner_id);

        cannonball.lived -= 2.; // give extra life, since they're random
        cannonball.body.speed = speed;
        cannonball.body.collider = None;
        cannonball.erupting = true;
        cannonball.erupting_enable_on_y = Some(enable_at_y);

        scene::add_node(cannonball);
    }

    fn body(&mut self) -> &mut PhysicsBody {
        &mut self.body
    }
    fn enable_at_y(&self) -> f32 {
        self.erupting_enable_on_y.unwrap()
    }
}

impl scene::Node for Cannonball {
    fn fixed_update(mut cannonball: RefMut<Self>) {
        if cannonball.erupting {
            let cannonball_enabled = cannonball.eruption_update();

            if !cannonball_enabled {
                return;
            }
        }

        cannonball.body.update();
        cannonball.lived += get_frame_time();
        cannonball.owner_safe_countdown -= get_frame_time();

        let hit_fxses = &mut storage::get_mut::<Resources>().cannonball_hit_fxses;

        let explosion_position =
            cannonball.body.pos + vec2(CANNONBALL_WIDTH / 2., CANNONBALL_HEIGHT / 2.);

        if cannonball.lived < cannonball.countdown {
            let explosion = Circle::new(
                cannonball.body.pos.x,
                cannonball.body.pos.y,
                EXPLOSION_RADIUS,
            );

            for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                if player.id != cannonball.owner_id || cannonball.owner_safe_countdown < 0. {
                    let player_hitbox = player.get_hitbox();
                    if explosion.overlaps(player_hitbox) {
                        hit_fxses.spawn(explosion_position);

                        scene::find_node_by_type::<crate::nodes::Camera>()
                            .unwrap()
                            .shake();

                        let direction =
                            cannonball.body.pos.x > (player.body.pos.x + player_hitbox.w / 2.);
                        player.kill(direction);

                        cannonball.delete();

                        return;
                    }
                }
            }

            return;
        }

        hit_fxses.spawn(explosion_position);

        cannonball.delete();
    }

    fn draw(mut cannonball: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

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
