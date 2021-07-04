use macroquad::{
    experimental::scene::{self, Handle, RefMut},
    prelude::*,
};

use crate::nodes::Player;

pub struct Camera {
    follow_buffer: Vec<Vec2>,
    player: Handle<Player>,
    macroquad_camera: Camera2D,
}

impl Camera {
    const BUFFER_CAPACITY: usize = 20;

    pub fn new(player: Handle<Player>) -> Camera {
        Camera {
            player,
            follow_buffer: vec![],
            macroquad_camera: Camera2D::default(),
        }
    }
}

impl Camera {
    pub fn macroquad_camera(&self) -> &Camera2D {
        &self.macroquad_camera
    }
}

impl scene::Node for Camera {
    fn update(mut node: RefMut<Self>) {
        let player = scene::try_get_node::<Player>(node.player);
        let foe_pos = scene::find_nodes_by_type::<Player>()
            .map(|player| player.body.pos)
            .next()
            .unwrap();

        if let Some(player) = player {
            node.follow_buffer.insert(0, foe_pos);
            node.follow_buffer.truncate(Self::BUFFER_CAPACITY);

            let mut sum = (0.0f64, 0.0f64);
            for pos in &node.follow_buffer {
                sum.0 += pos.x as f64;
                sum.1 += pos.y as f64;
            }
            let foe_pos = vec2(
                (sum.0 / node.follow_buffer.len() as f64) as f32,
                (sum.1 / node.follow_buffer.len() as f64) as f32,
            );

            let aspect = screen_width() / screen_height() / 2.;
            let middle_point = (player.body.pos + foe_pos) / 2.;

            let border = 100.;
            let mut scale = (player.body.pos - foe_pos).abs() + vec2(border * 2., border * 2.);

            // if we'd use scaled X dimension as new Y dimension
            // will it fit the original Y dimension?
            if scale.y < scale.x / aspect {
                scale.y = scale.x / aspect;
            }
            // if not - lets stretch another axis
            else {
                scale.x = scale.y * aspect;
            }

            node.macroquad_camera = Camera2D {
                viewport: if player.controller_id == 0 {
                    Some((0, 0, screen_width() as i32 / 2, screen_height() as i32))
                } else {
                    Some((
                        screen_width() as i32 / 2,
                        0,
                        screen_width() as i32 / 2,
                        screen_height() as i32,
                    ))
                },
                target: middle_point,
                zoom: vec2(1., -1.) / scale * 2.,
                ..Camera2D::default()
            };

            if player.controller_id == 0 {
                scene::set_camera_1(*node.macroquad_camera());
            } else {
                scene::set_camera_2(*node.macroquad_camera());
            }
        }
    }
}
