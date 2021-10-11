use std::{cell::RefCell, convert::TryInto};

use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene,
        scene::{Node, RefMut},
    },
    math::{vec2, Vec2},
    prelude::Rect,
};

use crate::{
    capabilities::Weapon, components::PhysicsBody, nodes::player::Player, GameWorld, Resources,
};

use super::explosive::{self, DetonationParameters, Explosive};

pub struct ArmedKickBomb {
    explosive: Explosive,
}

impl ArmedKickBomb {
    pub const COLLIDER_WIDTH: f32 = 30.0;
    pub const COLLIDER_HEIGHT: f32 = 30.0;
    pub const KICK_SPEED_THRESHOLD: f32 = 150.0;
    pub const KICK_FORCE: f32 = 900.0;
    pub const COUNTDOWN_DURATION: f32 = 3.0;
    pub const EXPLOSION_RADIUS: f32 = 100.0;

    pub fn new(pos: Vec2, facing: bool) -> Self {
        let sprite = AnimatedSprite::new(
            28,
            38,
            &[Animation {
                name: "idle".to_string(),
                row: 1,
                frames: 4,
                fps: 8,
            }],
            false,
        );

        let mut world = storage::get_mut::<GameWorld>();

        let mount_pos = if facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        ArmedKickBomb {
            explosive: Explosive::new(
                pos,
                vec2(100., 0.),
                DetonationParameters {
                    trigger_radius: None,
                    owner_safe_fuse: 0.,
                    explosion_radius: 100.,
                    fuse: Some(100.),
                },
                "kickbomb/kickbombs",
                sprite,
                vec2(
                    ArmedKickBomb::COLLIDER_WIDTH,
                    ArmedKickBomb::COLLIDER_HEIGHT,
                ),
                0,
            ),
        }
    }

    pub fn spawn(pos: Vec2, facing: bool) {
        let kick_bomb = ArmedKickBomb::new(pos, facing);
        scene::add_node(kick_bomb);
    }
}

impl Node for ArmedKickBomb {
    fn fixed_update(mut node: RefMut<Self>) {
        for mut player in explosive::player_circle_hit(node.explosive.body.pos, 10.) {
            if let Some(weapon) = player.weapon.as_mut() {
                (scene::get_untyped_node(weapon.node)
                    .unwrap()
                    .to_typed::<Weapon>()
                    .throw)(weapon.node, true);
                player.weapon = None;
            }
        }

        let hitbox = Rect::new(
            node.explosive.body.pos.x,
            node.explosive.body.pos.y,
            ArmedKickBomb::COLLIDER_WIDTH,
            ArmedKickBomb::COLLIDER_HEIGHT,
        );
        for player in scene::find_nodes_by_type::<crate::nodes::Player>() {
            if player.body.speed.x >= ArmedKickBomb::KICK_SPEED_THRESHOLD
                || player.body.speed.x <= -ArmedKickBomb::KICK_SPEED_THRESHOLD
            {
                let is_overlapping = hitbox.overlaps(&player.get_hitbox());
                if is_overlapping && hitbox.y + 36.0 >= player.body.pos.y + Player::LEGS_THRESHOLD {
                    let direction = node.explosive.body.pos.x > (player.body.pos.x + 10.);
                    if direction == player.body.facing {
                        node.explosive.body.speed.x = if direction {
                            Self::KICK_FORCE
                        } else {
                            -Self::KICK_FORCE
                        }
                    }
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
