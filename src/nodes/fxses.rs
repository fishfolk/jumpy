use macroquad::{
    camera::*,
    experimental::{
        collections::storage,
        scene::{self, RefMut},
    },
    math::Vec2,
    telemetry,
};

use crate::Resources;

pub struct Fxses {}

impl scene::Node for Fxses {
    fn draw(_node: RefMut<Self>) {
        let mut resources = storage::get_mut::<Resources>();

        let _z = telemetry::ZoneGuard::new("draw particles");

        resources.smoke_emitter.draw(Vec2::new(0., 0.));
        resources.hit_emitters.draw();
        resources.explosion_emitters.draw();
        resources.explosion_fire_emitter.draw(Vec2::new(0., 0.));
        resources.explosion_particle_emitters.draw();

        for fx in resources.item_emitters.values_mut() {
            fx.draw();
        }

        push_camera_state();
        set_default_camera();
        resources.life_ui_explosion_emitters.draw();
        pop_camera_state();
        // macroquad_profiler::profiler(macroquad_profiler::ProfilerParams {
        //     fps_counter_pos: macroquad::math::vec2(50.0, 20.0),
        // });
    }
}
