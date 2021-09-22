use macroquad::prelude::{collections::storage, get_frame_time, Vec2};
use macroquad_platformer::Tile;

use crate::{Resources, components::PhysicsBody};


pub trait EruptedItem {
    fn spawn_for_volcano(pos: Vec2, speed: Vec2, enable_at_y: f32, owner_id: u8);

    fn body(&mut self) -> &mut PhysicsBody;
    fn enable_at_y(&self) -> f32;

    // Assumes that the eruption is running; doesn't check it.
    fn eruption_update(&mut self) -> bool {
        //TODO: make it work for when it erupts from volcanoes

        true
    }
}