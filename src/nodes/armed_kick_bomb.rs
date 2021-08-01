use macroquad::{
    experimental::{
        collections::storage,
        scene::{
            RefMut,
            Node,
        },
        animation::{
            AnimatedSprite,
            Animation,
        },
    },
    color,
    prelude::*,
};

use crate::{
    nodes::player::{
        Player,
        PhysicsBody,
    },
    nodes::sproinger::Sproingable,
    Resources,
};
use crate::circle::Circle;

pub struct ArmedKickBomb {
    sprite: AnimatedSprite,
    pub body: PhysicsBody,
    lived: f32,
}

impl ArmedKickBomb {
    pub const COLLIDER_WIDTH: f32 = 30.0;
    pub const COLLIDER_HEIGHT: f32 = 30.0;
    pub const KICK_SPEED_THRESHOLD: f32 = 150.0;
    pub const KICK_FORCE: f32 = 900.0;
    pub const COUNTDOWN_DURATION: f32 = 3.0;
    pub const EXPLOSION_RADIUS: f32 = 150.0;

    pub fn new(pos: Vec2, facing: bool) -> Self {
        let sprite = AnimatedSprite::new(
            32,
            36,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 1,
                    frames: 1,
                    fps: 1,
                },
            ],
            false,
        );

        let mut body = PhysicsBody {
            pos,
            facing,
            angle: 0.0,
            speed: Vec2::ZERO,
            collider: None,
            on_ground: false,
            last_frame_on_ground: false,
            have_gravity: true,
            bouncyness: 0.5,
        };

        let mut resources = storage::get_mut::<Resources>();

        let mount_pos = if facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        body.collider = Some(resources.collision_world.add_actor(
            body.pos + mount_pos,
            30,
            30,
        ));

        ArmedKickBomb {
            sprite,
            body,
            lived: 0.0,
        }
    }

    pub fn spawn(pos: Vec2, facing: bool) {
        let kick_bomb = ArmedKickBomb::new(pos, facing);
        scene::add_node(kick_bomb);
    }
}

impl Node for ArmedKickBomb {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Sproingable>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(ArmedKickBomb::COLLIDER_WIDTH, ArmedKickBomb::COLLIDER_HEIGHT),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.body.update();
        node.lived += get_frame_time();

        let hitbox = Rect::new(node.body.pos.x, node.body.pos.y, ArmedKickBomb::COLLIDER_WIDTH, ArmedKickBomb::COLLIDER_HEIGHT);

        for mut player in scene::find_nodes_by_type::<Player>() {
            if hitbox.overlaps(&player.get_hitbox()) {
                if let Some((weapon, _, _, gun)) = player.weapon.as_mut() {
                    (gun.throw)(*weapon, false);
                    player.weapon = None;
                }
            }
        }

        if node.lived < ArmedKickBomb::COUNTDOWN_DURATION {
            for player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                if player.body.speed.x >= ArmedKickBomb::KICK_SPEED_THRESHOLD || player.body.speed.x <= -ArmedKickBomb::KICK_SPEED_THRESHOLD {
                    let is_overlapping =
                        hitbox.overlaps(&player.get_hitbox());
                    if is_overlapping && hitbox.y + 36.0 >= player.body.pos.y + Player::LEGS_THRESHOLD {
                        let direction = node.body.pos.x > (player.body.pos.x + 10.);
                        if direction == player.body.facing {
                            node.body.speed.x = if direction {
                                Self::KICK_FORCE
                            } else {
                                -Self::KICK_FORCE
                            }
                        }
                    }
                }
            }
        } else {
            {
                let mut resources = storage::get_mut::<Resources>();
                resources.hit_fxses.spawn(node.body.pos);
            }
            let explosion = Circle::new(
                node.body.pos.x,
                node.body.pos.y,
                ArmedKickBomb::EXPLOSION_RADIUS,
            );
            for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                if explosion.overlaps(player.get_hitbox()) {
                    let direction = node.body.pos.x > (player.body.pos.x + 10.);
                    scene::find_node_by_type::<crate::nodes::Camera>()
                        .unwrap()
                        .shake();
                    player.kill(direction);
                }
            }
            node.delete();
        }
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();
        draw_texture_ex(
            resources.kick_bombs,
            node.body.pos.x,
            node.body.pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.sprite.frame().source_rect),
                dest_size: Some(node.sprite.frame().dest_size),
                flip_x: false,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}
