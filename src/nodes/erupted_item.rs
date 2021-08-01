use macroquad::prelude::{collections::storage, get_frame_time, Vec2};
use macroquad_platformer::Tile;

use crate::Resources;

use super::player::PhysicsBody;

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
        if body.collider.is_none() {
            body.pos.y += PhysicsBody::GRAVITY * get_frame_time().powi(2) / 2.
                + body.speed.y * get_frame_time();
            body.pos.x += body.speed.x * get_frame_time();

            body.speed.y += PhysicsBody::GRAVITY * get_frame_time();

            if body.pos.y < enable_at_y || body.speed.y < 0. {
                return false;
            }

            let collision_world = &mut storage::get_mut::<Resources>().collision_world;

            let tile = collision_world.collide_solids(body.pos, 15, 15);

            if tile != Tile::Empty {
                return false;
            }

            body.collider = Some(collision_world.add_actor(body.pos, 15, 15));
        }

        true
    }
}
