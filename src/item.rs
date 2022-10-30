use crate::prelude::*;

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Item>();
    }
}

/// Marker component indicating that the entity it is attached to is an item that can be picked up
/// by a player.
#[derive(Component, Reflect, Default, Serialize, Deserialize)]
#[reflect(Default, Component)]
pub struct Item;
