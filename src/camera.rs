use ff_core::noise::NoiseGenerator;
use ff_core::prelude::*;

use crate::player::Player;

struct Shake {
    direction: (f32, f32),
    kind: ShakeType,
    magnitude: f32,
    length: f32, //in frames, but stored in float to avoid casting
    age: f32,
    random_offset: f32,
    frequency: f32, // 1 is pretty standard, .2 is a punch (with 10 frames of shake it oscillates about max twice). With .5 it's more of a rumble
}

#[allow(dead_code)]
enum ShakeType {
    Noise,
    Sinusoidal,
    Rotational,
}

const CAMERA_FOLLOW_BUFFER_CAPACITY: usize = 20;

pub struct CameraController {
    follow_buffer: Vec<(Vec2, f32)>,
    shake: Vec<Shake>,
    noisegen: NoiseGenerator,
    noisegen_position: f32,
    position_override: Option<Vec2>,
    zoom_override: Option<f32>,
}

impl CameraController {
    pub fn new() -> Self {
        CameraController {
            follow_buffer: Vec::new(),
            shake: Vec::new(),
            position_override: None,
            zoom_override: None,
            noisegen: NoiseGenerator::new(5),
            noisegen_position: 5.0,
        }
    }

    pub fn set_overrides<P, Z>(&mut self, position: P, zoom: Z)
    where
        P: Into<Option<Vec2>>,
        Z: Into<Option<f32>>,
    {
        self.position_override = position.into();
        self.zoom_override = zoom.into();
    }

    pub fn shake_noise(&mut self, magnitude: f32, length: i32, frequency: f32) {
        self.shake.push(Shake {
            direction: (1.0, 1.0),
            kind: ShakeType::Noise,
            magnitude,
            length: length as f32,
            age: 0.0,
            random_offset: rand::gen_range(1.0, 100.0),
            frequency,
        });
    }

    pub fn shake_noise_dir(
        &mut self,
        magnitude: f32,
        length: i32,
        frequency: f32,
        direction: (f32, f32),
    ) {
        self.shake.push(Shake {
            direction,
            kind: ShakeType::Noise,
            magnitude,
            length: length as f32,
            age: 0.0,
            random_offset: rand::gen_range(1.0, 100.0),
            frequency,
        });
    }

    pub fn shake_sinusoidal(&mut self, magnitude: f32, length: i32, frequency: f32, angle: f32) {
        self.shake.push(Shake {
            direction: (angle.cos(), angle.sin()),
            kind: ShakeType::Sinusoidal,
            magnitude,
            length: length as f32,
            age: 0.0,
            random_offset: 0.0,
            frequency,
        });
    }

    pub fn shake_rotational(&mut self, magnitude: f32, length: i32) {
        self.shake.push(Shake {
            direction: (1.0, 1.0),
            kind: ShakeType::Rotational,
            magnitude: magnitude * (rand::gen_range(0, 2) as f32 - 0.5) * 2.0,
            length: length as f32,
            age: 0.0,
            random_offset: 0.0,
            frequency: 0.0,
        });
    }

    pub fn get_shake(&mut self) -> (Vec2, f32) {
        self.noisegen_position += 0.5;
        let mut shake_offset = vec2(0.0, 0.0);
        let mut shake_rotation = 0.0;
        for i in 0..self.shake.len() {
            let strength = 1.0 - self.shake[i].age / self.shake[i].length;
            match self.shake[i].kind {
                ShakeType::Noise => {
                    shake_offset.x += self.noisegen.perlin_2d(
                        self.noisegen_position * self.shake[i].frequency
                            + self.shake[i].random_offset,
                        5.0,
                    ) * self.shake[i].magnitude
                        * self.shake[i].direction.0
                        * strength
                        * 100.0;
                    shake_offset.y += self.noisegen.perlin_2d(
                        self.noisegen_position * self.shake[i].frequency
                            + self.shake[i].random_offset,
                        7.0,
                    ) * self.shake[i].magnitude
                        * self.shake[i].direction.1
                        * strength
                        * 100.0;
                }
                ShakeType::Sinusoidal => {
                    shake_offset.x += (self.noisegen_position * self.shake[i].frequency * 1.0)
                        .sin()
                        * self.shake[i].magnitude
                        * self.shake[i].direction.0
                        * strength
                        * 50.0; // Noise values are +/- 0.5, trig is twice as large
                    shake_offset.y += (self.noisegen_position * self.shake[i].frequency * 1.0)
                        .sin()
                        * self.shake[i].magnitude
                        * self.shake[i].direction.1
                        * strength
                        * 50.0;
                }
                ShakeType::Rotational => {
                    //shake_rotation += self.noisegen.perlin_2d(self.noisegen_position * self.shake[i].frequency + self.shake[i].random_offset, 5.0) * self.shake[i].magnitude * strength.powi(3);
                    shake_rotation += self.shake[i].magnitude * strength.powi(3) * 3.0;
                }
            };

            self.shake[i].age += 1.0;
        }

        self.shake.retain(|s| s.age < s.length);

        shake_offset.x = (shake_offset.x.abs() + 1.0).log2() * shake_offset.x.signum(); // log2(x+1) is almost linear from 0-1, but then flattens out. Limits the screenshake so if there is lots at the same time, the scene won't fly away
        shake_offset.y = (shake_offset.y.abs() + 1.0).log2() * shake_offset.y.signum();

        (shake_offset, shake_rotation)
    }
}

pub fn update_camera(world: &mut World, _delta_time: f32) -> Result<()> {
    let mut player_rects = Vec::new();

    for (_, (transform, player)) in world.query_mut::<(&Transform, &mut Player)>() {
        let rect = Rect::new(transform.position.x, transform.position.y, 32.0, 60.0);

        if rect.x < player.camera_box.x {
            player.camera_box.x = rect.x;
        }

        if rect.x + rect.width > player.camera_box.x + player.camera_box.width {
            player.camera_box.x = rect.x + rect.width - player.camera_box.width;
        }

        if rect.y < player.camera_box.y {
            player.camera_box.y = rect.y;
        }

        if rect.y + rect.height > player.camera_box.y + player.camera_box.height {
            player.camera_box.y = rect.y + rect.height - player.camera_box.height;
        }

        player_rects.push(rect);
    }

    let (_, (transform, camera_ctrl)) = world
        .query_mut::<(&mut Transform, &mut CameraController)>()
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("ERROR: No camera controller found!"));

    let mut camera = main_camera();
    let aspect_ratio = camera.aspect_ratio();

    {
        let mut middle_point = vec2(0.0, 0.0);
        let mut min = vec2(10000.0, 10000.0);
        let mut max = vec2(-10000.0, -10000.0);

        let player_cnt = player_rects.len();
        for rect in player_rects {
            let camera_pox_middle = rect.point() + rect.size() / 2.0;
            middle_point += camera_pox_middle;

            min = min.min(camera_pox_middle);
            max = max.max(camera_pox_middle);
        }

        middle_point /= player_cnt as f32;

        let border_x = 150.0;
        let border_y = 200.0;
        let mut scale = (max - min).abs() + vec2(border_x * 2.0, border_y * 2.0);

        if scale.x > scale.y * aspect_ratio {
            scale.y = scale.x / aspect_ratio;
        }

        let mut zoom = scale.y;

        let bounds = camera.bounds;

        // bottom camera bound
        if scale.y / 2.0 + middle_point.y > bounds.height {
            middle_point.y = bounds.height - scale.y / 2.0;
        }

        if let Some(override_position) = camera_ctrl.position_override {
            middle_point = override_position;
        }

        if let Some(zoom_override) = camera_ctrl.zoom_override {
            zoom = zoom_override;
        }

        camera_ctrl.follow_buffer.insert(0, (middle_point, zoom));
        camera_ctrl
            .follow_buffer
            .truncate(CAMERA_FOLLOW_BUFFER_CAPACITY);
    }

    let mut sum_pos = (0.0f64, 0.0f64);
    let mut sum_zoom = 0.0;
    for (pos, zoom) in &camera_ctrl.follow_buffer {
        sum_pos.0 += pos.x as f64;
        sum_pos.1 += pos.y as f64;
        sum_zoom += *zoom as f64;
    }

    transform.position = vec2(
        (sum_pos.0 / camera_ctrl.follow_buffer.len() as f64) as f32,
        (sum_pos.1 / camera_ctrl.follow_buffer.len() as f64) as f32,
    );

    let shake = camera_ctrl.get_shake();
    transform.position += shake.0;
    transform.rotation = shake.1;

    let zoom = (sum_zoom / camera_ctrl.follow_buffer.len() as f64) as f32;

    camera.zoom = vec2(1.0 / aspect_ratio, -1.0) / zoom * 2.0;
    camera.target = transform.position;
    camera.rotation = transform.rotation;

    #[cfg(feature = "macroquad")]
    {
        use ff_core::macroquad::camera::Camera2D;
        use ff_core::macroquad::experimental::scene;

        let macroquad_camera = Camera2D {
            target: transform.position,
            zoom: camera.zoom,
            rotation: camera.rotation,
            ..Camera2D::default()
        };

        scene::set_camera(0, Some(macroquad_camera));
    }

    Ok(())
}
