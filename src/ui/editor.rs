use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, math::Vec3Swizzles, render::camera::Viewport};
use bevy_egui::*;
use bevy_fluent::Localization;
use bevy_prototype_lyon::prelude::*;

use crate::{
    localization::LocalizationExt,
    metadata::{GameMeta, MapMeta},
    prelude::*,
};

use super::{widget, widgets::bordered_button::BorderedButton, WidgetSystem};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorViewSettings>()
            .add_system(
                editor_update
                    .run_in_state(GameState::InGame)
                    .run_in_state(InGameState::Editing),
            )
            .add_system(
                iyes_loopless::condition::IntoConditionalExclusiveSystem::run_in_state(
                    editor_ui_system,
                    GameState::InGame,
                )
                .run_in_state(InGameState::Editing)
                .at_end(),
            )
            .add_enter_system(InGameState::Editing, setup_editor)
            .add_exit_system(InGameState::Editing, cleanup_editor);
    }
}
/// Marker component for the map grid
#[derive(Component)]
struct MapGridView;

struct EditorViewSettings {
    pub show_grid: bool,
}

impl Default for EditorViewSettings {
    fn default() -> Self {
        Self { show_grid: true }
    }
}

fn setup_editor(mut camera: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>) {
    // Reset camera position and scale
    let (mut camera_transform, mut projection) = camera.single_mut();
    *camera_transform = default();
    projection.scale = 1.0;
}

fn editor_update(
    mut map_grid: Query<&mut Visibility, With<MapGridView>>,
    settings: Res<EditorViewSettings>,
) {
    for mut visibility in &mut map_grid {
        visibility.is_visible = settings.show_grid;
    }
}

fn cleanup_editor(
    mut camera: Query<&mut Camera>,
    mut map_grid: Query<&mut Visibility, With<MapGridView>>,
) {
    // Reset the camera viewport
    camera.single_mut().viewport = default();

    // Hide the map grid
    for mut visibility in &mut map_grid {
        visibility.is_visible = false;
    }
}

pub fn editor_ui_system(world: &mut World) {
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
    map: Query<'w, 's, Entity, With<MapMeta>>,
    settings: ResMut<'w, EditorViewSettings>,
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

            ui.add_space(ui.spacing().icon_spacing);

            ui.checkbox(
                &mut params.settings.show_grid,
                &params.localization.get("show-grid"),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(&params.localization.get("main-menu")).clicked() {
                    params
                        .commands
                        .insert_resource(NextState(GameState::MainMenu));
                }

                ui.scope(|ui| {
                    ui.set_enabled(params.map.get_single().is_ok());
                    if ui.button(&params.localization.get("play")).clicked() {
                        params
                            .commands
                            .insert_resource(NextState(InGameState::Playing));
                    }
                });
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
    map: Query<'w, 's, &'static MapMeta>,
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
        let has_map = params.map.get_single().is_ok();
        ui.set_enabled(has_map);

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
    commands: Commands<'w, 's>,
    show_map_create: Local<'s, bool>,
    map_create_info: Local<'s, MapCreateInfo>,
    game: Res<'w, GameMeta>,
    map: Query<'w, 's, Entity, With<MapMeta>>,
    camera: Query<
        'w,
        's,
        (
            &'static mut Camera,
            &'static mut Transform,
            &'static mut OrthographicProjection,
        ),
    >,
    localization: Res<'w, Localization>,
    settings: Res<'w, EditorViewSettings>,
}

struct MapCreateInfo {
    name: String,
    map_width: u32,
    map_height: u32,
}

impl Default for MapCreateInfo {
    fn default() -> Self {
        Self {
            name: default(),
            map_width: 27,
            map_height: 21,
        }
    }
}

impl MapCreateInfo {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.map_width > 10 && self.map_height > 10
    }
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
        let has_map = params.map.get_single().is_ok();

        map_create_dialog(ui, &mut params);

        if has_map {
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
                let drag_delta =
                    response.drag_delta() * params.game.ui_theme.scale * projection.scale;
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
                physical_size: UVec2::new(
                    (rect.width() * ppp) as u32,
                    (rect.height() * ppp) as u32,
                ),
                ..default()
            });
        } else {
            ui.add_space(ui.available_height() / 2.0);
            ui.vertical_centered(|ui| {
                if BorderedButton::themed(
                    &params.game.ui_theme.button_styles.normal,
                    &params.localization.get("open-map"),
                )
                .show(ui)
                .clicked()
                {
                    error!("Unimplemented");
                }

                ui.add_space(ui.spacing().item_spacing.y);

                if BorderedButton::themed(
                    &params.game.ui_theme.button_styles.normal,
                    &params.localization.get("create-map"),
                )
                .show(ui)
                .clicked()
                {
                    *params.show_map_create = true;
                    *params.map_create_info = default();
                }
            });
        }
    }
}

fn map_create_dialog(ui: &mut egui::Ui, params: &mut EditorCentralPanel) {
    let space = ui.spacing().icon_width;

    if *params.show_map_create {
        let is_valid = params.map_create_info.is_valid();
        themed_overlay_window(
            ui,
            "create-map-window",
            &params.localization.get("create-map"),
            params.game.main_menu.menu_width,
            |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(&params.localization.get("name"));
                        ui.text_edit_singleline(&mut params.map_create_info.name);
                    });

                    ui.add_space(space / 2.0);

                    ui.horizontal(|ui| {
                        ui.label(&params.localization.get("tilemap-size"));
                        ui.add(egui::DragValue::new(&mut params.map_create_info.map_width));
                        ui.label("X");
                        ui.add(egui::DragValue::new(&mut params.map_create_info.map_height));
                    });

                    ui.add_space(space);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.scope(|ui| {
                            ui.set_enabled(is_valid);

                            if BorderedButton::themed(
                                &params.game.ui_theme.button_styles.small,
                                &params.localization.get("create"),
                            )
                            .show(ui)
                            .clicked()
                            {
                                *params.show_map_create = false;
                                create_map(params);
                            }
                        });

                        ui.add_space(space);

                        if BorderedButton::themed(
                            &params.game.ui_theme.button_styles.small,
                            &params.localization.get("cancel"),
                        )
                        .show(ui)
                        .clicked()
                        {
                            *params.show_map_create = false;
                        }
                    });
                });
            },
        );
    }
}

fn create_map(params: &mut EditorCentralPanel) {
    let info = &params.map_create_info;
    let grid_size = UVec2::new(info.map_width, info.map_height);
    let tile_size = UVec2::new(10, 10);

    let grid = GeometryBuilder::build_as(
        &grid::Grid {
            grid_size,
            tile_size,
        },
        DrawMode::Stroke(StrokeMode::new(Color::rgba(0.8, 0.8, 0.8, 0.8), 0.25)),
        default(),
    );

    params
        .commands
        .spawn()
        .insert(MapMeta {
            grid_size,
            tile_size,
            ..default()
        })
        .insert_bundle(VisibilityBundle::default())
        .insert_bundle(TransformBundle {
            local: Transform::from_xyz(
                (grid_size.x * tile_size.x) as f32 / -2.0,
                (grid_size.y * tile_size.y) as f32 / -2.0,
                0.0,
            ),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn()
                .insert(MapGridView)
                .insert_bundle(grid)
                .insert_bundle(VisibilityBundle {
                    visibility: Visibility {
                        is_visible: params.settings.show_grid,
                    },
                    ..default()
                });
        });
}

mod grid {
    use bevy_prototype_lyon::prelude::tess::{
        geom::{
            euclid::{Point2D, Size2D},
            Rect,
        },
        path::traits::PathBuilder,
    };

    use super::*;

    pub struct Grid {
        pub grid_size: UVec2,
        pub tile_size: UVec2,
    }

    impl Geometry for Grid {
        fn add_geometry(&self, b: &mut tess::path::path::Builder) {
            for x in 0..self.grid_size.x {
                for y in 0..self.grid_size.y {
                    b.add_rectangle(
                        &Rect {
                            origin: Point2D::new(
                                x as f32 * self.tile_size.x as f32,
                                y as f32 * self.tile_size.y as f32,
                            ),
                            size: Size2D::new(self.tile_size.x as f32, self.tile_size.y as f32),
                        },
                        tess::path::Winding::Positive,
                    );
                }
            }
        }
    }
}

/// Helper to render an egui frame in the center of the screen as an overlay
fn themed_overlay_window<R, F: FnOnce(&mut egui::Ui) -> R>(
    ui: &mut egui::Ui,
    id: &str,
    title: &str,
    width: f32,
    f: F,
) -> egui::InnerResponse<R> {
    let space = ui.spacing().icon_width;
    let i = egui::Window::new(title)
        .auto_sized()
        .id(egui::Id::new(id))
        .frame(
            egui::Frame::window(ui.style()).inner_margin(egui::style::Margin::symmetric(
                space,
                ui.spacing().window_margin.top,
            )),
        )
        .default_width(width)
        .collapsible(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        // .frame(egui::Frame::none())
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(title);
            });
            ui.separator();
            ui.add_space(space);
            let r = f(ui);
            ui.add_space(space / 2.0);
            r
        })
        .unwrap();

    egui::InnerResponse {
        inner: i.inner.unwrap(),
        response: i.response,
    }
}
