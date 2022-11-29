use crate::{
    animation::AnimatedSprite,
    damage::{DamageRegion, DamageRegionOwner},
    item::{Item, ItemDropped, ItemUsed},
    lifetime::Lifetime,
    map::MapElementHydrated,
    metadata::{BuiltinElementKind, MapElementMeta},
    name::EntityName,
    physics::{collisions::CollisionWorld, KinematicBody},
    player::{input::PlayerInputs, PlayerIdx, MAX_PLAYERS},
    prelude::*,
    utils::Sort,
};

// Meta/environment elements
pub mod decoration;
pub mod player_spawner;
pub mod sproinger;

// Items
pub mod grenade;
pub mod sword;

pub struct MapElementsPlugin;

impl Plugin for MapElementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(decoration::DecorationPlugin)
            .add_plugin(grenade::GrenadePlugin)
            .add_plugin(player_spawner::PlayerSpawnerPlugin)
            .add_plugin(sproinger::SproingerPlugin)
            .add_plugin(sword::SwordPlugin);
    }
}
