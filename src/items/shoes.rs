use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::{HandleUntyped, Node, RefMut},
    },
    prelude::*,
};

use crate::{
    capabilities,
    components::{PhysicsBody, ThrowableItem},
    nodes::Player,
    GameWorld, Resources,
};

pub struct Shoes {
    sprite: AnimatedSprite,
    pub body: PhysicsBody,
    pub throwable: ThrowableItem,
}

impl Shoes {
    pub const COLLIDER_WIDTH: f32 = 30.0;
    pub const COLLIDER_HEIGHT: f32 = 30.0;

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let sprite = AnimatedSprite::new(
            32,
            32,
            &[Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            false,
        );

        let mut world = storage::get_mut::<GameWorld>();

        scene::add_node(Shoes {
            sprite,
            body: PhysicsBody::new(
                &mut world.collision_world,
                pos,
                0.0,
                vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT),
            ),
            throwable: ThrowableItem::default(),
        })
        .untyped()
    }

    fn physics_capabilities() -> capabilities::PhysicsObject {
        fn active(_: HandleUntyped) -> bool {
            true
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle).unwrap().to_typed::<Shoes>();

            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                node.body.size.x,
                node.body.size.y,
            )
        }
        fn set_speed_x(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle).unwrap().to_typed::<Shoes>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle).unwrap().to_typed::<Shoes>();
            node.body.speed.y = speed;
        }

        capabilities::PhysicsObject {
            active,
            collider,
            set_speed_x,
            set_speed_y,
        }
    }
}

impl Node for Shoes {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::physics_capabilities());
    }

    fn fixed_update(mut node: scene::RefMut<Self>) {
        {
            let node = &mut *node;
            node.throwable.update(&mut node.body, false);
        }

        let shoes_rect = Rect::new(
            node.body.pos.x,
            node.body.pos.y,
            Self::COLLIDER_WIDTH,
            Self::COLLIDER_HEIGHT,
        );
        for mut player in scene::find_nodes_by_type::<Player>() {
            if player.get_hitbox().overlaps(&shoes_rect) {
                player.can_head_boink = true;
                node.delete();
                return;
            }
        }
    }

    fn draw(mut node: RefMut<Self>) {
        node.sprite.update();

        let resources = storage::get::<Resources>();
        let texture_entry = resources.textures.get("boots").unwrap();

        draw_texture_ex(
            texture_entry.texture,
            node.body.pos.x,
            node.body.pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.sprite.frame().source_rect),
                dest_size: Some(node.sprite.frame().dest_size),
                flip_x: false,
                rotation: node.body.angle,
                ..Default::default()
            },
        )
    }
}
