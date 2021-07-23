use macroquad::{
    color,
    prelude::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, Coroutine},
        draw_texture_ex,
        scene::{self, Handle, HandleUntyped, RefMut},
        vec2, DrawTextureParams, Rect, Vec2,
    },
};

use crate::Resources;

use super::{
    player::{capabilities, PhysicsBody, Weapon, PLAYER_HITBOX_HEIGHT, PLAYER_HITBOX_WIDTH},
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

    origin_pos: Vec2,
    deadly_dangerous: bool,
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
            origin_pos: pos,
            deadly_dangerous: false,
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

        let curse_mount_pos = if self.body.facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        if self.body.collider.is_none() {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos + curse_mount_pos,
                40,
                30,
            ));
        } else {
            resources
                .collision_world
                .set_actor_position(self.body.collider.unwrap(), self.body.pos + curse_mount_pos);
        }
        self.origin_pos = self.body.pos + curse_mount_pos / 2.;
    }

    pub fn shoot(node_h: Handle<Curse>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            let node = scene::get_node(node_h);
            let player = &mut *scene::get_node(player);

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
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Curse>();

            Curse::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Curse>()
                .handle();

            Curse::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Curse>();

            node.thrown
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
            Self::gun_capabilities(),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.curse_sprite.update();

        if node.thrown {
            node.body.update();
            node.body.update_throw();

            if (node.origin_pos - node.body.pos).length() > 70. {
                node.deadly_dangerous = true;
            }
            if node.body.speed.length() <= 200.0 {
                node.deadly_dangerous = false;
            }
            if node.body.on_ground {
                node.deadly_dangerous = false;
            }

            if node.deadly_dangerous {
                let others = scene::find_nodes_by_type::<crate::nodes::Player>();
                let curse_hit_box =
                    Rect::new(node.body.pos.x, node.body.pos.y, CURSE_WIDTH, CURSE_HEIGHT);

                for mut other in others {
                    if Rect::new(
                        other.body.pos.x,
                        other.body.pos.y,
                        PLAYER_HITBOX_WIDTH,
                        PLAYER_HITBOX_HEIGHT,
                    )
                    .overlaps(&curse_hit_box)
                    {
                        other.kill(!node.body.facing);
                    }
                }
            }
        }
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        let curse_mount_pos = if node.thrown == false {
            if node.body.facing {
                vec2(CURSE_MOUNT_X_REL, CURSE_MOUNT_Y)
            } else {
                vec2(-CURSE_MOUNT_X_REL, CURSE_MOUNT_Y)
            }
        } else {
            if node.body.facing {
                vec2(-CURSE_WIDTH, 0.)
            } else {
                vec2(CURSE_WIDTH, 0.)
            }
        };

        draw_texture_ex(
            resources.curse,
            node.body.pos.x + curse_mount_pos.x,
            node.body.pos.y + curse_mount_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.curse_sprite.frame().source_rect),
                dest_size: Some(node.curse_sprite.frame().dest_size),
                flip_x: !node.body.facing,
                rotation: node.body.angle,
                ..Default::default()
            },
        );
    }
}
