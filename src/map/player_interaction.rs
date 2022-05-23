use core::Transform;

use hecs::World;
use macroquad::prelude::collections::storage;

use crate::{
    player::{Player, PlayerState},
    PhysicsBody,
};

use super::Map;

pub fn update_map_kill_zone(world: &mut World) {
    let map = storage::get::<Map>();

    for (_, (player, transform, body)) in world
        .query::<(&mut Player, &Transform, &PhysicsBody)>()
        .iter()
    {
        let player: &mut Player = player;
        let transform: &Transform = transform;
        let body: &PhysicsBody = body;

        let player_rect = body.as_rect(transform.position);

        if !map.get_playable_area().overlaps(&player_rect) {
            player.state = PlayerState::Dead;
        }
    }
}
