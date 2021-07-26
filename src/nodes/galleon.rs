use macroquad::{
    color,
    prelude::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, Coroutine},
        draw_texture_ex,
        scene::{self, Handle, HandleUntyped, RefMut},
        vec2, DrawTextureParams, Vec2,
    },
};

use crate::Resources;

use super::{
    player::{capabilities, PhysicsBody, Weapon},
    Player,
};

const GALLEON_WIDTH: f32 = 32.;
const GALLEON_HEIGHT: f32 = 29.;
const GALLEON_ANIMATION_BASE: &'static str = "base";
const GALLEON_MOUNT_X_REL: f32 = -12.;
const GALLEON_MOUNT_Y: f32 = -10.;

pub struct Galleon {
    galleon_sprite: AnimatedSprite,

    pub thrown: bool,

    pub body: PhysicsBody,
}

impl Galleon {
    pub fn new(facing: bool, pos: Vec2) -> Self {
        let galleon_sprite = AnimatedSprite::new(
            GALLEON_WIDTH as u32,
            GALLEON_HEIGHT as u32,
            &[Animation {
                name: GALLEON_ANIMATION_BASE.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            false,
        );

        Self {
            galleon_sprite,
            body: PhysicsBody {
                pos,
                facing,
                angle: 0.0,
                speed: vec2(0., 0.),
                collider: None,
                on_ground: false,
                last_frame_on_ground: false,
                have_gravity: true,
                bouncyness: 0.0,
            },
            thrown: false,
        }
    }

    pub fn throw(&mut self, force: bool) {
        self.thrown = true;

        if force {
            self.body.speed = if self.body.facing {
                vec2(600., -200.)
            } else {
                vec2(-600., -200.)
            };
        } else {
            self.body.angle = 3.5;
        }

        let mut resources = storage::get_mut::<Resources>();

        let galleon_mount_pos = if self.body.facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        if self.body.collider.is_none() {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos + galleon_mount_pos,
                GALLEON_WIDTH as i32,
                GALLEON_HEIGHT as i32,
            ));
        } else {
            resources.collision_world.set_actor_position(
                self.body.collider.unwrap(),
                self.body.pos + galleon_mount_pos,
            );
        }
    }

    pub fn shoot(_galleon: Handle<Galleon>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            let player = &mut *scene::get_node(player);

            player.floating = false;

            player.state_machine.set_state(Player::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(galleon: HandleUntyped, force: bool) {
            let mut galleon = scene::get_untyped_node(galleon)
                .unwrap()
                .to_typed::<Galleon>();

            Galleon::throw(&mut *galleon, force)
        }

        fn shoot(galleon: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let galleon = scene::get_untyped_node(galleon)
                .unwrap()
                .to_typed::<Galleon>()
                .handle();

            Galleon::shoot(galleon, player)
        }

        fn is_thrown(galleon: HandleUntyped) -> bool {
            let galleon = scene::get_untyped_node(galleon)
                .unwrap()
                .to_typed::<Galleon>();

            galleon.thrown
        }

        fn pick_up(galleon: HandleUntyped) {
            let mut galleon = scene::get_untyped_node(galleon)
                .unwrap()
                .to_typed::<Galleon>();

            galleon.body.angle = 0.;

            galleon.thrown = false;
        }

        capabilities::Gun {
            throw,
            shoot,
            is_thrown,
            pick_up,
        }
    }
}

impl scene::Node for Galleon {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            Self::gun_capabilities(),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.galleon_sprite.update();

        if node.thrown {
            node.body.update();
            node.body.update_throw();
        }
    }

    fn draw(galleon: RefMut<Self>) {
        let texture = storage::get_mut::<Resources>().galleon_icon;

        let mount_pos = if !galleon.thrown {
            if galleon.body.facing {
                vec2(GALLEON_MOUNT_X_REL, GALLEON_MOUNT_Y)
            } else {
                vec2(-GALLEON_MOUNT_X_REL, GALLEON_MOUNT_Y)
            }
        } else {
            if galleon.body.facing {
                vec2(-GALLEON_WIDTH, 0.)
            } else {
                vec2(GALLEON_WIDTH, 0.)
            }
        };

        draw_texture_ex(
            texture,
            galleon.body.pos.x + mount_pos.x,
            galleon.body.pos.y + mount_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(galleon.galleon_sprite.frame().source_rect),
                dest_size: Some(galleon.galleon_sprite.frame().dest_size),
                flip_x: galleon.body.facing,
                rotation: galleon.body.angle,
                ..Default::default()
            },
        );
    }
}
