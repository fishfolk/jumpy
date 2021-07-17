use macroquad::{
    //audio::play_sound_once,
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, Handle, HandleUntyped, RefMut},
    },
    prelude::*,
};

use crate::{
    nodes::{
        player::{capabilities, PhysicsBody, Weapon},
        Player,
    },
    Resources,
};

pub struct Mines {
    mines_sprite: AnimatedSprite,

    pub thrown: bool,

    pub amount: i32,
    pub body: PhysicsBody,

    origin_pos: Vec2,
    deadly_dangerous: bool,
}

impl Mines {
    pub const INITIAL_AMOUNT: i32 = 3;
    pub const MAXIMUM_AMOUNT: i32 = 3;

    pub fn new(facing: bool, pos: Vec2) -> Self {
        let mines_sprite = AnimatedSprite::new(
            30,
            15,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                },
                Animation {
                    name: "shoot".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                },
            ],
            false,
        );

        Mines {
            mines_sprite,
            body: PhysicsBody {
                pos,
                facing,
                angle: 0.0,
                speed: vec2(0., 0.),
                collider: None,
                on_ground: false,
                last_frame_on_ground: false,
                have_gravity: true,
            },
            thrown: false,
            amount: Self::INITIAL_AMOUNT,
            origin_pos: pos,
            deadly_dangerous: false,
        }
    }

    fn draw_hud(&self) {
        let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
        let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);
        for i in 0..Self::MAXIMUM_AMOUNT {
            let x = self.body.pos.x + 15.0 * i as f32;

            if i >= self.amount {
                draw_circle_lines(x, self.body.pos.y - 12.0, 4.0, 2., empty_color);
            } else {
                draw_circle(x, self.body.pos.y - 12.0, 4.0, full_color);
            };
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

        let mines_mount_pos = if self.body.facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        if self.body.collider.is_none() {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos + mines_mount_pos,
                15,
                30,
            ));
        } else {
            resources
                .collision_world
                .set_actor_position(self.body.collider.unwrap(), self.body.pos + mines_mount_pos);
        }
        self.origin_pos = self.body.pos + mines_mount_pos / 2.;
    }

    pub fn shoot(node: Handle<Mines>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let node = scene::get_node(node);
                if node.amount <= 0 {
                    let player = &mut *scene::get_node(player);
                    player.state_machine.set_state(Player::ST_NORMAL);

                    return;
                }
            }

            {
                //let resources = storage::get_mut::<Resources>();
                //play_sound_once(resources.shoot_sound);

                let node = scene::get_node(node);

                let mut mines = scene::find_node_by_type::<crate::nodes::ArmedMines>().unwrap();
                mines.spawn_mine(node.body.pos, node.body.facing);
            }
            {
                let node = &mut *scene::get_node(node);
                node.mines_sprite.set_animation(1);
            }
            {
                let node = &mut *scene::get_node(node);
                node.mines_sprite.set_frame(0);
            }

            wait_seconds(0.08).await;

            {
                let mut node = scene::get_node(node);
                node.mines_sprite.set_animation(0);
            }

            {
                let mut node = scene::get_node(node);
                node.amount -= 1;
            }

            {
                let player = &mut *scene::get_node(player);
                // node.weapon_animation.play(0, 0..5).await;
                // node.weapon_animation.play(0, 5..).await;
                player.state_machine.set_state(Player::ST_NORMAL);
            }
        };

        start_coroutine(coroutine)
    }

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Mines>();

            Mines::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Mines>()
                .handle();

            Mines::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Mines>();

            node.thrown
        }

        fn pick_up(node: HandleUntyped) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Mines>();

            node.body.angle = 0.;
            node.amount = 3;
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

impl scene::Node for Mines {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            Self::gun_capabilities(),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.mines_sprite.update();

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
                let mines_hit_box = Rect::new(node.body.pos.x - 7.5, node.body.pos.y, 15., 15.);

                for mut other in others {
                    if Rect::new(other.body.pos.x, other.body.pos.y, 20., 64.)
                        .overlaps(&mines_hit_box)
                    {
                        other.kill(!node.body.facing);
                    }
                }
            }
        }
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        let mine_mount_pos = if node.thrown == false {
            if node.body.facing {
                vec2(0., 16.)
            } else {
                vec2(-5., 16.)
            }
        } else {
            if node.body.facing {
                vec2(-25., 0.)
            } else {
                vec2(5., 0.)
            }
        };

        draw_texture_ex(
            resources.mines,
            node.body.pos.x + mine_mount_pos.x,
            node.body.pos.y + mine_mount_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.mines_sprite.frame().source_rect),
                dest_size: Some(node.mines_sprite.frame().dest_size),
                flip_x: !node.body.facing,
                rotation: node.body.angle,
                ..Default::default()
            },
        );

        if node.thrown == false {
            node.draw_hud();
        }
    }
}
