use crate::prelude::*;
use bevy_inspector_egui::{bevy_inspector, inspector_egui_impls};

pub struct JumpyDebugPlugin;

#[derive(Resource, Deref, DerefMut, Default)]
pub struct WorldInspectorEnabled(pub bool);

impl Plugin for JumpyDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldInspectorEnabled>()
            .add_system(world_inspector)
            .add_system_to_stage(CoreStage::Last, || {
                puffin::GlobalProfiler::lock().new_frame();
            });

        let type_registry = app.world.resource::<bevy::app::AppTypeRegistry>();
        let mut type_registry = type_registry.write();

        inspector_egui_impls::register_std_impls(&mut type_registry);
        inspector_egui_impls::register_glam_impls(&mut type_registry);
        inspector_egui_impls::register_bevy_impls(&mut type_registry);
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
