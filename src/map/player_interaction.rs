use core::Transform;

use hecs::{Entity, World};
use macroquad::prelude::collections::storage;

use crate::{
    items::{RespawnInfo, RespawningItem, RespawningItemKind, Weapon},
    player::{Player, PlayerState},
    utils::timer::Timer,
    Item, PhysicsBody,
};

use super::Map;

pub fn update_map_kill_zone(world: &mut World) {
    let map = storage::get::<Map>();

    // Kill players out of bounds
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

    struct ToDestroy {
        entity: Entity,
        respawn_info: Option<RespawnInfo>,
    }
    let mut to_destroy = Vec::new();

    // Despawn items out of bounds
    for (entity, (item, transform, body)) in world
        .query::<(&mut Item, &Transform, &PhysicsBody)>()
        .iter()
    {
        let item: &mut Item = item;
        let transform: &Transform = transform;
        let body: &PhysicsBody = body;

        let item_rect = body.as_rect(transform.position);

        if !map.get_playable_area().overlaps(&item_rect) {
            to_destroy.push(ToDestroy {
                entity,
                respawn_info: item.respawn_info,
            })
        }
    }

    // Despawn weapons out of bounds
    for (entity, (weapon, transform, body)) in world
        .query::<(&mut Weapon, &Transform, &PhysicsBody)>()
        .iter()
    {
        let weapon: &mut Weapon = weapon;
        let transform: &Transform = transform;
        let body: &PhysicsBody = body;

        let item_rect = body.as_rect(transform.position);

        if !map.get_playable_area().overlaps(&item_rect) {
            to_destroy.push(ToDestroy {
                entity,
                respawn_info: weapon.respawn_info,
            })
        }
    }

    for to_destroy in to_destroy {
        let entity = to_destroy.entity;

        if let Some(respawn_info) = to_destroy.respawn_info {
            if let Ok(weapon) = world.remove_one::<Weapon>(entity) {
                world
                    .insert_one(
                        entity,
                        RespawningItem {
                            timer: Timer::new(respawn_info.respawn_delay),
                            info: respawn_info,
                            kind: RespawningItemKind::Weapon(weapon),
                        },
                    )
                    .unwrap();
            } else if let Ok(item) = world.remove_one::<Item>(entity) {
                world
                    .insert_one(
                        entity,
                        RespawningItem {
                            timer: Timer::new(respawn_info.respawn_delay),
                            info: respawn_info,
                            kind: RespawningItemKind::Item(item),
                        },
                    )
                    .unwrap();
            }
        } else if let Err(err) = world.despawn(entity) {
            #[cfg(debug_assertions)]
            println!("WARNING: {}", err);
        }
    }
}
