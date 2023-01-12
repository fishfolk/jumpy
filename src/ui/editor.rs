use std::{marker::PhantomData, mem::discriminant};

use bevy::{
    ecs::system::SystemParam, math::Vec3Swizzles, render::camera::Viewport, utils::HashSet,
};
use bevy_egui::*;
use bevy_fluent::Localization;

use crate::prelude::*;

use super::{widget, widgets::bordered_button::BorderedButton, WidgetSystem};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>()
            .add_system(
                editor_update
                    .run_in_state(GameState::InGame)
                    .run_in_state(GameEditorState::Visible),
            )
            .add_system(
                editor_ui_system
                    .into_conditional_exclusive()
                    .run_in_state(GameState::InGame)
                    .run_in_state(GameEditorState::Visible)
                    .at_end(),
            )
            .add_enter_system(GameEditorState::Visible, setup_editor)
            .add_exit_system(GameEditorState::Visible, cleanup_editor);
    }
}

struct EditorState {
    pub show_grid: bool,
    pub current_layer_idx: usize,
    pub hidden_layers: HashSet<usize>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            show_grid: true,
            current_layer_idx: 0,
            hidden_layers: default(),
        }
    }
}

fn setup_editor(
    game: Res<GameMeta>,
    mut game_camera: Query<&mut Camera, With<GameCamera>>,
    mut editor_camera: Query<&mut Camera, (With<EditorCamera>, Without<GameCamera>)>,
    mut player_inputs: ResMut<PlayerInputs>,
) {
    // Disable the game camera
    let mut game_camera = game_camera.single_mut();
    game_camera.is_active = false;

    // Make sure there is a player on the map so that there is somebody to use when pressing "Play"
    if !player_inputs.players[0].active {
        player_inputs.players[0].active = true;
        player_inputs.players[0].selected_player = game.player_handles[0].clone_weak();
    }

    // Enable editor camera
    //
    // TODO: Fit map inside of viewport
    let mut camera = editor_camera.single_mut();
    camera.is_active = true;
}

fn editor_update(
    mut map_grid: Query<&mut Visibility, With<MapGridView>>,
    settings: Res<EditorState>,
) {
    for mut visibility in &mut map_grid {
        visibility.is_visible = settings.show_grid;
    }
}

fn cleanup_editor(
    mut game_camera: Query<&mut Camera, With<GameCamera>>,
    mut editor_camera: Query<&mut Camera, (With<EditorCamera>, Without<GameCamera>)>,
) {
    // Enable the game camera
    let mut game_camera = game_camera.single_mut();
    game_camera.is_active = true;

    // Disable the editor camera
    let mut editor_camera = editor_camera.single_mut();
    editor_camera.is_active = false;
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
    game: Res<'w, GameMeta>,
    camera_commands_resetcontroller: ParamSet<
        'w,
        's,
        (
            Query<
                'w,
                's,
                (&'static mut Transform, &'static mut OrthographicProjection),
                With<EditorCamera>,
            >,
            Commands<'w, 's>,
            ResetManager<'w, 's>,
        ),
    >,
    rids: ResMut<'w, RollbackIdProvider>,
    show_map_export_window: Local<'s, bool>,
    localization: Res<'w, Localization>,
    map_meta: Query<'w, 's, &'static MapMeta>,
    map_handle: Query<'w, 's, &'static AssetHandle<MapMeta>>,
    settings: ResMut<'w, EditorState>,
    session_manager: SessionManager<'w, 's>,
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
            let mut camera = params.camera_commands_resetcontroller.p0();
            let (mut transform, mut projection): (Mut<Transform>, Mut<OrthographicProjection>) =
                camera.single_mut();
            let zoom = 1.0 / projection.scale * 100.0;
            let [x, y]: [f32; 2] = transform.translation.xy().into();

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
                        .camera_commands_resetcontroller
                        .p1()
                        .insert_resource(NextState(GameState::MainMenu));
                }

                ui.scope(|ui| {
                    ui.set_enabled(params.map_meta.get_single().is_ok());
                    if ui.button(&params.localization.get("play")).clicked() {
                        params
                            .camera_commands_resetcontroller
                            .p1()
                            .insert_resource(NextState(GameEditorState::Hidden));
                    }

                    let mut reset_controller = params.camera_commands_resetcontroller.p2();
                    if ui.button(&params.localization.get("export")).clicked() {
                        *params.show_map_export_window = true;
                    }

                    if ui.button(&params.localization.get("close")).clicked() {
                        reset_controller.reset_world();
                    }

                    if ui.button(&params.localization.get("reload")).clicked() {
                        let map_handle = params.map_handle.get_single().ok().cloned();
                        reset_controller.reset_world();

                        if let Some(handle) = map_handle {
                            params
                                .camera_commands_resetcontroller
                                .p1()
                                .spawn()
                                .insert(handle)
                                .insert(Rollback::new(params.rids.next_id()));
                            params.session_manager.start_session();
                        }
                    }
                });
            });
        });
    }
}

fn map_export_window(ui: &mut egui::Ui, params: &mut EditorTopBar) {
    if !*params.show_map_export_window {
        return;
    }
    let map = params.map_meta.single();
    overlay_window(
        ui,
        "export-map-window",
        &params.localization.get("map-export"),
        params.game.main_menu.menu_width,
        |ui| {
            let mut export = serde_yaml::to_string(map).expect("Failure to export to YAMl");

            ui.vertical(|ui| {
                ui.set_height(params.game.camera_height as f32 * 0.6 * params.game.ui_theme.scale);
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
    commands: Commands<'w, 's>,
    show_layer_create: Local<'s, bool>,
    layer_create_info: Local<'s, LayerCreateInfo>,
    game: Res<'w, GameMeta>,
    localization: Res<'w, Localization>,
    map: Query<'w, 's, (Entity, &'static mut MapMeta)>,
    state: ResMut<'w, EditorState>,
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

        let map_query: Option<(_, &MapMeta)> = params.map.get_single().ok();

        ui.set_enabled(map_query.is_some());
        ui.add_space(ui.spacing().window_margin.top);

        ui.horizontal(|ui| {
            ui.label(&params.localization.get("map-info"));
        });
        ui.separator();

        let row_height = ui.spacing().interact_size.y;
        ui.push_id("info", |ui| {
            let table = egui_extras::TableBuilder::new(ui)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Size::relative(0.5))
                .column(egui_extras::Size::remainder())
                .resizable(false);

            table.body(|mut body| {
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.label(&params.localization.get("name"));
                    });
                    row.col(|ui| {
                        ui.label(map_query.map(|(_, map)| map.name.as_str()).unwrap_or(""));
                    });
                });
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.label(&params.localization.get("grid-size"));
                    });
                    if let Some((_, map)) = map_query {
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

        if let Some((_, map)) = map_query {
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

                        ui.vertical_centered(|ui| {
                            ui.set_width(width * 0.1);
                            ui.add_space(ui.spacing().interact_size.y * 0.2);
                            let is_visible = !params.state.hidden_layers.contains(&i);
                            if ui
                                .selectable_label(is_visible, "👁")
                                .on_hover_text(params.localization.get("toggle-visibility"))
                                .clicked()
                            {
                                if is_visible {
                                    params.state.hidden_layers.insert(i);
                                } else {
                                    params.state.hidden_layers.remove(&i);
                                }
                            };
                        });
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

    let is_valid = params.map.get_single().is_ok();
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

fn create_layer(params: &mut EditorRightToolbar) {
    use bevy_ecs_tilemap::prelude::*;

    let layer_info = &*params.layer_create_info;
    let (map_entity, mut map): (Entity, Mut<MapMeta>) = params.map.single_mut();

    let layer_idx = map.layers.len();
    let layer_entity = params.commands.spawn().id();
    map.layers.push(MapLayerMeta {
        id: layer_info.name.clone(),
        kind: layer_info.kind.clone(),
        entity: Some(layer_entity),
    });

    match &layer_info.kind {
        MapLayerKind::Tile(_) => {
            let grid_size = TilemapSize {
                x: map.grid_size.x,
                y: map.grid_size.y,
            };
            let tile_size = TilemapTileSize {
                x: map.tile_size.x as f32,
                y: map.tile_size.y as f32,
            };

            let storage = TileStorage::empty(grid_size);

            // Spawn the map layer
            params
                .commands
                .entity(map_entity)
                .push_children(&[layer_entity]);
            params
                .commands
                .entity(layer_entity)
                .insert_bundle(TilemapBundle {
                    size: grid_size,
                    storage,
                    tile_size,
                    transform: Transform::from_xyz(0.0, 0.0, layer_idx as f32),
                    ..default()
                });
        }
        MapLayerKind::Element(_) => (),
    }

    *params.layer_create_info = default();
}

#[derive(SystemParam)]
struct EditorCentralPanel<'w, 's> {
    show_map_create: Local<'s, bool>,
    show_map_open: Local<'s, bool>,
    map_create_info: Local<'s, MapCreateInfo>,
    commands: Commands<'w, 's>,
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
        With<EditorCamera>,
    >,
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
        let has_map = params.map.get_single().is_ok();

        map_open_dialog(ui, &mut params);
        map_create_dialog(ui, &mut params);

        if *params.show_map_create || *params.show_map_open {
            ui.set_enabled(false);
        }

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
            ui.set_height(params.game.camera_height as f32 * 0.6);
            ui.vertical_centered_justified(|ui| {
                ui.add_space(space * 2.5);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    #[allow(clippy::unnecessary_to_owned)]
                    // False alarm, necessary to avoid borrowing params
                    for (map_name, map_handle) in params
                        .game
                        .stable_maps
                        .to_vec()
                        .into_iter()
                        .chain(params.game.experimental_maps.to_vec().into_iter())
                        .zip(
                            params
                                .game
                                .stable_map_handles
                                .iter()
                                .chain(params.game.experimental_map_handles.iter())
                                .map(|x| x.clone_weak())
                                .collect::<Vec<_>>()
                                .into_iter(),
                        )
                    {
                        if ui.button(&map_name).clicked() {
                            params.commands.spawn().insert(map_handle);
                            params.session_manager.start_session();
                            *params.show_map_open = false;
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
    let info = &params.map_create_info;
    let grid_size = UVec2::new(info.map_width, info.map_height);
    let tile_size = UVec2::new(10, 10);
    let meta = MapMeta {
        background_color: params.game.clear_color,
        name: info.name.clone(),
        grid_size,
        tile_size,
        layers: default(),
        background_layers: default(),
    };

    params.commands.spawn().insert(meta);
    *params.show_map_open = false;
    *params.show_map_create = false;
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
