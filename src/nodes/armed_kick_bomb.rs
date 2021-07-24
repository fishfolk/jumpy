use macroquad::{
    experimental::{
        collections::storage,
        scene::{
            RefMut,
            Node,
            HandleUntyped,
            Handle,
            Lens,
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
    nodes::player::PhysicsBody,
    nodes::sproinger::Sproingable,
    Resources,
};

pub type Kickable = (HandleUntyped, Lens<PhysicsBody>, Vec2);

pub struct ArmedKickBomb {
    sprite: AnimatedSprite,
    pub body: PhysicsBody,
    lived: f32,
}

impl ArmedKickBomb {
    pub const COUNTDOWN_DURATION: f32 = 0.5;
    pub const EXPLOSION_WIDTH: f32 = 100.0;
    pub const EXPLOSION_HEIGHT: f32 = 100.0;

    pub fn new(pos: Vec2, facing: bool) -> Self {
        let sprite = AnimatedSprite::new(
            32,
            36,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
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
            vec2(30.0, 30.0),
        ));

        node.provides::<Kickable>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(30.0, 30.0),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.body.update();
        node.lived += get_frame_time();

        if node.lived >= ArmedKickBomb::COUNTDOWN_DURATION {
            {
                let mut resources = storage::get_mut::<Resources>();
                resources.hit_fxses.spawn(node.body.pos);
            }
            let kick_bomb_rect = Rect::new(
                node.body.pos.x - (ArmedKickBomb::EXPLOSION_WIDTH / 2.0),
                node.body.pos.y - (ArmedKickBomb::EXPLOSION_HEIGHT / 2.0),
                ArmedKickBomb::EXPLOSION_WIDTH,
                ArmedKickBomb::EXPLOSION_HEIGHT,
            );
            for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                let intersect =
                    kick_bomb_rect.intersect(Rect::new(
                        player.body.pos.x,
                        player.body.pos.y,
                        20.0,
                        64.0,
                    ));
                if !intersect.is_none() {
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
