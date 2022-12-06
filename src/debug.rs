use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};

use crate::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldInspectorParams {
            enabled: false,
            ..default()
        })
        .add_plugin(WorldInspectorPlugin::new());
    }
}
