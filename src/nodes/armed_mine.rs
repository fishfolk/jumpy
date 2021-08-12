use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::RefMut,
    },
    prelude::*,
};

use crate::circle::Circle;
use crate::{nodes::player::PhysicsBody, nodes::sproinger::Sproingable, Resources};

pub struct ArmedMine {
    mine_sprite: AnimatedSprite,
    pub body: PhysicsBody,
    lived: f32,
}

impl ArmedMine {
    pub const ARMED_AFTER_DURATION: f32 = 0.75;
    pub const TRIGGER_RADIUS: f32 = 15.0;
    pub const EXPLOSION_RADIUS: f32 = 150.0;

    pub fn new(pos: Vec2, facing: bool) -> Self {
        // TODO: In case we want to animate thrown grenades rotating etc.
        let mine_sprite = AnimatedSprite::new(
            26,
            40,
            &[Animation {
                name: "idle".to_string(),
                row: 1,
                frames: 10,
                fps: 8,
            }],
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

        if node.lived >= ArmedMine::ARMED_AFTER_DURATION
            && node.mine_sprite.current_animation() != 1
        {
            node.mine_sprite.playing = true;
        }

        if node.body.on_ground {
            node.body.speed = Vec2::ZERO;
        }

        if node.lived >= ArmedMine::ARMED_AFTER_DURATION {
            let mut killed = false;
            let trigger = Circle::new(node.body.pos.x, node.body.pos.y, ArmedMine::TRIGGER_RADIUS);
            for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                let player_hitbox = player.get_hitbox();
                if trigger.overlaps_rect(&player_hitbox) {
                    scene::find_node_by_type::<crate::nodes::Camera>()
                        .unwrap()
                        .shake();

                    let explosion = Circle::new(
                        node.body.pos.x,
                        node.body.pos.y,
                        ArmedMine::EXPLOSION_RADIUS,
                    );
                    if explosion.overlaps_rect(&player_hitbox) {
                        let direction = node.body.pos.x > (player.body.pos.x + 10.);
                        player.kill(direction);
                        killed = true;
                    }
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

        let resources = storage::get_mut::<Resources>();
        draw_texture_ex(
            resources.mines,
            node.body.pos.x,
            node.body.pos.y - 20.0,
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
