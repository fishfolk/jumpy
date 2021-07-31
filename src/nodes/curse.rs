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

const CURSE_WIDTH: f32 = 32.;
const CURSE_HEIGHT: f32 = 32.;
const CURSE_ANIMATION_BASE: &'static str = "base";
const CURSE_MOUNT_X_REL: f32 = -12.;
const CURSE_MOUNT_Y: f32 = -10.;

pub struct Curse {
    curse_sprite: AnimatedSprite,

    pub thrown: bool,

    pub body: PhysicsBody,
}

impl Curse {
    pub fn new(facing: bool, pos: Vec2) -> Self {
        let curse_sprite = AnimatedSprite::new(
            CURSE_WIDTH as u32,
            CURSE_HEIGHT as u32,
            &[Animation {
                name: CURSE_ANIMATION_BASE.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            false,
        );

        Self {
            curse_sprite,
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

        if let Some(collider) = self.body.collider {
            resources
                .collision_world
                .set_actor_position(collider, self.body.pos);
        } else {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos,
                CURSE_WIDTH as i32,
                CURSE_HEIGHT as i32,
            ));
        }
    }

    pub fn shoot(node_h: Handle<Curse>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            let node = scene::get_node(node_h);
            let player = &mut *scene::get_node(player);

            // `thrown` is still required, otherwise, spawning may be called multiple times.
            if node.thrown == true {
                player.state_machine.set_state(Player::ST_NORMAL);
                return;
            }

            let mut flying_curses =
                scene::find_node_by_type::<crate::nodes::FlyingCurses>().unwrap();
            flying_curses.spawn_flying_curse(node.body.pos, node.body.facing, player.id);

            player.weapon = None;
            player.floating = false;
            node.delete();

            player.state_machine.set_state(Player::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(curse: HandleUntyped, force: bool) {
            let mut curse = scene::get_untyped_node(curse).unwrap().to_typed::<Curse>();

            Curse::throw(&mut *curse, force)
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Curse>()
                .handle();

            Curse::shoot(node, player)
        }

        fn is_thrown(curse: HandleUntyped) -> bool {
            let curse = scene::get_untyped_node(curse);

            // The item may have been shot at this stage; in this case, it's gone.
            if let Some(curse) = curse {
                curse.to_typed::<Curse>().thrown
            } else {
                false
            }
        }

        fn pick_up(node: HandleUntyped) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Curse>();

            node.body.angle = 0.;

            node.thrown = false;
        }

        capabilities::Gun {
            throw,
            shoot,
            is_thrown,
            pick_up,
        }
    }
}

impl scene::Node for Curse {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(CURSE_WIDTH, CURSE_HEIGHT),
            Self::gun_capabilities(),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.curse_sprite.update();

        if node.thrown {
            node.body.update();
            node.body.update_throw();
        }
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        let mut draw_pos = node.body.pos;

        if !node.thrown {
            draw_pos += if node.body.facing {
                vec2(CURSE_MOUNT_X_REL, CURSE_MOUNT_Y)
            } else {
                vec2(-CURSE_MOUNT_X_REL, CURSE_MOUNT_Y)
            }
        };

        draw_texture_ex(
            resources.curse,
            draw_pos.x,
            draw_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.curse_sprite.frame().source_rect),
                dest_size: Some(node.curse_sprite.frame().dest_size),
                flip_x: node.body.facing,
                rotation: node.body.angle,
                ..Default::default()
            },
        );
    }
}
