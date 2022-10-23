use crate::networking::Connection;
use bevy::app::AppExit;

use crate::prelude::*;

pub struct ServerPlugin;

#[derive(Deref, DerefMut)]
pub struct NetClients(pub Vec<Connection>);

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::First,
            exit_on_disconnect.run_if_resource_exists::<NetClients>(),
        );
    }
}

fn exit_on_disconnect(mut clients: ResMut<NetClients>, mut exit_sender: EventWriter<AppExit>) {
    // Remove disconnected clients
    clients.retain(|conn| conn.close_reason().is_none());

    // If all clients have disconnected, exit the app
    if clients.is_empty() {
        info!("All clients disconnected from match");
        exit_sender.send_default();
    }
}
