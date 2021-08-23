use macroquad::{
    experimental::scene::{self, RefMut},
    prelude::*,
};

use crate::{nodes::Player, noise::NoiseGenerator};
const ZERO: f64 = 0.0;

struct Shake{
    direction: (f32, f32),
    kind: ShakeType,
    magnitude: f32,
    length: f32,
    age: f32,
    random_offset: f32,
    frequency: f32 // 1 is pretty standard, .2 is a punch (with 10 frames of shake it oscillates about max twice). With .5 it's more of a rumble
}

enum ShakeType{
    Noise,
    Sinusodial
}

pub struct Camera {
    bounds: Rect,
    follow_buffer: Vec<(Vec2, f32)>,
    shake: Vec<Shake>,
    noisegen: NoiseGenerator,
    noisegen_position: f32,

    pub manual: Option<(Vec2, f32)>,
}

impl Camera {
    const BUFFER_CAPACITY: usize = 20;

    pub fn new(bounds: Rect) -> Camera {
        /*let mut n = noise_generator::new(5);
        for i in 0..2000{
            let val = n.perlin_2d(0.0, i as f32/25.0);
            let mut v = 0.0;
            while v < val{
                print!(".");
                v += 0.01;
            }            
            println!("");
        }*/

        Camera {
            bounds,
            follow_buffer: vec![],
            shake: vec![],
            manual: None,
            noisegen: NoiseGenerator::new(5),
            noisegen_position: 5.0
        }
    }
}

impl Camera {

    pub fn shake_noise(&mut self, magnitude: f32, length: i32, frequency: f32){
        self.shake.push(Shake{
            direction: (1.0, 1.0),
            kind: ShakeType::Noise,
            magnitude,
            length: length as f32,
            age: 0.0,
            random_offset: rand::gen_range(1.0, 100.0),
            frequency
        });
    }

    pub fn shake_noise_dir(&mut self, magnitude: f32, length: i32, frequency: f32, direction: (f32, f32)){
        self.shake.push(Shake{
            direction,
            kind: ShakeType::Noise,
            magnitude,
            length: length as f32,
            age: 0.0,
            random_offset: rand::gen_range(1.0, 100.0),
            frequency
        });
    }

    pub fn shake_sinusodial(&mut self, magnitude: f32, length: i32, frequency: f32, angle: f32){
        self.shake.push(Shake{
            direction: (angle.cos(), angle.sin()),
            kind: ShakeType::Sinusodial,
            magnitude,
            length: length as f32,
            age: 0.0,
            random_offset: 0.0,
            frequency
        });
    }

    pub fn get_shake(&mut self) -> Vec2{
        self.noisegen_position += 0.5;
        let mut shake_offset = vec2(0.0, 0.0);
        for i in 0..self.shake.len() {
            let strength = 1.0 - self.shake[i].age/self.shake[i].length;
            match self.shake[i].kind{
                ShakeType::Noise =>{
                    shake_offset.x += self.noisegen.perlin_2d(self.noisegen_position * self.shake[i].frequency + self.shake[i].random_offset, 5.0) * self.shake[i].magnitude * self.shake[i].direction.0 * strength * 100.0;
                    shake_offset.y += self.noisegen.perlin_2d(self.noisegen_position * self.shake[i].frequency + self.shake[i].random_offset, 7.0) * self.shake[i].magnitude * self.shake[i].direction.1 * strength * 100.0;
                },
                ShakeType::Sinusodial =>{
                    shake_offset.x += (self.noisegen_position * self.shake[i].frequency * 1.0).sin() * self.shake[i].magnitude * self.shake[i].direction.0 * strength * 50.0; // Noise values are +/- 0.5, trig is twice as large
                    shake_offset.y += (self.noisegen_position * self.shake[i].frequency * 1.0).sin() * self.shake[i].magnitude * self.shake[i].direction.1 * strength * 50.0;
                }
            };

            self.shake[i].age += 1.0;
        }

        self.shake.retain(|s| s.age < s.length);

        shake_offset.x = (shake_offset.x.abs() + 1.0).log2() * shake_offset.x.signum(); // log2(x+1) is almost linear from 0-1, but then flattens out. Limits the screenshake so if there is lots at the same time, the scene won't fly away
        shake_offset.y = (shake_offset.y.abs() + 1.0).log2() * shake_offset.y.signum();

        shake_offset
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
                //let k = if player.controller_id == 1 { 0.8 } else { 0.2 };
                players_amount += 1;
                middle_point += camera_pox_middle; // * k;

                min = min.min(camera_pox_middle);
                max = max.max(camera_pox_middle);
            }
            middle_point /= players_amount as f32;

            let border_x = 150.;
            let border_y = 200.;
            let mut scale = (max - min).abs() + vec2(border_x * 2., border_y * 2.);

            if scale.x > scale.y * aspect {
                scale.y = scale.x / aspect;
            }
            let mut zoom = scale.y;

            // bottom camera bound
            if scale.y / 2. + middle_point.y > node.bounds.h {
                middle_point.y = node.bounds.h - scale.y / 2.;
            }

            if let Some((override_target, override_zoom)) = node.manual {
                middle_point = override_target;
                zoom = override_zoom;
            }

            node.follow_buffer.insert(0, (middle_point, zoom));
            node.follow_buffer.truncate(Self::BUFFER_CAPACITY);
        }
        let mut sum_pos = (ZERO, ZERO);
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

        let shake = node.get_shake();
        middle_point += shake;
        if shake != vec2(0.0, 0.0) && false{
            println!("{}", shake);
        }
        
        let aspect = screen_width() / screen_height();

        let rotation = 0.0;
        // let middle_point = vec2(400., 600.);
        // let zoom = 400.;
        let macroquad_camera = Camera2D {
            target: middle_point,
            zoom: vec2(1. / aspect, -1.) / zoom * 2.,
            rotation,
            ..Camera2D::default()
        };

        scene::set_camera_1(macroquad_camera);
    }
}
