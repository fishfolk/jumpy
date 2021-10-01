use macroquad::{
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::{self, HandleUntyped},
    },
    prelude::*,
};

use crate::{
    components::{GunlikeAnimation, PhysicsBody, ThrowableItem},
    items::gun::Gun,
    Resources,
};

const MUSKET_COLLIDER_WIDTH: f32 = 48.0;
const MUSKET_COLLIDER_HEIGHT: f32 = 32.0;
const MUSKET_RECOIL: f32 = 700.0;
const MUSKET_BULLETS: i32 = 3;
const MUSKET_BULLET_SPEED: f32 = 500.0;

impl Gun {
    pub fn spawn_musket(pos: Vec2) -> HandleUntyped {
        let mut resources = storage::get_mut::<Resources>();

        let gun_sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                92,
                32,
                &[
                    Animation {
                        name: "idle".to_string(),
                        row: 0,
                        frames: 1,
                        fps: 1,
                    },
                    Animation {
                        name: "shoot".to_string(),
                        row: 1,
                        frames: 3,
                        fps: 15,
                    },
                ],
                false,
            ),
            resources.items_textures["musket/gun"],
            MUSKET_COLLIDER_WIDTH,
        );

        let gun_fx_sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                92,
                32,
                &[Animation {
                    name: "shoot".to_string(),
                    row: 2,
                    frames: 3,
                    fps: 15,
                }],
                false,
            ),
            resources.items_textures["sniper/gun"],
            MUSKET_COLLIDER_WIDTH,
        );

        scene::add_node(Gun {
            gun_sprite,
            gun_fx_sprite,
            gun_fx: false,
            body: PhysicsBody::new(
                &mut resources.collision_world,
                pos,
                0.0,
                vec2(MUSKET_COLLIDER_WIDTH, MUSKET_COLLIDER_HEIGHT),
            ),
            throwable: ThrowableItem::default(),
            bullets: MUSKET_BULLETS,
            max_bullets: MUSKET_BULLETS,
            bullet_speed: MUSKET_BULLET_SPEED,
            collider_width: MUSKET_COLLIDER_WIDTH,
            collider_height: MUSKET_COLLIDER_HEIGHT,
            recoil: MUSKET_RECOIL,
        })
        .untyped()
    }
}
