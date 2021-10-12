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
    GameWorld, Resources,
};

const SNIPER_COLLIDER_WIDTH: f32 = 48.0;
const SNIPER_COLLIDER_HEIGHT: f32 = 32.0;
const SNIPER_RECOIL: f32 = 1400.0;
const SNIPER_BULLETS: i32 = 2;
const SNIPER_BULLET_SPEED: f32 = 1200.0;

impl Gun {
    pub fn spawn_sniper(pos: Vec2) -> HandleUntyped {
        let resources = storage::get::<Resources>();
        let texture_entry = resources.textures.get("sniper").unwrap();

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
            texture_entry.texture,
            SNIPER_COLLIDER_WIDTH,
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
            texture_entry.texture,
            SNIPER_COLLIDER_WIDTH,
        );

        let mut world = storage::get_mut::<GameWorld>();

        scene::add_node(Gun {
            gun_sprite,
            gun_fx_sprite,
            gun_fx: false,
            body: PhysicsBody::new(
                &mut world.collision_world,
                pos,
                0.0,
                vec2(SNIPER_COLLIDER_WIDTH, SNIPER_COLLIDER_HEIGHT),
            ),
            throwable: ThrowableItem::default(),
            bullets: SNIPER_BULLETS,
            max_bullets: SNIPER_BULLETS,
            bullet_speed: SNIPER_BULLET_SPEED,
            collider_width: SNIPER_COLLIDER_WIDTH,
            collider_height: SNIPER_COLLIDER_HEIGHT,
            recoil: SNIPER_RECOIL,
        })
        .untyped()
    }
}
