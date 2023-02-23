use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, math::Vec3Swizzles};
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
            .add_exit_system(GameEditorState::Visible, cleanup_editor);
    }
}

/// Resource containing the current position of the mouse cursor in the editor.
#[derive(Default)]
struct EditorCursor {
    pub current_pos: Option<Vec2>,
    pub context_click_pos: Option<Vec2>,
}

#[derive(Resource, Default)]
struct EditorState {
    pub cursor: EditorCursor,
    pub current_layer_idx: usize,
    pub current_tool: EditorTool,
    pub camera: EditorCameraPos,
    // pub hidden_layers: HashSet<usize>,
}

#[derive(Clone, Copy)]
struct EditorCameraPos {
    pos: Vec2,
    height: f32,
}

impl Default for EditorCameraPos {
    fn default() -> Self {
        Self {
            pos: Vec2::new(500.0, 350.0),
            height: 800.0,
        }
    }
}

/// The current export of the world's map metadata, if a map is loaded.
#[derive(Resource, Default, Deref, DerefMut)]
struct EditorMapExport(Option<MapMeta>);

#[derive(Default, PartialEq, Eq)]
enum EditorTool {
    #[default]
    Element,
    Tile,
}

impl EditorTool {
    pub fn cursor(&self) -> egui::CursorIcon {
        match self {
            EditorTool::Element => egui::CursorIcon::Default,
            EditorTool::Tile => egui::CursorIcon::Crosshair,
        }
    }
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
    // Force set the camera position
    {
        let world = world.cell();
        let session = world.get_resource::<Session>();
        let editor_state = world.resource_mut::<EditorState>();
        let camera_info = editor_state.camera;
        if let Some(session) = session {
            session.world.run_initialized_system(
                move |
                mut cameras: bones::CompMut<bones::Camera>,
                mut camera_shakes: bones::CompMut<bones::CameraShake>,
                mut camera_states: bones::CompMut<jumpy_core::camera::CameraState>| {
                    let Some(camera) = cameras.iter_mut().next() else { return };
                    let camera_shake = camera_shakes.iter_mut().next().unwrap();
                    let camera_state = camera_states.iter_mut().next().unwrap();
                    camera.height = camera_info.height;
                    camera_shake.center = camera_info.pos.extend(0.0);
                    camera_state.disable_controller = true;
            }).ok();
        }
    }

    // Get the world cursor position
    let cursor_pos = {
        let mut camera_query =
            world.query_filtered::<(&Camera, &Transform), With<BevyBonesEntity>>();
        let windows = world.resource::<Windows>();
        let window = windows.primary();
        camera_query
            .get_single(world)
            .ok()
            .and_then(|(camera, transform)| {
                window
                    .cursor_position()
                    .and_then(|pos| {
                        camera.viewport_to_world(&GlobalTransform::from(*transform), pos)
                    })
                    .map(|x| x.origin.truncate())
            })
    };

    // Get the up-to-date map meta export from the world
    let map_meta = {
        world
            .get_resource::<Session>()
            .map(|sess| sess.export_map())
    };
    world.insert_resource(EditorMapExport(map_meta));

    let mut state = world.resource_mut::<EditorState>();
    state.cursor.current_pos = cursor_pos;

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
    (
        &'static Camera,
        &'static mut Transform,
        &'static mut OrthographicProjection,
    ),
    (With<BevyBonesEntity>, Without<MenuCamera>),
>;

#[derive(SystemParam)]
struct EditorTopBar<'w, 's> {
    commands: Commands<'w, 's>,
    game: Res<'w, GameMeta>,
    core_meta: Res<'w, CoreMetaArc>,
    show_map_export_window: Local<'s, bool>,
    state: Res<'w, EditorState>,
    localization: Res<'w, Localization>,
    session_manager: SessionManager<'w, 's>,
    camera: CameraQuery<'w, 's>,
    clipboard: ResMut<'w, bevy_egui::EguiClipboard>,
    map_export: Res<'w, EditorMapExport>,
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

            if let Ok((_camera, transform, projection)) = params.camera.get_single() {
                let height = match projection.scaling_mode {
                    bevy::render::camera::ScalingMode::FixedVertical(height) => height,
                    _ => 1.0, // This shouldn't happen for now
                };
                let zoom = params.core_meta.camera.default_height / height * 100.0;
                let [view_x, view_y]: [f32; 2] = transform.translation.xy().into();

                ui.label(
                    egui::RichText::new(
                        params
                            .localization
                            .get(&format!("view-offset?x={view_x:.0}&y={view_y:.0}")),
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
                if let Some(cursor_pos) = params.state.cursor.current_pos.as_ref() {
                    let (cursor_x, cursor_y) = (cursor_pos.x, cursor_pos.y);
                    ui.label(
                        egui::RichText::new(
                            params
                                .localization
                                .get(&format!("cursor-position?x={cursor_x:.0}&y={cursor_y:.0}")),
                        )
                        .small(),
                    );
                }
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
                        params.session_manager.stop();
                        params.session_manager.start(GameSessionInfo {
                            meta: params.core_meta.0.clone(),
                            map_meta: params.map_export.0.as_ref().unwrap().clone(),
                            player_info: default(),
                        });
                        params
                            .commands
                            .insert_resource(NextState(InGameState::Playing));
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
    // let map = params.map_meta.single();
    overlay_window(
        ui,
        "export-map-window",
        &params.localization.get("map-export"),
        params.game.main_menu.menu_width,
        |ui| {
            let Some(map_meta) = params.map_export.0.as_ref() else { return };
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
    state: ResMut<'w, EditorState>,
    localization: Res<'w, Localization>,
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
        let mut params: EditorLeftToolbar = state.get_mut(world);
        let icons = &params.game.ui_theme.editor.icons;
        let width = ui.available_width();
        for tool in [EditorTool::Element, EditorTool::Tile] {
            let (image, hover_text) = match tool {
                EditorTool::Element => (&icons.elements, params.localization.get("elements")),
                EditorTool::Tile => (&icons.tiles, params.localization.get("tiles")),
            };
            ui.add_space(ui.spacing().window_margin.top);

            let image_aspect = image.image_size.y / image.image_size.x;
            let height = width * image_aspect;
            let button = ui
                .add(
                    egui::ImageButton::new(image.egui_texture_id, egui::vec2(width, height))
                        .selected(params.state.current_tool == tool),
                )
                .on_hover_text_at_pointer(&hover_text);

            if button.clicked() {
                params.state.current_tool = tool;
            }
        }
    }
}

#[derive(Default)]
struct LayerCreateInfo {
    name: String,
}

#[derive(SystemParam)]
struct EditorRightToolbar<'w, 's> {
    show_layer_create: Local<'s, bool>,
    layer_create_info: Local<'s, LayerCreateInfo>,
    game: Res<'w, GameMeta>,
    localization: Res<'w, Localization>,
    state: ResMut<'w, EditorState>,
    editor_action: ResMut<'w, EditorAction>,
    map_export: Res<'w, EditorMapExport>,
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

        let map_meta = params.map_export.0.as_ref();

        ui.add_space(ui.spacing().window_margin.top);

        ui.horizontal(|ui| {
            ui.label(&params.localization.get("map-info"));
        });
        ui.separator();

        let row_height = ui.spacing().interact_size.y;
        ui.push_id("info", |ui| {
            ui.set_enabled(map_meta.is_some());
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
            ui.set_enabled(map_meta.is_some());
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
                        ui.add_space(ui.spacing().item_spacing.x);

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

    let is_valid = !params.layer_create_info.name.is_empty();
    overlay_window(
        ui,
        "create-layer-window",
        &params.localization.get("create-layer"),
        params.game.main_menu.menu_width,
        |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(&params.localization.get("name"));
                    ui.text_edit_singleline(&mut params.layer_create_info.name);
                });

                ui.add_space(space / 2.0);

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
                            **params.editor_action = Some(EditorInput::CreateLayer {
                                id: params.layer_create_info.name.clone(),
                            });
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

#[derive(SystemParam)]
struct EditorCentralPanel<'w, 's> {
    show_map_create: Local<'s, bool>,
    show_map_open: Local<'s, bool>,
    map_create_info: Local<'s, MapCreateInfo>,
    game: Res<'w, GameMeta>,
    core_meta: Res<'w, CoreMetaArc>,
    state: ResMut<'w, EditorState>,
    map_assets: Res<'w, Assets<MapMeta>>,
    element_assets: Res<'w, Assets<ElementMeta>>,
    localization: Res<'w, Localization>,
    session_manager: SessionManager<'w, 's>,
    editor_action: ResMut<'w, EditorAction>,
    camera: CameraQuery<'w, 's>,
    map: Res<'w, EditorMapExport>,
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

        // Collect map element list
        let element_handles: &Vec<bones::Handle<ElementMeta>> = &params.core_meta.map_elements;
        let mut element_categories =
            HashMap::<String, Vec<(bones::Handle<ElementMeta>, &ElementMeta)>>::new();
        element_handles
            .iter()
            .map(|handle| (handle.clone(), handle.get_bevy_handle()))
            .map(|(handle, bevy_handle)| (handle, params.element_assets.get(&bevy_handle).unwrap()))
            .for_each(|(handle, element)| {
                element_categories
                    .entry(element.category.clone())
                    .or_default()
                    .push((handle, element));
            });
        let mut element_categories = element_categories
            .into_iter()
            .map(|(k, mut v)| {
                v.sort_by_key(|x| &x.1.name);
                (k, v)
            })
            .collect::<Vec<_>>();
        element_categories.sort_by(|a, b| a.0.cmp(&b.0));

        if let Some(session) = params.session_manager.session {
            let core_meta = session.world.resource::<CoreMetaArc>();
            let core_meta = core_meta.borrow();

            let mut map_response =
                ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());
            let map_response_rect = map_response.rect;

            // Element context menu
            if let EditorTool::Element = params.state.current_tool {
                map_response = map_response.context_menu(|ui| {
                    if ui.input().pointer.secondary_clicked() {
                        params.state.cursor.context_click_pos = params.state.cursor.current_pos;
                    }
                    ui.menu_button(
                        &format!("➕ {}", params.localization.get("add-element")),
                        |ui| {
                            for (category, elements) in element_categories {
                                ui.menu_button(&category, |ui| {
                                    for (handle, element) in elements {
                                        if ui.button(&element.name).clicked() {
                                            **params.editor_action =
                                                Some(EditorInput::SpawnElement {
                                                    handle,
                                                    translation: params
                                                        .state
                                                        .cursor
                                                        .context_click_pos
                                                        .unwrap(),
                                                    layer: params
                                                        .state
                                                        .current_layer_idx
                                                        .try_into()
                                                        .unwrap(),
                                                });
                                            ui.close_menu();
                                            params.state.cursor.context_click_pos = None;
                                        }
                                    }
                                });
                            }
                        },
                    );
                });
            }

            // Move camera
            let camera_zoom = {
                let cursor_icon = ui.output().cursor_icon;
                let input = ui.input();
                let ctrl_modifier = input.modifiers.command;
                let pointer = &input.pointer;
                let editor_camera_pos = &mut params.state.camera;
                // Handle camera zoom
                let hovered = pointer
                    .hover_pos()
                    .map(|pos| map_response_rect.contains(pos))
                    .unwrap_or_default();
                if hovered {
                    editor_camera_pos.height -= input.scroll_delta.y;
                    editor_camera_pos.height = editor_camera_pos.height.max(10.0);
                }
                let zoom = editor_camera_pos.height / core_meta.camera.default_height;

                // Handle camera pan
                let panning = pointer.is_moving()
                    && (pointer.middle_down() || (ctrl_modifier && pointer.primary_down()));
                if panning {
                    let drag_delta = pointer.delta() * params.game.ui_theme.scale * zoom;
                    editor_camera_pos.pos.x -= drag_delta.x;
                    editor_camera_pos.pos.y += drag_delta.y;
                }
                drop(input);

                // Handle cursor
                //
                // We only change the cursor if it's not been changed by another widget, for instance, for the
                // resize handle of the right sidebar.
                if cursor_icon == default() {
                    if panning {
                        map_response.on_hover_cursor(egui::CursorIcon::Grabbing);
                    } else if ctrl_modifier {
                        map_response.on_hover_cursor(egui::CursorIcon::Grab);
                    } else {
                        map_response.on_hover_cursor(params.state.current_tool.cursor());
                    }
                }

                zoom
            };
            let ppp = params.game.ui_theme.scale * camera_zoom;

            // Collect map elements
            if params.state.current_tool == EditorTool::Element {
                if let Ok((camera, camera_transform, _)) = params.camera.get_single() {
                    let elements = session
                        .world
                        .run_initialized_system(
                            |entities: bones::Res<bones::Entities>,
                             transforms: bones::Comp<bones::Transform>,
                             element_handles: bones::Comp<jumpy_core::elements::ElementHandle>,
                             spawned_map_layer_metas: bones::Comp<
                                jumpy_core::map::SpawnedMapLayerMeta,
                            >| {
                                Ok(entities
                                    .iter_with((
                                        &element_handles,
                                        &transforms,
                                        &spawned_map_layer_metas,
                                    ))
                                    .map(|(ent, (handle, transform, layer))| {
                                        (
                                            ent,
                                            handle.get_bevy_handle(),
                                            transform.translation,
                                            layer.layer_idx,
                                        )
                                    })
                                    .collect::<Vec<_>>())
                            },
                        )
                        .unwrap();

                    for (entity, handle, translation, layer_idx) in elements {
                        if layer_idx != params.state.current_layer_idx {
                            continue;
                        }

                        let element_meta = params.element_assets.get(&handle).unwrap();
                        let grab_size = element_meta.editor.grab_size;
                        let grab_offset = element_meta.editor.grab_offset;

                        let screen_rect = ui.input().screen_rect();
                        let window_size = screen_rect.size();
                        let Some(ndc) = camera
                        .world_to_ndc(
                            &(*camera_transform).into(),
                            translation
                        ) else { continue };
                        let ndc = (ndc + 1.0) / 2.0;
                        let pos = egui::pos2(
                            window_size.x * ndc.x,
                            window_size.y - window_size.y * ndc.y,
                        );

                        let rect = egui::Rect::from_center_size(
                            pos + egui::vec2(grab_offset.x, -grab_offset.y) / ppp,
                            egui::vec2(grab_size.x, grab_size.y) / ppp,
                        );
                        let mut color_override = None;
                        let response = ui
                            .allocate_rect(rect, egui::Sense::click_and_drag())
                            .context_menu(|ui| {
                                color_override = Some(egui::Color32::RED);
                                if ui
                                    .button(&format!(
                                        "🗑 {}",
                                        params.localization.get("delete-element")
                                    ))
                                    .clicked()
                                {
                                    ui.close_menu();
                                    **params.editor_action =
                                        Some(EditorInput::DeleteEntity { entity });
                                }
                            });

                        #[derive(Clone)]
                        struct ElementDrag {
                            offset: Vec2,
                        }
                        let drag_id = egui::Id::from("element_drag");
                        if response.drag_started() {
                            ui.data().insert_temp(
                                drag_id,
                                ElementDrag {
                                    offset: params.state.cursor.current_pos.unwrap()
                                        - translation.truncate(),
                                },
                            );
                        } else if response.drag_released() {
                            ui.data().remove::<ElementDrag>(drag_id);
                        }

                        let half_pixel_offset = Vec2::new(
                            if grab_size.x % 2.0 != 0.0 { 0.5 } else { 0.0 },
                            if grab_size.y % 2.0 != 0.0 { 0.5 } else { 0.0 },
                        );
                        let snap_to_grid = ui.input().modifiers.shift_only();
                        let ctrl_modifier = ui.input().modifiers.command;

                        let default_color = if response.dragged_by(egui::PointerButton::Primary)
                            && map_response_rect.contains(ui.input().pointer.hover_pos().unwrap())
                            && !ctrl_modifier
                        {
                            let element_drag: ElementDrag = ui.data().get_temp(drag_id).unwrap();

                            let new_pos =
                                params.state.cursor.current_pos.unwrap() - element_drag.offset;

                            let new_pos = if snap_to_grid {
                                let bottom_center_offset =
                                    -grab_offset + Vec2::new(0.0, grab_size.y / 2.0);
                                let bottom_center = new_pos - bottom_center_offset;

                                let increment = params.map.0.as_ref().unwrap().tile_size / 4.0;
                                let snapped_bottom_center =
                                    (bottom_center / increment).floor() * increment;
                                snapped_bottom_center + bottom_center_offset
                            } else {
                                new_pos.floor() + half_pixel_offset
                            };

                            **params.editor_action = Some(EditorInput::MoveEntity {
                                entity,
                                pos: new_pos,
                            });
                            response.on_hover_cursor(egui::CursorIcon::Grabbing);
                            egui::Color32::GREEN
                        } else {
                            response.on_hover_cursor(egui::CursorIcon::PointingHand);
                            egui::Color32::LIGHT_GRAY
                        };
                        let mut painter = ui.painter_at(screen_rect);
                        let color = color_override.unwrap_or(default_color);
                        painter.set_clip_rect(map_response_rect);
                        if element_meta.editor.show_name {
                            painter.text(
                                rect.center_top(),
                                egui::Align2::CENTER_BOTTOM,
                                &element_meta.name,
                                egui::FontId::new(15.0, egui::FontFamily::Proportional),
                                color,
                            );
                        }
                        painter.rect_stroke(rect, 2.0, (1.0, color));
                    }
                }
            }

        // If there is no current map
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

                ui.scope(|ui| {
                    ui.set_enabled(false);
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
                        let map_meta = &params
                            .map_assets
                            .get(&map_handle.get_bevy_handle())
                            .unwrap();
                        if ui.button(&map_meta.name).clicked() {
                            params.session_manager.start(GameSessionInfo {
                                meta: params.core_meta.0.clone(),
                                map_meta: (*map_meta).clone(),
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
