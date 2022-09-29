use bevy::{ecs::system::SystemParam, render::camera::Viewport, sprite::MaterialMesh2dBundle};
use bevy_egui::*;
use bevy_fluent::Localization;

use crate::{localization::LocalizationExt, metadata::GameMeta, prelude::*};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            editor
                .run_in_state(GameState::InGame)
                .run_in_state(InGameState::Editing),
        )
        .add_enter_system(InGameState::Editing, setup_editor)
        .add_exit_system(InGameState::Editing, cleanup_editor);
    }
}

#[derive(SystemParam)]
pub struct EditorParams<'w, 's> {
    camera: Query<
        'w,
        's,
        (
            &'static mut Camera,
            &'static mut Transform,
            &'static mut OrthographicProjection,
        ),
    >,
    commands: Commands<'w, 's>,
    game: Res<'w, GameMeta>,
    localization: Res<'w, Localization>,
}

fn setup_editor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::default().with_scale(Vec3::splat(128.)),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        ..default()
    });
}

fn cleanup_editor(mut camera: Query<&mut Camera>) {
    // Reset the camera viewport
    camera.single_mut().viewport = default();
}

/// The map editor system
pub fn editor(mut params: EditorParams, mut egui_ctx: ResMut<EguiContext>) {
    let ctx = egui_ctx.ctx_mut();
    egui::TopBottomPanel::top("top-bar").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            ui.label(&params.localization.get("editor"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(&params.localization.get("main-menu")).clicked() {
                    params
                        .commands
                        .insert_resource(NextState(GameState::MainMenu));
                }
                if ui.button(&params.localization.get("play")).clicked() {
                    params
                        .commands
                        .insert_resource(NextState(InGameState::Playing));
                }
            });
        });
    });

    egui::SidePanel::left("left-toolbar")
        .width_range(40.0..=40.0)
        .resizable(false)
        .show(ctx, |ui| {
            let icons = &params.game.ui_theme.editor.icons;
            let width = ui.available_width();
            for image in &[&icons.select, &icons.tile, &icons.spawn, &icons.erase] {
                ui.add_space(ui.spacing().window_margin.top);

                let image_aspect = image.image_size.y / image.image_size.x;
                let height = width * image_aspect;
                ui.add(egui::ImageButton::new(
                    image.egui_texture_id,
                    egui::vec2(width, height),
                ));
            }
        });

    egui::SidePanel::right("right-toolbar")
        .min_width(190.0)
        .show(ctx, |ui| {
            ui.add_space(ui.spacing().window_margin.top);
            ui.horizontal(|ui| {
                ui.label("Layers");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("➕")
                        .on_hover_text("Create a new metatile")
                        .clicked()
                    {}
                });
            });
            ui.separator();
        });

    let response = egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag())
        })
        .inner;
    let rect = response.rect;

    let (mut camera, mut camera_transform, mut projection): (
        Mut<Camera>,
        Mut<Transform>,
        Mut<OrthographicProjection>,
    ) = params.camera.single_mut();

    // Handle zoom
    if response.hovered() {
        projection.scale -= ctx.input().scroll_delta.y * 0.005;
        projection.scale = projection.scale.max(0.05);
    }

    // Handle pan
    if response.dragged_by(egui::PointerButton::Middle) || ctx.input().modifiers.command {
        let drag_delta = response.drag_delta() * params.game.ui_theme.scale * projection.scale;
        camera_transform.translation.x -= drag_delta.x;
        camera_transform.translation.y += drag_delta.y;
    }

    // Handle cursor
    if response.dragged_by(egui::PointerButton::Middle)
        || (ctx.input().modifiers.command && response.dragged_by(egui::PointerButton::Primary))
    {
        response.on_hover_cursor(egui::CursorIcon::Grabbing);
    } else if ctx.input().modifiers.command {
        response.on_hover_cursor(egui::CursorIcon::Grab);
    } else {
        response.on_hover_cursor(egui::CursorIcon::Crosshair);
    }

    // Update camera viewport
    let ppp = ctx.pixels_per_point();
    camera.viewport = Some(Viewport {
        physical_position: UVec2::new((rect.min.x * ppp) as u32, (rect.min.y.floor() * ppp) as u32),
        physical_size: UVec2::new((rect.width() * ppp) as u32, (rect.height() * ppp) as u32),
        ..default()
    });
}
