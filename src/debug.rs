use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};

use crate::{config::ENGINE_CONFIG, prelude::*};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Add debug plugins if enabled
        if ENGINE_CONFIG.debug_tools {
            app.insert_resource(WorldInspectorParams {
                enabled: false,
                ..default()
            })
            .add_plugin(WorldInspectorPlugin::new());
        }
    }
}
