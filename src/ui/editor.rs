use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, math::Vec3Swizzles, render::camera::Viewport};
use bevy_egui::*;
use bevy_fluent::Localization;
use iyes_loopless::condition::IntoConditionalExclusiveSystem;

use crate::{localization::LocalizationExt, metadata::GameMeta, prelude::*};

use super::{widget, WidgetSystem};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            editor_system
                .run_in_state(GameState::InGame)
                .run_in_state(InGameState::Editing)
                .at_end(),
        )
        .add_enter_system(InGameState::Editing, setup_editor)
        .add_exit_system(InGameState::Editing, cleanup_editor);
    }
}

fn setup_editor(mut camera: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>) {
    // Reset camera position and scale
    let (mut camera_transform, mut projection) = camera.single_mut();
    *camera_transform = default();
    projection.scale = 1.0;
}

fn cleanup_editor(mut camera: Query<&mut Camera>) {
    // Reset the camera viewport
    camera.single_mut().viewport = default();
}

pub fn editor_system(world: &mut World) {
    world.resource_scope(|world: &mut World, mut egui_ctx: Mut<EguiContext>| {
        let ctx = egui_ctx.ctx_mut();

        egui::TopBottomPanel::top("top-bar").show(ctx, |ui| {
            widget::<EditorTopBar>(world, ui, "editor-top-bar".into(), ());
        });

        egui::SidePanel::left("left-toolbar")
            .width_range(40.0..=40.0)
            .resizable(false)
            .show(ctx, |ui| {
                widget::<EditorLeftToolbar>(world, ui, "editor-left-toolbar".into(), ());
            });

        egui::SidePanel::right("right-toolbar")
            .min_width(125.0)
            .show(ctx, |ui| {
                widget::<EditorRightToolbar>(world, ui, "editor-right-toolbar".into(), ());
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                widget::<EditorCentralPanel>(world, ui, "editor-central-panel".into(), ());
            });
    });
}

#[derive(SystemParam)]
struct EditorTopBar<'w, 's> {
    commands: Commands<'w, 's>,
    camera: Query<'w, 's, (&'static mut Transform, &'static mut OrthographicProjection)>,
    localization: Res<'w, Localization>,
}

impl<'w, 's> WidgetSystem for EditorTopBar<'w, 's> {
    type Args = ();

    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
        _id: super::WidgetId,
        _args: Self::Args,
    ) {
        let mut params: EditorTopBar = state.get_mut(world);
        let (mut transform, mut projection): (Mut<Transform>, Mut<OrthographicProjection>) =
            params.camera.single_mut();
        let zoom = 1.0 / projection.scale * 100.0;
        let [x, y]: [f32; 2] = transform.translation.xy().into();

        ui.horizontal_centered(|ui| {
            ui.label(&params.localization.get("editor"));
            ui.separator();

            if ui
                .small_button(&params.localization.get("view-reset"))
                .clicked()
            {
                projection.scale = 1.0;
                *transform = default();
            }
            ui.label(
                egui::RichText::new(
                    params
                        .localization
                        .get(&format!("view-offset?x={x:.0}&y={y:.0}")),
                )
                .small(),
            );
            ui.label(
                egui::RichText::new(
                    params
                        .localization
                        .get(&format!("view-zoom?percent={zoom:.0}")),
                )
                .small(),
            );

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
    }
}

#[derive(SystemParam)]
struct EditorLeftToolbar<'w, 's> {
    game: Res<'w, GameMeta>,
    #[system_param(ignore)]
    _phantom: PhantomData<(&'w (), &'s ())>,
}

impl<'w, 's> WidgetSystem for EditorLeftToolbar<'w, 's> {
    type Args = ();

    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
        _id: super::WidgetId,
        _args: Self::Args,
    ) {
        let params: EditorLeftToolbar = state.get_mut(world);
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
    }
}

#[derive(SystemParam)]
struct EditorRightToolbar<'w, 's> {
    localization: Res<'w, Localization>,
    #[system_param(ignore)]
    _phantom: PhantomData<(&'w (), &'s ())>,
}

impl<'w, 's> WidgetSystem for EditorRightToolbar<'w, 's> {
    type Args = ();

    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
        _id: super::WidgetId,
        _args: Self::Args,
    ) {
        let params: EditorRightToolbar = state.get_mut(world);
        ui.add_space(ui.spacing().window_margin.top);
        ui.horizontal(|ui| {
            ui.label(&params.localization.get("layers"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button("➕")
                    .on_hover_text(&params.localization.get("create-layer"))
                    .clicked()
                {}
            });
        });
        ui.separator();
    }
}

#[derive(SystemParam)]
struct EditorCentralPanel<'w, 's> {
    game: Res<'w, GameMeta>,
    camera: Query<
        'w,
        's,
        (
            &'static mut Camera,
            &'static mut Transform,
            &'static mut OrthographicProjection,
        ),
    >,
    #[system_param(ignore)]
    _phantom: PhantomData<(&'w (), &'s ())>,
}

impl<'w, 's> WidgetSystem for EditorCentralPanel<'w, 's> {
    type Args = ();

    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
        _id: super::WidgetId,
        _args: Self::Args,
    ) {
        let mut params: EditorCentralPanel = state.get_mut(world);

        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

        let rect = response.rect;

        let (mut camera, mut camera_transform, mut projection): (
            Mut<Camera>,
            Mut<Transform>,
            Mut<OrthographicProjection>,
        ) = params.camera.single_mut();

        // Handle zoom
        if response.hovered() {
            projection.scale -= ui.input().scroll_delta.y * 0.005;
            projection.scale = projection.scale.max(0.05);
        }

        // Handle pan
        if response.dragged_by(egui::PointerButton::Middle) || ui.input().modifiers.command {
            let drag_delta = response.drag_delta() * params.game.ui_theme.scale * projection.scale;
            camera_transform.translation.x -= drag_delta.x;
            camera_transform.translation.y += drag_delta.y;
        }

        // Handle cursor
        //
        // We only change the cursor if it's not been changed by another widget, for instance, for the
        // resize handle of the right sidebar.
        if ui.output().cursor_icon == default() {
            if response.dragged_by(egui::PointerButton::Middle)
                || (ui.input().modifiers.command
                    && response.dragged_by(egui::PointerButton::Primary))
            {
                response.on_hover_cursor(egui::CursorIcon::Grabbing);
            } else if ui.input().modifiers.command {
                response.on_hover_cursor(egui::CursorIcon::Grab);
            } else {
                response.on_hover_cursor(egui::CursorIcon::Crosshair);
            }
        }

        // Update camera viewport
        let ppp = ui.ctx().pixels_per_point();
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(
                (rect.min.x * ppp) as u32,
                (rect.min.y.floor() * ppp) as u32,
            ),
            physical_size: UVec2::new((rect.width() * ppp) as u32, (rect.height() * ppp) as u32),
            ..default()
        });
    }
}
