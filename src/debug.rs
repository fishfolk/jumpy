use crate::prelude::*;
use bevy_inspector_egui::{bevy_inspector, DefaultInspectorConfigPlugin};

pub struct JumpyDebugPlugin;

#[derive(Resource, Deref, DerefMut, Default)]
pub struct WorldInspectorEnabled(pub bool);

impl Plugin for JumpyDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldInspectorEnabled>()
            .add_plugin(DefaultInspectorConfigPlugin)
            .add_system(world_inspector);
    }
}

pub fn world_inspector(world: &mut World) {
    if !**world.resource::<WorldInspectorEnabled>() {
        return;
    }

    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();
    egui::Window::new("World Inspector")
        .default_size(egui::vec2(400.0, 400.0))
        .show(&egui_context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector::ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}
