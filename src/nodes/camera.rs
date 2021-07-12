use macroquad::{
    experimental::scene::{self, RefMut},
    prelude::*,
};

use crate::nodes::Player;

pub struct Camera {
    bounds: Rect,
    follow_buffer: Vec<(Vec2, f32)>,
    macroquad_camera: Camera2D,
    shake: Option<(f32, f32)>,
}

impl Camera {
    const BUFFER_CAPACITY: usize = 20;

    pub fn new(bounds: Rect) -> Camera {
        Camera {
            bounds,
            follow_buffer: vec![],
            macroquad_camera: Camera2D::default(),
            shake: None,
        }
    }
}

impl Camera {
    pub fn macroquad_camera(&self) -> &Camera2D {
        &self.macroquad_camera
    }

    pub fn shake(&mut self) {
        self.shake = Some((0.0, 0.1));
    }
}

impl scene::Node for Camera {
    fn fixed_update(mut node: RefMut<Self>) {
        {
            let players = scene::find_nodes_by_type::<Player>();
            let aspect = screen_width() / screen_height();

            let mut players_amount = 0;
            let mut middle_point = vec2(0., 0.);
            let mut min = vec2(10000., 10000.);
            let mut max = vec2(-10000., -10000.);

            for player in players {
                let camera_pox_middle = player.camera_box.point() + player.camera_box.size() / 2.;
                players_amount += 1;
                middle_point += camera_pox_middle;

                min = min.min(camera_pox_middle);
                max = max.max(camera_pox_middle);
            }
            middle_point /= players_amount as f32;

            let border = 150.;
            let mut scale = (max - min).abs() + vec2(border * 2., border * 2.);

            if scale.x > scale.y * aspect {
                scale.y = scale.x / aspect;
            }
            let zoom = scale.y;

            // bottom camera bound
            if scale.y / 2. + middle_point.y > node.bounds.h {
                middle_point.y = node.bounds.h - scale.y / 2.;
            }

            node.follow_buffer.insert(0, (middle_point, zoom));
            node.follow_buffer.truncate(Self::BUFFER_CAPACITY);
        }
        let mut sum_pos = (0.0f64, 0.0f64);
        let mut sum_zoom = 0.0;
        for (pos, zoom) in &node.follow_buffer {
            sum_pos.0 += pos.x as f64;
            sum_pos.1 += pos.y as f64;
            sum_zoom += *zoom as f64;
        }
        let mut middle_point = vec2(
            (sum_pos.0 / node.follow_buffer.len() as f64) as f32,
            (sum_pos.1 / node.follow_buffer.len() as f64) as f32,
        );
        let zoom = (sum_zoom / node.follow_buffer.len() as f64) as f32;

        let mut rotation = 0.0;

        if let Some(ref mut shake) = node.shake {
            let t = shake.0 / shake.1;

            let t1 = 1.0 - t;
            //let shift_x = 0.;
            let shift_y = t1 * t1;
            middle_point += vec2(0., shift_y * 20.);
            rotation = (t * t * std::f32::consts::PI).cos() * 2.;
            shake.0 += get_frame_time();
            if shake.0 >= shake.1 {
                node.shake = None;
            }
        }

        let aspect = screen_width() / screen_height();

        // let middle_point = vec2(400., 600.);
        // let zoom = 400.;
        node.macroquad_camera = Camera2D {
            target: middle_point,
            zoom: vec2(1. / aspect, -1.) / zoom * 2.,
            rotation,
            ..Camera2D::default()
        };

        scene::set_camera_1(*node.macroquad_camera());
    }
}
