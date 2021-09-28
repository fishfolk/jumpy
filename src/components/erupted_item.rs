use macroquad::math::{vec2, Vec2};
use macroquad::prelude::{collections::storage, get_frame_time};
use macroquad_platformer::Tile;

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

        body.pos.y += PhysicsBody::GRAVITY * get_frame_time().powi(2) / 2.
            + body.speed.y * get_frame_time();
        body.pos.x += body.speed.x * get_frame_time();
        body.speed.y += PhysicsBody::GRAVITY * get_frame_time();

        collision_world.move_h(body.collider, body.speed.x * get_frame_time());
        
        collision_world.move_v(body.collider, body.speed.y * get_frame_time());


        if body.pos.y < enable_at_y || body.speed.y < 0. {
            return false;
        }

        let tile = collision_world.collide_solids(body.pos, 15, 15);

        if tile != Tile::Empty {
            return false;
        }

        true
    }
}
