use macroquad::{
    experimental::scene::{self, Handle, RefMut},
    prelude::*,
};

use crate::nodes::Player;

pub struct Camera {
    bounds: Rect,
    viewport_height: f32,
    follow_buffer: Vec<Vec2>,
    player: Handle<Player>,
    macroquad_camera: Camera2D,
}

impl Camera {
    const BUFFER_CAPACITY: usize = 20;

    pub fn new(bounds: Rect, viewport_height: f32, player: Handle<Player>) -> Camera {
        Camera {
            player,
            bounds,
            follow_buffer: vec![],
            viewport_height,
            macroquad_camera: Camera2D::default(),
        }
    }
}

impl Camera {
    pub fn pos(&self) -> Vec2 {
        self.macroquad_camera.target
    }

    pub fn macroquad_camera(&self) -> &Camera2D {
        &self.macroquad_camera
    }
}

impl scene::Node for Camera {
    fn update(mut node: RefMut<Self>) {
        if let Some(player) = scene::try_get_node::<Player>(node.player) {
            node.follow_buffer.insert(0, player.pos());
            node.follow_buffer.truncate(Self::BUFFER_CAPACITY);

            let mut sum = (0.0f64, 0.0f64);
            for pos in &node.follow_buffer {
                sum.0 += pos.x as f64;
                sum.1 += pos.y as f64;
            }
            let mut pos = vec2(
                (sum.0 / node.follow_buffer.len() as f64) as f32,
                (sum.1 / node.follow_buffer.len() as f64) as f32,
            );
            let aspect = screen_width() / screen_height();

            let viewport_width = node.viewport_height * aspect;

            if pos.x < viewport_width / 2. {
                pos.x = viewport_width / 2.;
            }

            if pos.x > node.bounds.w as f32 - viewport_width / 2. {
                pos.x = node.bounds.w as f32 - viewport_width / 2.;
            }
            if pos.y < node.viewport_height / 2. {
                pos.y = node.viewport_height / 2.;
            }

            if pos.y > node.bounds.h as f32 - node.viewport_height / 2. {
                pos.y = node.bounds.h as f32 - node.viewport_height / 2.;
            }
            node.macroquad_camera = Camera2D {
                zoom: vec2(
                    1.0 / viewport_width as f32 * 2.,
                    -1.0 / node.viewport_height as f32 * 2.,
                ),
                target: vec2(pos.x, pos.y),
                ..Default::default()
            }
        }
        scene::set_camera(*node.macroquad_camera());
    }
}
