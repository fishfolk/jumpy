use macroquad::{
    color,
    //audio::play_sound_once,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::{self, Node, RefMut},
    },
    prelude::*,
};

use crate::{
    components::{EruptedItem, PhysicsBody},
    GameWorld, Resources,
};

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
    pub const EXPLOSION_RADIUS: f32 = 100.0;

    pub fn new(pos: Vec2, facing: bool) -> Self {
        // TODO: In case we want to animate thrown grenades rotating etc.
        let grenade_sprite = AnimatedSprite::new(
            21,
            28,
            &[Animation {
                name: "idle".to_string(),
                row: 1,
                frames: 4,
                fps: 8,
            }],
            false,
        );

        let speed = if facing {
            vec2(600., -200.)
        } else {
            vec2(-600., -200.)
        };

        let mut world = storage::get_mut::<GameWorld>();

        let grenade_mount_pos = if facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        let size = vec2(15.0, 15.0);

        let body = PhysicsBody {
            pos,
            facing,
            angle: 0.0,
            size,
            speed,
            collider: world.collision_world.add_actor(
                pos + grenade_mount_pos,
                size.x as _,
                size.y as _,
            ),
            on_ground: false,
            last_frame_on_ground: false,
            have_gravity: true,
            bouncyness: 0.5,
        };

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
    fn spawn_for_volcano(pos: Vec2, speed: Vec2, enable_at_y: f32, _owner_id: u8) {
        let mut grenade = ArmedGrenade::new(pos, true);
        grenade.lived -= 2.; // give extra life, since they're random
        grenade.body.speed = speed;
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
    fn fixed_update(mut node: RefMut<Self>) {
        node.grenade_sprite.update();
        // the check to avoid EruptedItem and PhysicsBody to run in the same frame
        if node.erupting {
            let node_enabled = node.eruption_update();
            node.erupting = !node_enabled;

            if !node_enabled {
                return;
            }
        }

        node.body.update();

        node.lived += get_frame_time();

        if node.lived >= ArmedGrenade::COUNTDOWN_DURATION {
            {
                let mut resources = storage::get_mut::<Resources>();
                resources.hit_emitters.spawn(node.body.pos);
            }
            let grenade_circ = Circle::new(
                node.body.pos.x,
                node.body.pos.y,
                ArmedGrenade::EXPLOSION_RADIUS,
            );
            for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                if grenade_circ.overlaps_rect(&player.get_hitbox()) {
                    let direction = node.body.pos.x > (player.body.pos.x + 10.);
                    scene::find_node_by_type::<crate::nodes::Camera>().unwrap();
                    player.kill(direction);
                }
            }
            node.delete();
        }
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get::<Resources>();
        let texture_entry = resources.textures.get("grenade").unwrap();

        draw_texture_ex(
            texture_entry.texture,
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
