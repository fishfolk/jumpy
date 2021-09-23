use macroquad::prelude::{collections::storage, get_frame_time};
use macroquad::math::{vec2, Vec2};

use crate::Resources;

use crate::components::PhysicsBody;

//at the moment is not possible to test the code as ite requires EruptingVolcano to test
//It also is dependency of ArmedGrenade so a rough estimation of the code to not implement the trait later
//is here.
pub trait EruptedItem {
    fn spawn_for_volcano(pos: Vec2, speed: Vec2, enable_at_y: f32, owner_id: u8);

    fn body(&mut self) -> &mut PhysicsBody;
    fn enable_at_y(&self) -> f32;

    // Assumes that the eruption is running; doesn't check it.
    fn eruption_update(&mut self) -> bool {
        let enable_at_y = self.enable_at_y();
        let body = self.body();

        // Take control while erupting, and the collider hasn't been enabled. Afer that point, behave
        // as usual.
        let collision_world = &mut storage::get_mut::<Resources>().collision_world;

        body.pos = collision_world.actor_pos(body.collider);
        body.last_frame_on_ground = body.on_ground;
        body.on_ground = collision_world.collide_check(body.collider, body.pos + vec2(0., 1.));
        if !body.on_ground && body.have_gravity {
            body.pos.y += PhysicsBody::GRAVITY * get_frame_time().powi(2) / 2.
                + body.speed.y * get_frame_time();
            body.pos.x += body.speed.x * get_frame_time();
            body.speed.y += PhysicsBody::GRAVITY * get_frame_time();
        }
        if !collision_world.move_h(body.collider, body.speed.x * get_frame_time()) {
            body.speed.x *= -body.bouncyness;
        }
        if !collision_world.move_v(body.collider, body.speed.y * get_frame_time()) {
            body.speed.y *= -body.bouncyness;
        }
        body.pos = collision_world.actor_pos(body.collider);

        if body.pos.y < enable_at_y || body.speed.y < 0. {
            return false;
        }

        true
    }
}