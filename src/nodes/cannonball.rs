use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::RefMut,
    },
    prelude::*,
};

use crate::{components::PhysicsBody, Resources};

//use super::EruptedItem;

pub struct Cannonball {
    cannonball_sprite: AnimatedSprite,
    body: PhysicsBody,
    lived: f32,
    countdown: f32,
    owner_id: u8,
    owner_safe_countdown: f32,
    //   True if erupting from a volcano
    //erupting: bool,
    //     When erupting, enable the collider etc. after passing this coordinate on the way down. Set/valid
    //     only when erupting.
    //erupting_enable_on_y: Option<f32>,
}

impl Cannonball {
    const CANNONBALL_COUNTDOWN_DURATION: f32 = 0.5;
    /// After shooting, the owner is safe for this amount of time. This is crucial, otherwise, given the
    /// large hitbox, they will die immediately on shoot.
    /// The formula is simplified (it doesn't include mount position, run speed and throwback).
    const CANNONBALL_OWNER_SAFE_TIME: f32 =
        Self::EXPLOSION_RADIUS / Self::CANNONBALL_INITIAL_SPEED_X_REL;

    const CANNONBALL_WIDTH: f32 = 32.;
    pub const CANNONBALL_HEIGHT: f32 = 36.;
    const CANNONBALL_ANIMATION_ROLLING: &'static str = "rolling";
    const CANNONBALL_INITIAL_SPEED_X_REL: f32 = 600.;
    const CANNONBALL_INITIAL_SPEED_Y: f32 = -200.;
    //const CANNONBALL_MOUNT_X_REL: f32 = 20.;
    //const CANNONBALL_MOUNT_Y: f32 = 40.;

    const EXPLOSION_RADIUS: f32 = 4. * Self::CANNONBALL_WIDTH;

    // Use Cannonball::spawn(), which handles the scene graph.
    //
    fn new(pos: Vec2, facing: bool, owner_id: u8) -> Self {
        // This can be easily turned into a single sprite, rotated via DrawTextureParams.
        //
        let mut resources = storage::get_mut::<Resources>();
        let cannonball_sprite = AnimatedSprite::new(
            Self::CANNONBALL_WIDTH as u32,
            Self::CANNONBALL_HEIGHT as u32,
            &[Animation {
                name: Self::CANNONBALL_ANIMATION_ROLLING.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            true,
        );

        let speed = if facing {
            vec2(
                Self::CANNONBALL_INITIAL_SPEED_X_REL,
                Self::CANNONBALL_INITIAL_SPEED_Y,
            )
        } else {
            vec2(
                -Self::CANNONBALL_INITIAL_SPEED_X_REL,
                Self::CANNONBALL_INITIAL_SPEED_Y,
            )
        };

        let mut body = PhysicsBody::new(
            &mut resources.collision_world,
            pos,
            0.0,
            vec2(Self::CANNONBALL_WIDTH, Self::CANNONBALL_HEIGHT),
        );
        body.speed = speed;

        /*PhysicsBody {
            pos,
            facing,
            angle: 0.0,
            speed,
            collider: None,
            on_ground: false,
            last_frame_on_ground: false,
            have_gravity: true,
            bouncyness: 1.0,
        };*/

        /*let cannonball_mount_pos = if facing {
            vec2(Self::CANNONBALL_MOUNT_X_REL, Self::CANNONBALL_MOUNT_Y)
        } else {
            vec2(-Self::CANNONBALL_MOUNT_X_REL, Self::CANNONBALL_MOUNT_Y)
        };*/

        /*body.collider = Some(resources.collision_world.add_actor(
            body.pos + cannonball_mount_pos,
            CANNONBALL_WIDTH as i32,
            CANNONBALL_HEIGHT as i32,
        ));*/

        Self {
            cannonball_sprite,
            body,
            lived: 0.0,
            countdown: Self::CANNONBALL_COUNTDOWN_DURATION,
            owner_id,
            owner_safe_countdown: Self::CANNONBALL_OWNER_SAFE_TIME,
            //erupting: false,
            //erupting_enable_on_y: None,
        }
    }

    pub fn spawn(pos: Vec2, facing: bool, owner_id: u8) {
        let cannonball = Cannonball::new(pos, facing, owner_id);
        scene::add_node(cannonball);
    }
}

/*impl EruptedItem for Cannonball {
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
}*/

impl scene::Node for Cannonball {
    fn fixed_update(mut cannonball: RefMut<Self>) {
        /*if cannonball.erupting {
            let cannonball_enabled = cannonball.eruption_update();

            if !cannonball_enabled {
                return;
            }
        }*/

        cannonball.body.update();
        cannonball.lived += get_frame_time();
        cannonball.owner_safe_countdown -= get_frame_time();

        let explosion_position =
            cannonball.body.pos + vec2(Self::CANNONBALL_WIDTH / 2., Self::CANNONBALL_HEIGHT / 2.);

        let mut explode = false;

        if cannonball.lived < cannonball.countdown {
            for player in
                crate::utils::player_circle_hit(explosion_position, Self::EXPLOSION_RADIUS)
            {
                if player.id != cannonball.owner_id || cannonball.owner_safe_countdown < 0. {
                    explode = true;
                    break;
                }
            }
        } else {
            explode = true;
        }
        if explode {
            crate::utils::explode(explosion_position, Self::EXPLOSION_RADIUS);
            cannonball.delete();
        }
    }

    fn draw(mut cannonball: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        cannonball.cannonball_sprite.update();

        draw_texture_ex(
            resources.items_textures["cannon/cannonball"],
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
