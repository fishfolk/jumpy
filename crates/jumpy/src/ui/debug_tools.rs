use bevy::prelude::*;
use bevy_egui::*;
use bevy_fluent::Localization;
use bevy_inspector_egui::WorldInspectorParams;
use bevy_rapier2d::{
    plugin::RapierContext,
    prelude::{ColliderDebugColor, DebugRenderContext},
    rapier::{
        math::{Point, Real},
        prelude::{DebugRenderBackend, DebugRenderObject},
    },
};

use crate::localization::LocalizationExt;

/// System that renders the debug tools window which can be toggled by pressing F12
pub fn debug_tools_window(
    mut visible: Local<bool>,
    mut egui_context: ResMut<EguiContext>,
    localization: Res<Localization>,
    input: Res<Input<KeyCode>>,
    mut rapier_debug: ResMut<DebugRenderContext>,
    mut inspector: ResMut<WorldInspectorParams>,
) {
    let ctx = egui_context.ctx_mut();

    // Toggle debug window visibility
    if input.just_pressed(KeyCode::F12) {
        *visible = !*visible;
    }

    // Shortcut to toggle collision shapes without having to use the menu
    if input.just_pressed(KeyCode::F10) {
        rapier_debug.enabled = !rapier_debug.enabled;
    }
    // Shortcut to toggle the inspector without having to use the menu
    if input.just_pressed(KeyCode::F9) {
        inspector.enabled = !inspector.enabled;
    }

    // Display debug tool window
    egui::Window::new(localization.get("debug-tools"))
        // ID is needed because title comes from localizaition which can change
        .id(egui::Id::new("debug_tools"))
        .open(&mut visible)
        .show(ctx, |ui| {
            // Show collision shapes
            ui.checkbox(
                &mut rapier_debug.enabled,
                format!("{} ( F10 )", localization.get("show-collision-shapes")),
            );

            // Show world inspector
            ui.checkbox(
                &mut inspector.enabled,
                format!("{} ( F9 )", localization.get("show-world-inspector")),
            );
        });
}

/// Renders the rapier debug display
pub fn rapier_debug_render(
    rapier_context: Res<RapierContext>,
    mut egui_context: ResMut<EguiContext>,
    mut rapier_debug: ResMut<DebugRenderContext>,
    camera: Query<(&Camera, &GlobalTransform)>,
    custom_colors: Query<&ColliderDebugColor>,
) {
    if !rapier_debug.enabled {
        return;
    }
    let (camera, camera_transform) = camera.single();

    // Create a frameless panel to allow us to render over anywhere on the screen
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(egui_context.ctx_mut(), |ui| {
            let painter = ui.painter();

            let mut backend = RapierEguiRenderBackend {
                egui_size: ui.available_size(),
                camera,
                camera_transform,
                custom_colors,
                context: &rapier_context,
                painter,
            };

            rapier_debug.pipeline.render(
                &mut backend,
                &rapier_context.bodies,
                &rapier_context.colliders,
                &rapier_context.impulse_joints,
                &rapier_context.multibody_joints,
                &rapier_context.narrow_phase,
            );
        });
}

/// Rapier debug rendering backend that uses Egui to draw the lines
struct RapierEguiRenderBackend<'world, 'state, 'a, 'b, 'c> {
    egui_size: egui::Vec2,
    custom_colors: Query<'world, 'state, &'a ColliderDebugColor>,
    context: &'b RapierContext,
    camera: &'c Camera,
    camera_transform: &'c GlobalTransform,
    painter: &'c egui::Painter,
}

impl<'world, 'state, 'a, 'b, 'c> RapierEguiRenderBackend<'world, 'state, 'a, 'b, 'c> {
    /// Helper to grab the objects custom collider color if it exists
    fn object_color(&self, object: DebugRenderObject, default: [f32; 4]) -> egui::Color32 {
        let color = match object {
            DebugRenderObject::Collider(h, ..) => self.context.colliders.get(h).and_then(|co| {
                self.custom_colors
                    .get(Entity::from_bits(co.user_data as u64))
                    .map(|co| co.0)
                    .ok()
            }),
            _ => None,
        };

        let color = color.map(|co| co.as_hsla_f32()).unwrap_or(default);

        egui::Rgba::from_rgba_premultiplied(color[0], color[1], color[2], color[3]).into()
    }
}

impl<'world, 'state, 'a, 'b, 'c> DebugRenderBackend
    for RapierEguiRenderBackend<'world, 'state, 'a, 'b, 'c>
{
    /// Draw a debug line
    fn draw_line(
        &mut self,
        object: DebugRenderObject,
        a: Point<Real>,
        b: Point<Real>,
        color: [f32; 4],
    ) {
        // Convert world coordinates to normalized device coordinates
        let a = self
            .camera
            .world_to_ndc(self.camera_transform, Vec3::new(a[0], a[1], 0.0));
        let b = self
            .camera
            .world_to_ndc(self.camera_transform, Vec3::new(b[0], b[1], 0.0));

        if let (Some(a), Some(b)) = (a, b) {
            // Invert y and convert to egui vec2
            let a = egui::Vec2::new(a.x, -a.y);
            let b = egui::Vec2::new(b.x, -b.y);

            // Map NDC coordinates to egui points
            let half_size = self.egui_size / 2.0;
            let a = a * half_size + half_size;
            let b = b * half_size + half_size;

            // Paint the line
            self.painter.line_segment(
                [a.to_pos2(), b.to_pos2()],
                (1.0, self.object_color(object, color)),
            )
        }
    }
}
