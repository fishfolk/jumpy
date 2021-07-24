use macroquad::{
    camera::*,
    experimental::{
        collections::storage,
        scene::{self, RefMut},
    },
    telemetry,
};

use crate::Resources;

pub struct Fxses {}

impl scene::Node for Fxses {
    fn draw(_node: RefMut<Self>) {
        let mut resources = storage::get_mut::<Resources>();

        let _z = telemetry::ZoneGuard::new("draw particles");

        resources.hit_fxses.draw();
        resources.cannonball_hit_fxses.draw();
        resources.explosion_fxses.draw();

        push_camera_state();
        set_default_camera();
        resources.life_explosion_fxses.draw();
        pop_camera_state();
        // macroquad_profiler::profiler(macroquad_profiler::ProfilerParams {
        //     fps_counter_pos: macroquad::math::vec2(50.0, 20.0),
        // });
    }
}
