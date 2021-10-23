use macroquad::{
    experimental::{
        animation::{AnimatedSprite, Animation},
        scene,
        scene::{Node, RefMut},
    },
    math::{vec2, Vec2},
    prelude::Rect,
};

use super::explosive::{DetonationParameters, Explosive};

pub struct ArmedKickBomb {
    explosive: Explosive,
}

impl ArmedKickBomb {
    pub const COLLIDER_WIDTH: f32 = 32.0;
    pub const COLLIDER_HEIGHT: f32 = 36.0;
    pub const KICK_SPEED_THRESHOLD: f32 = 150.0;
    pub const KICK_FULL_FORCE: f32 = 900.0;
    pub const KICK_NUDGE_FORCE: f32 = 50.0;

    pub fn new(pos: Vec2) -> Self {
        let sprite = AnimatedSprite::new(
            Self::COLLIDER_WIDTH as u32,
            Self::COLLIDER_HEIGHT as u32,
            &[Animation {
                name: "idle".to_string(),
                row: 1,
                frames: 4,
                fps: 8,
            }],
            false,
        );

        ArmedKickBomb {
            explosive: Explosive::new(
                pos,
                vec2(0., 0.),
                DetonationParameters {
                    trigger_radius: None,
                    owner_safe_fuse: 0.,
                    explosion_radius: 100.,
                    fuse: Some(2.),
                },
                "kickbomb/bomb",
                sprite,
                vec2(
                    ArmedKickBomb::COLLIDER_WIDTH,
                    ArmedKickBomb::COLLIDER_HEIGHT,
                ),
                0,
            ),
        }
    }

    pub fn spawn(pos: Vec2) {
        let kick_bomb = ArmedKickBomb::new(pos);
        scene::add_node(kick_bomb);
    }
}

impl Node for ArmedKickBomb {
    fn fixed_update(mut node: RefMut<Self>) {
        let hitbox = Rect::new(
            node.explosive.body.pos.x,
            node.explosive.body.pos.y,
            ArmedKickBomb::COLLIDER_WIDTH,
            ArmedKickBomb::COLLIDER_HEIGHT,
        );
        for player in scene::find_nodes_by_type::<crate::nodes::Player>() {
            if hitbox.overlaps(&player.get_hitbox()) {
                let direction = node.explosive.body.pos.x > (player.body.pos.x + 10.);
                let mut speed: f32 = 0.0;
                if direction == player.body.facing {
                    if player.body.speed.x >= ArmedKickBomb::KICK_SPEED_THRESHOLD
                        || player.body.speed.x <= -ArmedKickBomb::KICK_SPEED_THRESHOLD
                    {
                        speed = ArmedKickBomb::KICK_FULL_FORCE;
                    } else if player.body.facing == (node.explosive.body.speed.x < 0.) {
                        // Only put the bomb down if player is facing opposite to the bomb's direction (can't catch with back)
                        speed = ArmedKickBomb::KICK_NUDGE_FORCE;
                    }
                }
                if speed > 0. {
                    node.explosive.body.speed.y = -speed / 3.;
                    node.explosive.body.speed.x = if direction { speed } else { -speed }
                }
            }
        }
        if Explosive::update(&mut node.explosive) {
            node.delete();
        }
    }

    fn draw(mut node: RefMut<Self>) {
        Explosive::draw(&mut node.explosive);
    }
}