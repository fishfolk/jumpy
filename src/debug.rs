//! Debugging tools.
//!
//! Sets up the Bevy world inspector, and the puffin profiler.
//!
//! More debug related ui code can be found in [`ui::debug_tools`].

use crate::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContext;
use bevy_inspector_egui::{bevy_inspector, inspector_egui_impls};

/// Debug plugin.
pub struct JumpyDebugPlugin;

/// Resource that tracks whether or not the world inspector window is visible.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct WorldInspectorEnabled(pub bool);

impl Plugin for JumpyDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldInspectorEnabled>()
            .add_system(world_inspector)
            .add_system(
                (|| {
                    profiling::mark_new_frame();
                })
                .in_base_set(CoreSet::Last),
            );

        let type_registry = app.world.resource::<bevy::app::AppTypeRegistry>();
        let mut type_registry = type_registry.write();

        inspector_egui_impls::register_std_impls(&mut type_registry);
        inspector_egui_impls::register_glam_impls(&mut type_registry);
        inspector_egui_impls::register_bevy_impls(&mut type_registry);
    }
}

/// Renders the world-inspector UI.
pub fn world_inspector(world: &mut World) {
    if !**world.resource::<WorldInspectorEnabled>() {
        return;
    }

    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    egui::Window::new("World Inspector")
        .default_size(egui::vec2(400.0, 400.0))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector::ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}
