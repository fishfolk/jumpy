use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_rapier2d::prelude::*;

use crate::{config::ENGINE_CONFIG, prelude::*};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default());

        // Add debug plugins if enabled
        if ENGINE_CONFIG.debug_tools {
            app.insert_resource(DebugRenderContext {
                enabled: false,
                ..default()
            })
            .add_plugin(InspectableRapierPlugin);
        }
    }
}
