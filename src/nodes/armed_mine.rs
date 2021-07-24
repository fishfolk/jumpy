use macroquad::{
    experimental::{
        collections::storage,
        scene::RefMut,
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

pub struct ArmedMine {
    mine_sprite: AnimatedSprite,
    pub body: PhysicsBody,
    lived: f32,
}

impl ArmedMine {
    pub const ARMED_AFTER_DURATION: f32 = 0.75;
    pub const TRIGGER_WIDTH: f32 = 30.0;
    pub const TRIGGER_HEIGHT: f32 = 15.0;

    pub fn new(pos: Vec2, facing: bool) -> Self {
        // TODO: In case we want to animate thrown grenades rotating etc.
        let mine_sprite = AnimatedSprite::new(
            30,
            15,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                },
                Animation {
                    name: "armed".to_string(),
                    row: 0,
                    frames: 2,
                    fps: 3,
                },
            ],
            false,
        );

        let speed = if facing {
            vec2(150., -200.)
        } else {
            vec2(-150., -200.)
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
            bouncyness: 0.0,
        };

        let mut resources = storage::get_mut::<Resources>();

        let grenade_mount_pos = if facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        body.collider = Some(resources.collision_world.add_actor(
            body.pos + grenade_mount_pos,
            30,
            15,
        ));

        ArmedMine {
            mine_sprite,
            body,
            lived: 0.0,
        }
    }

    pub fn spawn(pos: Vec2, facing: bool) {
        let mine = ArmedMine::new(pos, facing);
        scene::add_node(mine);
    }
}

impl scene::Node for ArmedMine {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Sproingable>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(32.0, 16.0),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.body.update();
        node.lived += get_frame_time();

        // TODO: Fix animation
        if node.lived >= ArmedMine::ARMED_AFTER_DURATION && node.mine_sprite.current_animation() != 1 {
            node.mine_sprite.set_animation(1);
            node.mine_sprite.playing = true;
        }

        if node.body.on_ground {
            node.body.speed = Vec2::ZERO;
        }

        if node.lived >= ArmedMine::ARMED_AFTER_DURATION {
            let mut killed = false;
            let mine_rect = Rect::new(
                node.body.pos.x - (ArmedMine::TRIGGER_WIDTH / 2.0),
                node.body.pos.y - (ArmedMine::TRIGGER_HEIGHT / 2.0),
                ArmedMine::TRIGGER_WIDTH,
                ArmedMine::TRIGGER_HEIGHT,
            );
            for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                let intersect =
                    mine_rect.intersect(Rect::new(
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
                    killed = true;
                }
            }
            if killed {
                let mut resources = storage::get_mut::<Resources>();
                resources.hit_fxses.spawn(node.body.pos);
                node.delete();
            }
        }
    }

    fn draw(mut node: RefMut<Self>) {
        node.mine_sprite.update();

        if node.lived >= ArmedMine::ARMED_AFTER_DURATION && node.mine_sprite.current_animation() != 1 {
            node.mine_sprite.set_animation(1);
        }

        let resources = storage::get_mut::<Resources>();
        draw_texture_ex(
            resources.mines,
            node.body.pos.x,
            node.body.pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.mine_sprite.frame().source_rect),
                dest_size: Some(node.mine_sprite.frame().dest_size),
                flip_x: false,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}
