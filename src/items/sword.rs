use macroquad::{
    audio::play_sound_once,
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
    capabilities,
    components::{PhysicsBody, ThrowableItem},
    nodes::Player,
    GameWorld, Resources,
};

pub struct Sword {
    sprite: AnimatedSprite,
    body: PhysicsBody,
    throwable: ThrowableItem,
    origin_pos: Vec2,

    deadly_dangerous: bool,

    // hack, just for swordthrow loc
    spawn_pos: (Vec2, bool),
}

impl scene::Node for Sword {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::gun_capabilities());
        node.provides(Self::physics_capabilities());
    }

    fn draw(sword: RefMut<Self>) {
        let resources = storage::get::<Resources>();

        //sword.dead == false && matches!(sword.weapon, Some(Weapon::Sword)) {
        // for attack animation - old, pre-rotated sprite
        if sword.sprite.current_animation() == 1 {
            // sword attack animation spritesheet is very different
            // from just a sword, it has different size and rotation
            // this little hack compensates this
            let hack_offset = vec2(0.0, -35.);

            let texture_entry = resources.textures.get("sword").unwrap();

            draw_texture_ex(
                texture_entry.texture,
                sword.body.pos.x + hack_offset.x,
                sword.body.pos.y + hack_offset.y,
                color::WHITE,
                DrawTextureParams {
                    source: Some(sword.sprite.frame().source_rect),
                    dest_size: Some(sword.sprite.frame().dest_size),
                    flip_x: !sword.body.facing,
                    ..Default::default()
                },
            );
        } else {
            // just casually holding a sword

            let rotation = if sword.body.facing {
                -sword.body.angle
            } else {
                sword.body.angle
            };

            let texture_entry = resources.textures.get("sword_held").unwrap();

            draw_texture_ex(
                texture_entry.texture,
                sword.body.pos.x,
                sword.body.pos.y,
                color::WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(65., 17.)),
                    flip_x: !sword.body.facing,
                    rotation, //get_time() as _,
                    ..Default::default()
                },
            );
        }

        //let pos = resources.collision_world.actor_pos(collider);

        // let sword_hit_box = Rect::new(pos.x, pos.y, 40., 30.);
        // draw_rectangle(
        //     sword_hit_box.x,
        //     sword_hit_box.y,
        //     sword_hit_box.w,
        //     sword_hit_box.h,
        //     RED,
        // );
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.sprite.update();

        let map_bottom = {
            let world = storage::get::<GameWorld>();

            world.map.grid_size.y as f32 * world.map.tile_size.y
        } as f32;

        // respawn sword
        // should not be here, just a hack for swordthrow loc
        if node.body.pos.y > map_bottom {
            node.body.pos = node.spawn_pos.0;
            node.body.facing = node.spawn_pos.1;
            node.body.speed = vec2(0., 0.);
            node.deadly_dangerous = false;
            node.throw(false);
            return;
        }

        let node = &mut *node;
        node.throwable.update(&mut node.body, false);

        if node.throwable.thrown() {
            if (node.origin_pos - node.body.pos).length() > 70. {
                node.deadly_dangerous = true;
            }
            if node.body.speed.length() <= 200.0 {
                node.deadly_dangerous = false;
            }
            if node.body.on_ground && node.body.speed.length() <= 400.0 {
                node.deadly_dangerous = false;
            }

            if node.deadly_dangerous {
                let others = scene::find_nodes_by_type::<crate::nodes::Player>();
                let sword_hit_box = Rect::new(node.body.pos.x - 10., node.body.pos.y, 60., 30.);

                for mut other in others {
                    if Rect::new(other.body.pos.x, other.body.pos.y, 20., 64.)
                        .overlaps(&sword_hit_box)
                    {
                        other.kill(!node.body.facing);
                    }
                }
            }
        }
    }
}

impl Sword {
    pub const COLLIDER_WIDTH: f32 = 48.0;
    pub const COLLIDER_HEIGHT: f32 = 32.0;

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let sprite = AnimatedSprite::new(
            65,
            93,
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
                    frames: 4,
                    fps: 15,
                },
            ],
            false,
        );

        let mut world = storage::get_mut::<GameWorld>();

        scene::add_node(Sword {
            sprite,
            body: PhysicsBody::new(
                &mut world.collision_world,
                pos,
                std::f32::consts::PI / 4. + 0.3,
                vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT),
            ),
            throwable: ThrowableItem::default(),
            origin_pos: pos,
            deadly_dangerous: false,
            spawn_pos: (pos, true),
        })
        .untyped()
    }

    pub fn throw(&mut self, force: bool) {
        self.throwable.throw(&mut self.body, force);

        self.origin_pos = self.body.pos;
    }

    pub fn shoot(node: Handle<Sword>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let resources = storage::get_mut::<Resources>();
                play_sound_once(resources.sword_sound);

                let sword = &mut *scene::get_node(node);
                sword.sprite.set_animation(1);
            }

            {
                let player = &mut *scene::get_node(player);
                let others = scene::find_nodes_by_type::<crate::nodes::Player>();
                let sword_hit_box = if player.body.facing {
                    Rect::new(player.body.pos.x + 35., player.body.pos.y - 5., 40., 60.)
                } else {
                    Rect::new(player.body.pos.x - 50., player.body.pos.y - 5., 40., 60.)
                };

                for mut other in others {
                    if Rect::new(other.body.pos.x, other.body.pos.y, 20., 64.)
                        .overlaps(&sword_hit_box)
                    {
                        scene::find_node_by_type::<crate::nodes::Camera>()
                            .unwrap()
                            .shake_noise(2., 6, 1.0);
                        other.kill(!player.body.facing);
                    }
                }
            }

            for i in 0u32..3 {
                {
                    let sword = &mut *scene::get_node(node);
                    sword.sprite.set_frame(i);
                }

                wait_seconds(0.08).await;
            }

            {
                let mut sword = scene::get_node(node);
                sword.sprite.set_animation(0);
            }

            let player = &mut *scene::get_node(player);
            player.state_machine.set_state(Player::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }

    pub fn gun_capabilities() -> capabilities::Weapon {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Sword>();

            Sword::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Sword>()
                .handle();

            Sword::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Sword>();

            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Sword>();

            node.body.angle = std::f32::consts::PI / 4. + 0.3;
            node.throwable.owner = Some(owner);
        }

        fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Sword>();

            let sword_mount_pos = if parent_facing {
                vec2(5., 10.)
            } else {
                vec2(-45., 10.)
            };

            node.body.pos = parent_pos + sword_mount_pos;
            node.body.facing = parent_facing;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Sword>();

            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                Sword::COLLIDER_WIDTH,
                Sword::COLLIDER_HEIGHT,
            )
        }

        capabilities::Weapon {
            collider,
            mount,
            is_thrown,
            pick_up,
            throw,
            shoot,
        }
    }

    fn physics_capabilities() -> capabilities::PhysicsObject {
        fn active(handle: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(handle).unwrap().to_typed::<Sword>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle).unwrap().to_typed::<Sword>();

            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                node.body.size.x,
                node.body.size.y,
            )
        }
        fn set_speed_x(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle).unwrap().to_typed::<Sword>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle).unwrap().to_typed::<Sword>();
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
