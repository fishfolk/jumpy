use macroquad::{
    experimental::{
        collections::storage,
        scene::RefMut,
        animation::{
            AnimatedSprite,
            Animation,
        },
    },
    color,
    prelude::*,
};

use crate::{
    nodes::player::PhysicsBody,
    Resources,
};

pub struct ArmedMine {
    mine_sprite: AnimatedSprite,
    pub body: PhysicsBody,
    lived: f32,
}

impl ArmedMine {
    pub const ARMED_AFTER_DURATION: f32 = 0.75;
    pub const TRIGGER_WIDTH: f32 = 30.0;
    pub const TRIGGER_HEIGHT: f32 = 15.0;

    pub fn new(pos: Vec2, facing: bool) -> Self {
        // TODO: In case we want to animate thrown grenades rotating etc.
        let mine_sprite = AnimatedSprite::new(
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
                    name: "armed".to_string(),
                    row: 0,
                    frames: 2,
                    fps: 3,
                },
            ],
            false,
        );

        let speed = if facing {
            vec2(150., -200.)
        } else {
            vec2(-150., -200.)
        };

        let mut body = PhysicsBody {
            pos,
            facing,
            angle: 0.0,
            speed,
            collider: None,
            on_ground: false,
            last_frame_on_ground: false,
            have_gravity: true,
        };

        let mut resources = storage::get_mut::<Resources>();

        let grenade_mount_pos = if facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        body.collider = Some(resources.collision_world.add_actor(
            body.pos + grenade_mount_pos,
            30,
            15,
        ));

        ArmedMine {
            mine_sprite,
            body,
            lived: 0.0,
        }
    }
}

pub struct ArmedMines {
    pub mines: Vec<ArmedMine>,
}

impl ArmedMines {
    pub fn new() -> Self {
        ArmedMines {
            mines: Vec::with_capacity(200),
        }
    }

    pub fn spawn_mine(&mut self, pos: Vec2, facing: bool) {
        self.mines.push(ArmedMine::new(pos, facing));
    }
}

impl scene::Node for ArmedMines {
    fn fixed_update(mut node: RefMut<Self>) {
        for mine in &mut node.mines {
            mine.body.update();
            mine.lived += get_frame_time();

            // TODO: Fix animation
            if mine.lived >= ArmedMine::ARMED_AFTER_DURATION && mine.mine_sprite.current_animation() != 1 {
                mine.mine_sprite.set_animation(1);
                mine.mine_sprite.playing = true;
            }

            if mine.body.on_ground {
                mine.body.speed = Vec2::ZERO;
            }
        }

        node.mines.retain(|mine| {
            if mine.lived >= ArmedMine::ARMED_AFTER_DURATION {
                let mut killed = false;
                let mine_rect = Rect::new(
                    mine.body.pos.x - (ArmedMine::TRIGGER_WIDTH / 2.0),
                    mine.body.pos.y - (ArmedMine::TRIGGER_HEIGHT / 2.0),
                    ArmedMine::TRIGGER_WIDTH,
                    ArmedMine::TRIGGER_HEIGHT,
                );
                for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                    let intersect =
                        mine_rect.intersect(Rect::new(
                            player.body.pos.x,
                            player.body.pos.y,
                            20.0,
                            64.0,
                        ));
                    if !intersect.is_none() {
                        let direction = mine.body.pos.x > (player.body.pos.x + 10.);
                        scene::find_node_by_type::<crate::nodes::Camera>()
                            .unwrap()
                            .shake();
                        player.kill(direction);
                        killed = true;
                    }
                }
                if killed {
                    let mut resources = storage::get_mut::<Resources>();
                    resources.hit_fxses.spawn(mine.body.pos);
                    return false;
                }
            }
            return true;
        });
    }

    fn draw(mut node: RefMut<Self>) {
        for mine in &mut node.mines {
            mine.mine_sprite.update();

            if mine.lived >= ArmedMine::ARMED_AFTER_DURATION && mine.mine_sprite.current_animation() != 1 {
                mine.mine_sprite.set_animation(1);
            }

            let resources = storage::get_mut::<Resources>();
            draw_texture_ex(
                resources.mines,
                mine.body.pos.x,
                mine.body.pos.y,
                color::WHITE,
                DrawTextureParams {
                    source: Some(mine.mine_sprite.frame().source_rect),
                    dest_size: Some(mine.mine_sprite.frame().dest_size),
                    flip_x: false,
                    rotation: 0.0,
                    ..Default::default()
                },
            );
        }
    }
}
