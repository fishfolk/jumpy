use crate::prelude::*;

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Item>()
            .add_fixed_update_event::<ItemGrabEvent>()
            .add_fixed_update_event::<ItemDropEvent>()
            .add_fixed_update_event::<ItemUseEvent>();
    }
}

/// Component indicating that the entity it is attached to is an item that can be picked up
/// by a player.
#[derive(Component, Reflect, Default, Serialize, Deserialize, Debug)]
#[reflect(Default, Component)]
pub struct Item {
    /// The path to the item's script
    pub script: String,
}

/// An event triggered when an item is grabbed
#[derive(Reflect, Clone, Debug)]
pub struct ItemGrabEvent {
    pub player: Entity,
    pub item: Entity,
    pub position: Vec3,
}

/// An event triggered when an item is dropped
#[derive(Reflect, Clone, Debug)]
pub struct ItemDropEvent {
    pub player: Entity,
    pub item: Entity,
    pub position: Vec3,
    pub velocity: Vec2,
}

/// An event triggered when an item is used
#[derive(Reflect, Clone, Debug)]
pub struct ItemUseEvent {
    pub player: Entity,
    pub item: Entity,
    pub position: Vec3,
}
