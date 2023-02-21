use std::{marker::PhantomData, mem::discriminant};

use bevy::{
    ecs::system::{Command, SystemParam, SystemState},
    math::Vec3Swizzles,
};
use bevy_egui::*;
use bevy_fluent::Localization;
use bones_bevy_renderer::BevyBonesEntity;

use crate::prelude::*;

use super::{widget, widgets::bordered_button::BorderedButton, WidgetSystem};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>()
            .add_system(
                editor_ui_system
                    .run_in_state(EngineState::InGame)
                    .run_in_state(GameEditorState::Visible)
                    .at_end(),
            )
            .add_enter_system(GameEditorState::Visible, setup_editor)
            .add_exit_system(GameEditorState::Visible, cleanup_editor);
    }
}

#[derive(Resource, Default)]
struct EditorState {
    pub current_layer_idx: usize,
    // pub hidden_layers: HashSet<usize>,
}

/// Bevy [`Command`] for centering the game camera.
///
/// TODO: Maybe move this logic to the [`SessionManager`] and add a way to load a map with the
/// camera initially set centered over the map.
struct CenterGameCamera;
impl Command for CenterGameCamera {
    fn write(self, world: &mut World) {
        let mut state = SystemState::<(Res<Assets<MapMeta>>, Option<ResMut<Session>>)>::new(world);
        let (map_assets, session) = state.get_mut(world);

        if let Some(session) = session {
            let map_handle = session.world.resource::<jumpy_core::map::MapHandle>();
            let map_handle = map_handle.borrow();
            let map = map_assets.get(&map_handle.get_bevy_handle()).unwrap();
            let (grid_size, tile_size) = (map.grid_size, map.tile_size);

            session
                .world
                .run_initialized_system(move |mut commands: bones::Commands| {
                    // Using commands here instead of directly will make sure that it waits the next
                    // frame until the camera is spawned.
                    commands.add(
                        move |mut cameras: bones::CompMut<bones::Camera>,
                              mut camera_shakes: bones::CompMut<bones::CameraShake>,
                              core_meta: bones::Res<CoreMetaArc>| {
                            let camera = cameras.iter_mut().next().unwrap();
                            let camera_shake = camera_shakes.iter_mut().next().unwrap();

                            camera.height = core_meta.camera.default_height * 2.0;
                            camera_shake.center =
                                (tile_size * (grid_size / 2).as_vec2()).extend(0.0);
                        },
                    )
                })
                .unwrap();
        }
    }
}

pub fn setup_editor(mut commands: Commands) {
    commands.add(CenterGameCamera);
}

pub fn cleanup_editor(session: Option<ResMut<Session>>) {
    // Update camera viewport to fit into central editor area.
    if let Some(session) = session {
        let cameras = session.world.components.get::<bones::Camera>();
        let mut cameras = cameras.borrow_mut();
        let camera = cameras.iter_mut().next().unwrap();
        camera.viewport = None;
        camera.height = bones::Camera::default().height;

        // Enable the default camera controller
        let camera_states = session
            .world
            .components
            .get::<jumpy_core::camera::CameraState>();
        let mut camera_states = camera_states.borrow_mut();
        camera_states.iter_mut().next().unwrap().disable_controller = false;
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

type CameraQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut OrthographicProjection),
    With<BevyBonesEntity>,
>;

#[derive(SystemParam)]
struct EditorTopBar<'w, 's> {
    commands: Commands<'w, 's>,
    game: Res<'w, GameMeta>,
    core_meta: Res<'w, CoreMetaArc>,
    map_assets: Res<'w, Assets<MapMeta>>,
    show_map_export_window: Local<'s, bool>,
    localization: Res<'w, Localization>,
    session_manager: SessionManager<'w, 's>,
    camera: CameraQuery<'w, 's>,
    clipboard: ResMut<'w, bevy_egui::EguiClipboard>,
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

        map_export_window(ui, &mut params);

        ui.horizontal_centered(|ui| {
            ui.label(&params.localization.get("map-editor"));
            ui.separator();

            if let Ok((transform, projection)) = params.camera.get_single() {
                let height = match projection.scaling_mode {
                    bevy::render::camera::ScalingMode::FixedVertical(height) => height,
                    _ => 1.0, // This shouldn't happen for now
                };
                let zoom = params.core_meta.camera.default_height / height * 100.0;
                let [x, y]: [f32; 2] = transform.translation.xy().into();

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
            }

            ui.add_space(ui.spacing().icon_spacing);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(&params.localization.get("main-menu")).clicked() {
                    params
                        .commands
                        .insert_resource(NextState(EngineState::MainMenu));
                }

                ui.scope(|ui| {
                    ui.set_enabled(params.session_manager.session.is_some());
                    if ui.button(&params.localization.get("play")).clicked() {
                        params
                            .commands
                            .insert_resource(NextState(GameEditorState::Hidden));
                    }

                    if ui.button(&params.localization.get("export")).clicked() {
                        *params.show_map_export_window = true;
                    }

                    if ui.button(&params.localization.get("close")).clicked() {
                        params.session_manager.stop();
                    }

                    if ui.button(&params.localization.get("reload")).clicked() {
                        params.session_manager.restart();
                        params
                            .commands
                            .insert_resource(NextState(InGameState::Playing));
                        params.commands.add(CenterGameCamera);
                    }
                });

                ui.label(
                    egui::RichText::new(params.localization.get("map-editor-preview-warning"))
                        .color(egui::Color32::RED),
                );
            });
        });
    }
}

fn map_export_window(ui: &mut egui::Ui, params: &mut EditorTopBar) {
    if !*params.show_map_export_window {
        return;
    }
    // let map = params.map_meta.single();
    overlay_window(
        ui,
        "export-map-window",
        &params.localization.get("map-export"),
        params.game.main_menu.menu_width,
        |ui| {
            let Some(session) = params.session_manager.session.as_mut() else { return };
            let map_handle = session.world.resource::<jumpy_core::map::MapHandle>();
            let map_handle = map_handle.borrow();
            let map_meta = params
                .map_assets
                .get(&map_handle.get_bevy_handle())
                .unwrap();
            let mut export = serde_yaml::to_string(map_meta).unwrap();

            ui.vertical(|ui| {
                ui.set_height(
                    params.core_meta.camera.default_height * 0.6 * params.game.ui_theme.scale,
                );
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut export)
                            .font(egui::TextStyle::Monospace) // for cursor height
                            .code_editor()
                            .desired_width(ui.available_width())
                            .lock_focus(true),
                    );
                });

                ui.add_space(ui.spacing().icon_width);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if BorderedButton::themed(
                        &params.game.ui_theme.button_styles.small,
                        &params.localization.get("close"),
                    )
                    .focus_on_hover(false)
                    .show(ui)
                    .clicked()
                    {
                        *params.show_map_export_window = false;
                    }

                    if BorderedButton::themed(
                        &params.game.ui_theme.button_styles.small,
                        &params.localization.get("copy-to-clipboard"),
                    )
                    .focus_on_hover(false)
                    .show(ui)
                    .clicked()
                    {
                        params.clipboard.set_contents(&export);
                    }
                });
            });
        },
    );
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

struct LayerCreateInfo {
    name: String,
    kind: MapLayerKind,
}

impl Default for LayerCreateInfo {
    fn default() -> Self {
        Self {
            name: Default::default(),
            kind: MapLayerKind::Tile(default()),
        }
    }
}

#[derive(SystemParam)]
struct EditorRightToolbar<'w, 's> {
    map_assets: Res<'w, Assets<MapMeta>>,
    show_layer_create: Local<'s, bool>,
    layer_create_info: Local<'s, LayerCreateInfo>,
    game: Res<'w, GameMeta>,
    localization: Res<'w, Localization>,
    state: ResMut<'w, EditorState>,
    session_manager: SessionManager<'w, 's>,
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
        let mut params: EditorRightToolbar = state.get_mut(world);
        layer_create_dialog(ui, &mut params);

        let map_meta = params
            .session_manager
            .map_handle()
            .and_then(|handle| params.map_assets.get(&handle));
        ui.set_enabled(map_meta.is_some());

        ui.add_space(ui.spacing().window_margin.top);

        ui.horizontal(|ui| {
            ui.label(&params.localization.get("map-info"));
        });
        ui.separator();

        let row_height = ui.spacing().interact_size.y;
        ui.push_id("info", |ui| {
            let table = egui_extras::TableBuilder::new(ui)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::remainder())
                .resizable(false);

            table.body(|mut body| {
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.label(&params.localization.get("name"));
                    });
                    row.col(|ui| {
                        ui.label(map_meta.map(|map| map.name.as_str()).unwrap_or(""));
                    });
                });
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.label(&params.localization.get("grid-size"));
                    });
                    if let Some(map) = map_meta {
                        let x = map.grid_size.x;
                        let y = map.grid_size.y;
                        row.col(|ui| {
                            ui.label(format!("{x} x {y}"));
                        });
                    }
                });
            });
        });

        ui.add_space(ui.spacing().icon_width);

        ui.separator();
        ui.horizontal(|ui| {
            ui.label(&params.localization.get("layers"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button("➕")
                    .on_hover_text(params.localization.get("create-layer"))
                    .clicked()
                {
                    *params.show_layer_create = true;
                }
            });
        });
        ui.separator();

        if let Some(map) = map_meta {
            let row_height = ui.spacing().interact_size.y * 1.4;
            ui.push_id("layers", |ui| {
                let width = ui.available_width() - ui.spacing().item_spacing.x * 4.0;
                for (i, layer) in map.layers.iter().enumerate() {
                    let layer: &MapLayerMeta = layer;

                    ui.horizontal(|ui| {
                        ui.set_width(ui.available_width());
                        ui.set_height(row_height);

                        let row_rect = ui.max_rect();

                        let hovered = ui
                            .input()
                            .pointer
                            .hover_pos()
                            .map(|pos| row_rect.contains(pos))
                            .unwrap_or(false);
                        let active = hovered && ui.input().pointer.primary_down();
                        let highlighted = hovered || params.state.current_layer_idx == i;
                        let clicked = ui.input().pointer.primary_released() && hovered;

                        if highlighted {
                            ui.painter().rect_filled(
                                row_rect,
                                0.0,
                                if active {
                                    ui.visuals().widgets.active.bg_stroke.color
                                } else {
                                    ui.visuals().widgets.hovered.bg_fill
                                },
                            );
                        }

                        if clicked {
                            params.state.current_layer_idx = i;
                        }

                        ui.scope(|ui| {
                            ui.set_width(width * 0.1);
                            ui.vertical_centered(|ui| {
                                ui.add_space(ui.spacing().interact_size.y * 0.2);
                                match layer.kind {
                                    MapLayerKind::Tile(_) => {
                                        ui.label(&params.localization.get("tile-layer-icon"))
                                            .on_hover_text(params.localization.get("tile-layer"));
                                    }
                                    MapLayerKind::Element(_) => {
                                        ui.label(&params.localization.get("element-layer-icon"))
                                            .on_hover_text(
                                                params.localization.get("element-layer"),
                                            );
                                    }
                                };
                            });
                        });

                        ui.vertical(|ui| {
                            ui.set_width(width * 0.8);
                            ui.add_space(ui.spacing().interact_size.y * 0.2);
                            ui.label(&layer.id);
                        });

                        // ui.vertical_centered(|ui| {
                        //     ui.set_width(width * 0.1);
                        //     ui.add_space(ui.spacing().interact_size.y * 0.2);
                        //     let is_visible = !params.state.hidden_layers.contains(&i);
                        //     if ui
                        //         .selectable_label(is_visible, "👁")
                        //         .on_hover_text(params.localization.get("toggle-visibility"))
                        //         .clicked()
                        //     {
                        //         if is_visible {
                        //             params.state.hidden_layers.insert(i);
                        //         } else {
                        //             params.state.hidden_layers.remove(&i);
                        //         }
                        //     };
                        // });
                    });
                }
            });
        }
    }
}

fn layer_create_dialog(ui: &mut egui::Ui, params: &mut EditorRightToolbar) {
    let space = ui.spacing().icon_width;

    if !*params.show_layer_create {
        return;
    }

    // let is_valid = params.map.get_single().is_ok();
    let is_valid = false;
    overlay_window(
        ui,
        "create-map-window",
        &params.localization.get("create-layer"),
        params.game.main_menu.menu_width,
        |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(&params.localization.get("name"));
                    ui.text_edit_singleline(&mut params.layer_create_info.name);
                });

                ui.add_space(space / 2.0);

                ui.horizontal(|ui| {
                    ui.label(&format!("{}: ", params.localization.get("layer-kind")));
                    ui.add_space(space);
                    for (label, layer_kind) in [
                        (
                            params.localization.get("tile"),
                            MapLayerKind::Tile(default()),
                        ),
                        (
                            params.localization.get("element"),
                            MapLayerKind::Element(default()),
                        ),
                    ] {
                        let selected = discriminant(&params.layer_create_info.kind)
                            == discriminant(&layer_kind);

                        if ui.selectable_label(selected, label).clicked() {
                            params.layer_create_info.kind = layer_kind;
                        }
                    }
                });

                ui.add_space(space);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.scope(|ui| {
                        ui.set_enabled(is_valid);

                        if BorderedButton::themed(
                            &params.game.ui_theme.button_styles.small,
                            &params.localization.get("create"),
                        )
                        .focus_on_hover(false)
                        .show(ui)
                        .clicked()
                        {
                            *params.show_layer_create = false;
                            create_layer(params);
                        }
                    });

                    ui.add_space(space);

                    if BorderedButton::themed(
                        &params.game.ui_theme.button_styles.small,
                        &params.localization.get("cancel"),
                    )
                    .focus_on_hover(false)
                    .show(ui)
                    .clicked()
                    {
                        *params.layer_create_info = default();
                        *params.show_layer_create = false;
                    }
                });
            });
        },
    );
}

fn create_layer(_params: &mut EditorRightToolbar) {
    // let layer_info = &*params.layer_create_info;

    todo!();
}

#[derive(SystemParam)]
struct EditorCentralPanel<'w, 's> {
    show_map_create: Local<'s, bool>,
    show_map_open: Local<'s, bool>,
    map_create_info: Local<'s, MapCreateInfo>,
    game: Res<'w, GameMeta>,
    core_meta: Res<'w, CoreMetaArc>,
    map_assets: Res<'w, Assets<MapMeta>>,
    localization: Res<'w, Localization>,
    session_manager: SessionManager<'w, 's>,
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

        map_open_dialog(ui, &mut params);
        map_create_dialog(ui, &mut params);

        if *params.show_map_create || *params.show_map_open {
            ui.set_enabled(false);
        }

        if params.session_manager.session.is_some() {
            let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

            let rect = response.rect;

            'camera_control: {
                if let Some(session) = &mut params.session_manager.session {
                    let ppp = ui.ctx().pixels_per_point();

                    let core_meta = session.world.resource::<CoreMetaArc>();
                    let core_meta = core_meta.borrow();

                    let cameras = session.world.components.get::<bones::Camera>();
                    let mut cameras = cameras.borrow_mut();

                    // Update camera viewport to fit into central editor area.
                    let Some(camera) = cameras.iter_mut().next() else {
                        break 'camera_control;
                    };
                    camera.viewport = Some(bones::Viewport {
                        position: UVec2::new(
                            (rect.min.x * ppp) as u32,
                            (rect.min.y.floor() * ppp) as u32,
                        ),
                        size: UVec2::new((rect.width() * ppp) as u32, (rect.height() * ppp) as u32),
                        depth_min: 0.0,
                        depth_max: 1.0,
                    });

                    // Disable the default camera controller
                    let camera_states = session
                        .world
                        .components
                        .get::<jumpy_core::camera::CameraState>();
                    let mut camera_states = camera_states.borrow_mut();
                    camera_states.iter_mut().next().unwrap().disable_controller = true;

                    // Handle camera zoom
                    if response.hovered() {
                        camera.height -= ui.input().scroll_delta.y;
                        camera.height = camera.height.max(10.0);
                    }

                    // Handle camera pan
                    if response.dragged_by(egui::PointerButton::Middle)
                        || ui.input().modifiers.command
                    {
                        let camera_shakes = session.world.components.get::<bones::CameraShake>();
                        let mut camera_shakes = camera_shakes.borrow_mut();
                        let camera_shake = camera_shakes.iter_mut().next().unwrap();

                        let drag_delta =
                            response.drag_delta() * params.game.ui_theme.scale * camera.height
                                / core_meta.camera.default_height;
                        camera_shake.center.x -= drag_delta.x;
                        camera_shake.center.y += drag_delta.y;
                    }
                }
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
                    *params.show_map_open = true;
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

fn map_open_dialog(ui: &mut egui::Ui, params: &mut EditorCentralPanel) {
    let space = ui.spacing().icon_width;

    if !*params.show_map_open {
        return;
    }

    overlay_window(
        ui,
        "open-map-window",
        &params.localization.get("open-map"),
        params.game.main_menu.menu_width,
        |ui| {
            // ui.set_height(params.game.camera_height as f32 * 0.6);
            ui.vertical_centered_justified(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    #[allow(clippy::unnecessary_to_owned)] // False alarm
                    for map_handle in params
                        .core_meta
                        .stable_maps
                        .to_vec()
                        .into_iter()
                        .chain(params.core_meta.experimental_maps.to_vec().into_iter())
                    {
                        let map_name = &params
                            .map_assets
                            .get(&map_handle.get_bevy_handle())
                            .unwrap()
                            .name;
                        if ui.button(map_name).clicked() {
                            params.session_manager.start(GameSessionInfo {
                                meta: params.core_meta.0.clone(),
                                map: map_handle.clone(),
                                player_info: default(),
                            });
                            *params.show_map_open = false;
                            // TODO: center camera.
                        }
                    }
                });
            });

            ui.add_space(space);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if BorderedButton::themed(
                    &params.game.ui_theme.button_styles.small,
                    &params.localization.get("cancel"),
                )
                .focus_on_hover(false)
                .show(ui)
                .clicked()
                {
                    *params.show_map_open = false;
                }
            });
        },
    );
}

fn map_create_dialog(ui: &mut egui::Ui, params: &mut EditorCentralPanel) {
    let space = ui.spacing().icon_width;

    if !*params.show_map_create {
        return;
    }

    let is_valid = params.map_create_info.is_valid();
    overlay_window(
        ui,
        "create-map-window",
        &params.localization.get("create-map"),
        params.game.main_menu.menu_width,
        |ui| {
            ui.horizontal(|ui| {
                ui.label(&params.localization.get("name"));
                ui.text_edit_singleline(&mut params.map_create_info.name);
            });

            ui.add_space(space / 2.0);

            ui.horizontal(|ui| {
                ui.label(&params.localization.get("grid-size"));
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
                    .focus_on_hover(false)
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
                .focus_on_hover(false)
                .show(ui)
                .clicked()
                {
                    *params.show_map_create = false;
                }
            });
        },
    );
}

fn create_map(params: &mut EditorCentralPanel) {
    // let info = &params.map_create_info;
    // let grid_size = UVec2::new(info.map_width, info.map_height);
    // let tile_size = UVec2::new(10, 10);
    // let meta = MapMeta {
    //     background_color: params.game.clear_color,
    //     name: info.name.clone(),
    //     grid_size,
    //     tile_size,
    //     layers: default(),
    //     background_layers: default(),
    // };

    // params.commands.spawn().insert(meta);
    *params.show_map_open = false;
    *params.show_map_create = false;

    unimplemented!();
}

/// Helper to render an egui frame in the center of the screen as an overlay
fn overlay_window<R, F: FnOnce(&mut egui::Ui) -> R>(
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
        .show(ui.ctx(), |ui| {
            ui.vertical(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(title);
                });
                ui.separator();
                ui.add_space(space);
                let r = f(ui);
                ui.add_space(space / 2.0);
                r
            })
            .inner
        })
        .unwrap();

    egui::InnerResponse {
        inner: i.inner.unwrap(),
        response: i.response,
    }
}
