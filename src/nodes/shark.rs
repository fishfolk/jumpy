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
    Player, RainingShark,
};

const SHARK_WIDTH: f32 = 32.;
const SHARK_HEIGHT: f32 = 29.;
const SHARK_ANIMATION_BASE: &'static str = "base";
const SHARK_MOUNT_X_REL: f32 = -12.;
const SHARK_MOUNT_Y: f32 = -10.;

pub struct Shark {
    shark_sprite: AnimatedSprite,

    pub thrown: bool,
    pub used: bool,

    pub body: PhysicsBody,
}

impl Shark {
    pub fn new(facing: bool, pos: Vec2) -> Self {
        let shark_sprite = AnimatedSprite::new(
            SHARK_WIDTH as u32,
            SHARK_HEIGHT as u32,
            &[Animation {
                name: SHARK_ANIMATION_BASE.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            false,
        );

        Self {
            shark_sprite,
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
            used: false,
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

        let shark_mount_pos = if self.body.facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        if self.body.collider.is_none() {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos + shark_mount_pos,
                SHARK_WIDTH as i32,
                SHARK_HEIGHT as i32,
            ));
        } else {
            resources
                .collision_world
                .set_actor_position(self.body.collider.unwrap(), self.body.pos + shark_mount_pos);
        }
    }

    pub fn shoot(shark: Handle<Shark>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            let mut shark = scene::get_node(shark);
            let player = &mut *scene::get_node(player);

            // `used` is still required, otherwise, spawning may be called multiple times.
            if shark.used {
                player.state_machine.set_state(Player::ST_NORMAL);
                return;
            }

            shark.used = true;

            RainingShark::rain(player.id);

            player.weapon = None;
            player.floating = false;

            shark.delete();

            player.state_machine.set_state(Player::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(shark: HandleUntyped, force: bool) {
            let mut shark = scene::get_untyped_node(shark).unwrap().to_typed::<Shark>();

            Shark::throw(&mut *shark, force)
        }

        fn shoot(shark: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let shark = scene::get_untyped_node(shark)
                .unwrap()
                .to_typed::<Shark>()
                .handle();

            Shark::shoot(shark, player)
        }

        fn is_thrown(shark: HandleUntyped) -> bool {
            let shark = scene::get_untyped_node(shark);

            // The item may have been shot at this stage; in this case, it's gone.
            if let Some(shark) = shark {
                shark.to_typed::<Shark>().thrown
            } else {
                false
            }
        }

        fn pick_up(shark: HandleUntyped) {
            let mut shark = scene::get_untyped_node(shark).unwrap().to_typed::<Shark>();

            shark.body.angle = 0.;

            shark.thrown = false;
        }

        capabilities::Gun {
            throw,
            shoot,
            is_thrown,
            pick_up,
        }
    }
}

impl scene::Node for Shark {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(SHARK_WIDTH, SHARK_HEIGHT),
            Self::gun_capabilities(),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.shark_sprite.update();

        if node.thrown {
            node.body.update();
            node.body.update_throw();
        }
    }

    fn draw(shark: RefMut<Self>) {
        let texture = storage::get_mut::<Resources>().shark_icon;

        let mount_pos = if !shark.thrown {
            if shark.body.facing {
                vec2(SHARK_MOUNT_X_REL, SHARK_MOUNT_Y)
            } else {
                vec2(-SHARK_MOUNT_X_REL, SHARK_MOUNT_Y)
            }
        } else {
            if shark.body.facing {
                vec2(-SHARK_WIDTH, 0.)
            } else {
                vec2(SHARK_WIDTH, 0.)
            }
        };

        draw_texture_ex(
            texture,
            shark.body.pos.x + mount_pos.x,
            shark.body.pos.y + mount_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(shark.shark_sprite.frame().source_rect),
                dest_size: Some(shark.shark_sprite.frame().dest_size),
                flip_x: shark.body.facing,
                rotation: shark.body.angle,
                ..Default::default()
            },
        );
    }
}
