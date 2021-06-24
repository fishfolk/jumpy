use macroquad::{
    audio::play_sound_once,
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, Handle, RefMut},
    },
    prelude::*,
};

use crate::{nodes::Player, Resources};

pub struct Sword {
    pub sword_sprite: AnimatedSprite,
    pub facing: bool,
    pub pos: Vec2,
}

impl scene::Node for Sword {
    fn draw(mut sword: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        //sword.dead == false && matches!(sword.weapon, Some(Weapon::Sword)) {
        // for attack animation - old, pre-rotated sprite
        if sword.sword_sprite.current_animation() == 1 {
            let sword_mount_pos = if sword.facing {
                vec2(10., -35.)
            } else {
                vec2(-50., -35.)
            };
            draw_texture_ex(
                resources.sword,
                sword.pos.x + sword_mount_pos.x,
                sword.pos.y + sword_mount_pos.y,
                color::WHITE,
                DrawTextureParams {
                    source: Some(sword.sword_sprite.frame().source_rect),
                    dest_size: Some(sword.sword_sprite.frame().dest_size),
                    flip_x: !sword.facing,
                    ..Default::default()
                },
            );
        } else {
            // just casually holding a sword

            let sword_mount_pos = if sword.facing {
                vec2(5., 10.)
            } else {
                vec2(-45., 10.)
            };

            let rotation = if sword.facing {
                // 45 + a little bit
                -std::f32::consts::PI / 4. - 0.3
            } else {
                std::f32::consts::PI / 4. + 0.3
            };
            draw_texture_ex(
                resources.fish_sword,
                sword.pos.x + sword_mount_pos.x,
                sword.pos.y + sword_mount_pos.y,
                color::WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(65., 17.)),
                    flip_x: !sword.facing,
                    rotation: rotation, //get_time() as _,
                    ..Default::default()
                },
            );
        }
    }

    fn update(mut node: RefMut<Self>) {
        node.sword_sprite.update();
    }
}

impl Sword {
    pub fn new(facing: bool, pos: Vec2) -> Sword {
        let sword_sprite = AnimatedSprite::new(
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

        Sword {
            sword_sprite,
            pos,
            facing,
        }
    }
    pub fn shot(node: Handle<Sword>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let resources = storage::get_mut::<Resources>();
                play_sound_once(resources.sword_sound);

                let sword = &mut *scene::get_node(node);
                sword.sword_sprite.set_animation(1);
            }

            {
                let player = &mut *scene::get_node(player);
                let others = scene::find_nodes_by_type::<crate::nodes::Player>();
                let sword_hit_box = if player.fish.facing {
                    Rect::new(player.pos().x + 35., player.pos().y - 5., 40., 60.)
                } else {
                    Rect::new(player.pos().x - 50., player.pos().y - 5., 40., 60.)
                };

                for mut other in others {
                    if Rect::new(other.pos().x, other.pos().y, 20., 64.).overlaps(&sword_hit_box) {
                        other.kill(!player.fish.facing);
                    }
                }
            }

            for i in 0u32..3 {
                {
                    let sword = &mut *scene::get_node(node);
                    sword.sword_sprite.set_frame(i);
                }

                wait_seconds(0.08).await;
            }

            {
                let mut sword = scene::get_node(node);
                sword.sword_sprite.set_animation(0);
            }

            let player = &mut *scene::get_node(player);
            player.state_machine.set_state(Player::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }
}
