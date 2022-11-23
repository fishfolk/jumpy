use crate::{
    animation::AnimatedSprite,
    map::MapElementHydrated,
    metadata::{BuiltinElementKind, MapElementMeta},
    physics::{collisions::CollisionWorld, KinematicBody},
    player::{input::PlayerInputs, PlayerIdx, MAX_PLAYERS},
    prelude::*,
};

pub mod decoration;
pub mod player_spawner;
pub mod sproinger;
pub mod sword;

pub struct MapElementsPlugin;

impl Plugin for MapElementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(decoration::DecorationPlugin)
            .add_plugin(player_spawner::PlayerSpawnerPlugin)
            .add_plugin(sproinger::SproingerPlugin)
            .add_plugin(sword::SwordPlugin);
    }
}
