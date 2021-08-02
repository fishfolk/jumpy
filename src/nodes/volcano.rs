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
    EruptingVolcano, Player,
};

const VOLCANO_WIDTH: f32 = 36.;
const VOLCANO_HEIGHT: f32 = 22.;
const VOLCANO_ANIMATION_BASE: &'static str = "base";
const VOLCANO_MOUNT_X_REL: f32 = -12.;
const VOLCANO_MOUNT_Y: f32 = -10.;

pub struct Volcano {
    volcano_sprite: AnimatedSprite,

    pub thrown: bool,
    pub used: bool,

    pub body: PhysicsBody,
}

impl Volcano {
    pub fn new(facing: bool, pos: Vec2) -> Self {
        let volcano_sprite = AnimatedSprite::new(
            VOLCANO_WIDTH as u32,
            VOLCANO_HEIGHT as u32,
            &[Animation {
                name: VOLCANO_ANIMATION_BASE.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            false,
        );

        Self {
            volcano_sprite,
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

        if self.body.collider.is_none() {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos,
                VOLCANO_WIDTH as i32,
                VOLCANO_HEIGHT as i32,
            ));
        } else {
            resources
                .collision_world
                .set_actor_position(self.body.collider.unwrap(), self.body.pos);
        }
    }

    pub fn shoot(volcano: Handle<Volcano>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            let mut volcano = scene::get_node(volcano);
            let player = &mut *scene::get_node(player);

            // `used` is still required, otherwise, spawning may be called multiple times.
            if volcano.used {
                player.state_machine.set_state(Player::ST_NORMAL);
                return;
            }

            volcano.used = true;

            EruptingVolcano::spawn(player.id);

            player.weapon = None;
            player.floating = false;

            volcano.delete();

            player.state_machine.set_state(Player::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(volcano: HandleUntyped, force: bool) {
            let mut volcano = scene::get_untyped_node(volcano)
                .unwrap()
                .to_typed::<Volcano>();

            Volcano::throw(&mut *volcano, force)
        }

        fn shoot(volcano: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let volcano = scene::get_untyped_node(volcano)
                .unwrap()
                .to_typed::<Volcano>()
                .handle();

            Volcano::shoot(volcano, player)
        }

        fn is_thrown(volcano: HandleUntyped) -> bool {
            let volcano = scene::get_untyped_node(volcano);

            // The item may have been shot at this stage; in this case, it's gone.
            if let Some(volcano) = volcano {
                volcano.to_typed::<Volcano>().thrown
            } else {
                false
            }
        }

        fn pick_up(volcano: HandleUntyped) {
            let mut volcano = scene::get_untyped_node(volcano)
                .unwrap()
                .to_typed::<Volcano>();

            volcano.body.angle = 0.;

            volcano.thrown = false;
        }

        capabilities::Gun {
            throw,
            shoot,
            is_thrown,
            pick_up,
        }
    }
}

impl scene::Node for Volcano {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(VOLCANO_WIDTH, VOLCANO_HEIGHT),
            Self::gun_capabilities(),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.volcano_sprite.update();

        if node.thrown {
            node.body.update();
            node.body.update_throw();
        }
    }

    fn draw(volcano: RefMut<Self>) {
        let texture = storage::get_mut::<Resources>().volcano_icon;

        let mut draw_pos = volcano.body.pos;

        if !volcano.thrown {
            draw_pos += if volcano.body.facing {
                vec2(VOLCANO_MOUNT_X_REL, VOLCANO_MOUNT_Y)
            } else {
                vec2(-VOLCANO_MOUNT_X_REL, VOLCANO_MOUNT_Y)
            }
        };

        draw_texture_ex(
            texture,
            draw_pos.x,
            draw_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(volcano.volcano_sprite.frame().source_rect),
                dest_size: Some(volcano.volcano_sprite.frame().dest_size),
                flip_x: volcano.body.facing,
                rotation: volcano.body.angle,
                ..Default::default()
            },
        );
    }
}
