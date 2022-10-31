use bevy::hierarchy::HierarchyEvent;

use crate::{player::PlayerIdx, prelude::*};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Item>()
            .add_event::<ItemGrabEvent>()
            .add_event::<ItemDropEvent>()
            .add_system_to_stage(CoreStage::PreUpdate, trigger_item_events);
    }
}

/// Component indicating that the entity it is attached to is an item that can be picked up
/// by a player.
#[derive(Component, Reflect, Default, Serialize, Deserialize)]
#[reflect(Default, Component)]
pub struct Item {
    /// The path to the item's script
    script: String
}

/// An event triggered when an item is grabbed
pub struct ItemGrabEvent {
    pub player: Entity,
    pub item: Entity,
    pub position: Vec3,
}

/// An event triggered when an item is dropped
pub struct ItemDropEvent {
    pub player: Entity,
    pub item: Entity,
    pub position: Vec3,
}

/// Listen for Bevy hierarchy events that mean an item has been grabbed or dropped and send
/// [`ItemGrabEvent`] and [`ItemDropEvent`] events.
fn trigger_item_events(
    mut grab_events: EventWriter<ItemGrabEvent>,
    mut drop_events: EventWriter<ItemDropEvent>,
    mut hierarchy_events: EventReader<HierarchyEvent>,
    items: Query<Entity, With<Item>>,
    players: Query<&Transform, With<PlayerIdx>>,
) {
    for event in hierarchy_events.iter() {
        match event {
            HierarchyEvent::ChildAdded { child, parent } => {
                if let Ok(player_transform) = players.get(*parent) {
                    if items.contains(*child) {
                        grab_events.send(ItemGrabEvent {
                            player: *parent,
                            item: *child,
                            position: player_transform.translation,
                        })
                    }
                }
            }
            HierarchyEvent::ChildRemoved { child, parent } => {
                if let Ok(player_transform) = players.get(*parent) {
                    if items.contains(*child) {
                        drop_events.send(ItemDropEvent {
                            player: *parent,
                            item: *child,
                            position: player_transform.translation,
                        });
                    }
                }
            }
            HierarchyEvent::ChildMoved { .. } => (),
        }
    }
}
