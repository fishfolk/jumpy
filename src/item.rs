use crate::{
    networking::{proto::game::GameEventFromServer, server::NetServer, NetId, NetIdMap},
    prelude::*,
};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Item>()
            .add_event::<ItemGrabEvent>()
            .add_event::<ItemDropEvent>()
            .add_system_to_stage(
                CoreStage::PreUpdate,
                send_net_item_spawns.run_if_resource_exists::<NetServer>(),
            );
    }
}

/// Component indicating that the entity it is attached to is an item that can be picked up
/// by a player.
#[derive(Component, Reflect, Default, Serialize, Deserialize)]
#[reflect(Default, Component)]
pub struct Item {
    /// The path to the item's script
    pub script: String,
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

fn send_net_item_spawns(
    server: Res<NetServer>,
    new_items: Query<(Entity, &Transform, &Item), Added<Item>>,
    mut net_ids: ResMut<NetIdMap>,
) {
    for (entity, transform, item) in &new_items {
        let net_id = NetId::new();
        net_ids.insert(entity, net_id);

        server.broadcast_reliable(&GameEventFromServer::SpawnItem {
            net_id,
            script: item.script.clone(),
            pos: transform.translation,
        });
    }
}
