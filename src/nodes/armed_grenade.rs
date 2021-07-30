use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::{Node, RefMut},
    },
    prelude::*,
};

use crate::{nodes::player::PhysicsBody, nodes::sproinger::Sproingable, Resources};

use super::EruptedItem;

pub struct ArmedGrenade {
    grenade_sprite: AnimatedSprite,
    pub body: PhysicsBody,
    lived: f32,
    /// True if erupting from a volcano
    erupting: bool,
    /// When erupting, enable the collider etc. after passing this coordinate on the way down. Set/valid
    /// only when erupting.
    erupting_enable_on_y: Option<f32>,
}

impl ArmedGrenade {
    pub const COUNTDOWN_DURATION: f32 = 0.5;
    pub const EXPLOSION_WIDTH: f32 = 100.0;
    pub const EXPLOSION_HEIGHT: f32 = 100.0;

    pub fn new(pos: Vec2, facing: bool) -> Self {
        // TODO: In case we want to animate thrown grenades rotating etc.
        let grenade_sprite = AnimatedSprite::new(
            15,
            15,
            &[Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            false,
        );

        let speed = if facing {
            vec2(600., -200.)
        } else {
            vec2(-600., -200.)
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
            bouncyness: 0.5,
        };

        let mut resources = storage::get_mut::<Resources>();

        let grenade_mount_pos = if facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        body.collider = Some(resources.collision_world.add_actor(
            body.pos + grenade_mount_pos,
            15,
            15,
        ));

        ArmedGrenade {
            grenade_sprite,
            body,
            lived: 0.0,
            erupting: false,
            erupting_enable_on_y: None,
        }
    }

    pub fn spawn(pos: Vec2, facing: bool) {
        let grenade = ArmedGrenade::new(pos, facing);
        scene::add_node(grenade);
    }
}

impl EruptedItem for ArmedGrenade {
    fn spawn_for_volcano(pos: Vec2, speed: Vec2, enable_at_y: f32) {
        let mut grenade = ArmedGrenade::new(pos, true);

        grenade.lived -= 2.; // give extra life, since they're random
        grenade.body.speed = speed;
        grenade.body.collider = None;
        grenade.erupting = true;
        grenade.erupting_enable_on_y = Some(enable_at_y);

        scene::add_node(grenade);
    }

    fn body(&mut self) -> &mut PhysicsBody {
        &mut self.body
    }
    fn enable_at_y(&self) -> f32 {
        self.erupting_enable_on_y.unwrap()
    }
}

impl Node for ArmedGrenade {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Sproingable>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(16.0, 32.0),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        if node.erupting {
            let node_enabled = node.eruption_update();

            if !node_enabled {
                return;
            }
        }

        node.body.update();

        node.lived += get_frame_time();

        if node.lived >= ArmedGrenade::COUNTDOWN_DURATION {
            {
                let mut resources = storage::get_mut::<Resources>();
                resources.hit_fxses.spawn(node.body.pos);
            }
            let grenade_rect = Rect::new(
                node.body.pos.x - (ArmedGrenade::EXPLOSION_WIDTH / 2.0),
                node.body.pos.y - (ArmedGrenade::EXPLOSION_HEIGHT / 2.0),
                ArmedGrenade::EXPLOSION_WIDTH,
                ArmedGrenade::EXPLOSION_HEIGHT,
            );
            for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                let intersect = grenade_rect.intersect(player.get_hitbox());
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
            resources.grenades,
            node.body.pos.x,
            node.body.pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.grenade_sprite.frame().source_rect),
                dest_size: Some(node.grenade_sprite.frame().dest_size),
                flip_x: false,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}
