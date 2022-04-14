use std::any::TypeId;
use std::path::Path;

use crate::{
    exit_to_main_menu,
    map::{CRAB_TEXTURE_ID, FISH_SCHOOL_ICON_TEXTURE_ID},
    quit_to_desktop, Resources,
};

mod camera;

pub use camera::EditorCamera;

pub mod gui;

use gui::{
    toggle_editor_menu,
    toolbars::{
        LayerListElement, ObjectListElement, TilesetDetailsElement, TilesetListElement,
        ToolSelectorElement, Toolbar, ToolbarPosition,
    },
    CreateLayerWindow, CreateObjectWindow, CreateTilesetWindow, EditorGui, TilesetPropertiesWindow,
};

mod actions;

use actions::{
    CreateLayerAction, CreateObjectAction, CreateTilesetAction, DeleteLayerAction,
    DeleteObjectAction, DeleteTilesetAction, EditorAction, PlaceTileAction, RemoveTileAction,
    SetLayerDrawOrderIndexAction, UndoableAction, UpdateTilesetAction,
};

mod input;

mod history;
mod tools;

pub use tools::{
    add_tool_instance, get_tool_instance, get_tool_instance_of_id, EraserTool, ObjectPlacementTool,
    TilePlacementTool, DEFAULT_TOOL_ICON_TEXTURE_ID,
};

use history::EditorHistory;
pub use input::EditorInputScheme;

use crate::editor::actions::{
    CreateSpawnPointAction, DeleteSpawnPointAction, ImportAction, MoveSpawnPointAction,
    UpdateBackgroundAction, UpdateLayerAction, UpdateObjectAction, UpdateTileAttributesAction,
};
use crate::editor::gui::windows::{
    BackgroundPropertiesWindow, CreateMapWindow, ImportWindow, LoadMapWindow,
    ObjectPropertiesWindow, SaveMapWindow, TilePropertiesWindow,
};
use crate::editor::input::{collect_editor_input, EditorInput};
use crate::editor::tools::SpawnPointPlacementTool;
use crate::gui::SELECTION_HIGHLIGHT_COLOR;
use crate::map::{MapObject, MapObjectKind};
use crate::player::IDLE_ANIMATION_ID;
use macroquad::{
    color,
    experimental::{
        collections::storage,
        scene::{Node, RefMut},
    },
    prelude::*,
};

use core::text::{draw_aligned_text, HorizontalAlignment, VerticalAlignment};

use super::map::{Map, MapLayerKind};
use crate::resources::{
    map_name_to_filename, MapResource, MAP_EXPORTS_DEFAULT_DIR, MAP_EXPORTS_EXTENSION,
};

#[derive(Debug, Clone)]
pub struct EditorContext {
    pub selected_tool: Option<TypeId>,
    pub selected_layer: Option<String>,
    pub selected_tileset: Option<String>,
    pub selected_tile: Option<u32>,
    pub selected_object: Option<usize>,
    pub input_scheme: EditorInputScheme,
    pub cursor_position: Vec2,
    pub is_user_map: bool,
    pub is_tiled_map: bool,
    pub should_snap_to_grid: bool,
}

impl Default for EditorContext {
    fn default() -> Self {
        EditorContext {
            selected_tool: None,
            selected_layer: None,
            selected_tileset: None,
            selected_tile: None,
            selected_object: None,
            input_scheme: EditorInputScheme::Mouse,
            cursor_position: Vec2::ZERO,
            is_user_map: false,
            is_tiled_map: false,
            should_snap_to_grid: false,
        }
    }
}

#[derive(Debug, Clone)]
enum DraggedObject {
    MapObject {
        id: String,
        kind: MapObjectKind,
        index: usize,
        layer_id: String,
        click_offset: Vec2,
    },
    SpawnPoint {
        index: usize,
        click_offset: Vec2,
    },
}

const SPAWN_POINT_COLLIDER_WIDTH: f32 = 38.0;
const SPAWN_POINT_COLLIDER_HEIGHT: f32 = 49.0;

pub struct Editor {
    map_resource: MapResource,

    selected_tool: Option<TypeId>,
    selected_layer: Option<String>,
    selected_tileset: Option<String>,
    // Selected tile in tileset
    selected_tile: Option<u32>,
    selected_object: Option<usize>,
    selected_spawn_point: Option<usize>,

    // Selected tile in map
    selected_map_tile_index: Option<usize>,

    input_scheme: EditorInputScheme,
    previous_cursor_position: Vec2,
    cursor_position: Vec2,
    history: EditorHistory,

    previous_input: EditorInput,
    input: EditorInput,
    mouse_movement: Vec2,

    info_message: Option<String>,

    dragged_object: Option<DraggedObject>,

    info_message_timer: f32,
    double_click_timer: f32,

    should_draw_grid: bool,
    should_snap_to_grid: bool,
    is_parallax_disabled: bool,
}

impl Editor {
    const CAMERA_PAN_THRESHOLD: f32 = 0.025;

    const CAMERA_PAN_SPEED: f32 = 5.0;
    const CAMERA_ZOOM_STEP: f32 = 0.1;
    const CAMERA_ZOOM_MIN: f32 = 0.1;
    const CAMERA_ZOOM_MAX: f32 = 2.5;

    const CURSOR_MOVE_SPEED: f32 = 5.0;

    const OBJECT_SELECTION_RECT_SIZE: f32 = 75.0;
    const OBJECT_SELECTION_RECT_PADDING: f32 = 8.0;

    const GRID_LINE_WIDTH: f32 = 1.0;
    const GRID_COLOR: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.25,
    };

    const DOUBLE_CLICK_THRESHOLD: f32 = 0.25;

    const MESSAGE_TIMEOUT: f32 = 2.5;

    pub fn new(input_scheme: EditorInputScheme, map_resource: MapResource) -> Self {
        add_tool_instance(TilePlacementTool::new());
        add_tool_instance(ObjectPlacementTool::new());
        add_tool_instance(SpawnPointPlacementTool::new());
        add_tool_instance(EraserTool::new());

        let selected_tool = None;

        let selected_layer = map_resource.map.draw_order.first().cloned();

        let cursor_position = match input_scheme {
            EditorInputScheme::Mouse => mouse_position().into(),
            EditorInputScheme::Gamepad(..) => vec2(screen_width() / 2.0, screen_height() / 2.0),
        };

        let tool_selector_element = ToolSelectorElement::new()
            .with_tool::<TilePlacementTool>()
            .with_tool::<ObjectPlacementTool>()
            .with_tool::<SpawnPointPlacementTool>()
            .with_tool::<EraserTool>();

        let left_toolbar = Toolbar::new(ToolbarPosition::Left, EditorGui::LEFT_TOOLBAR_WIDTH)
            .with_element(
                EditorGui::TOOL_SELECTOR_HEIGHT_FACTOR,
                tool_selector_element,
            );

        let right_toolbar = Toolbar::new(ToolbarPosition::Right, EditorGui::RIGHT_TOOLBAR_WIDTH)
            .with_element(EditorGui::LAYER_LIST_HEIGHT_FACTOR, LayerListElement::new())
            .with_element(
                EditorGui::TILESET_LIST_HEIGHT_FACTOR,
                TilesetListElement::new(),
            )
            .with_element(
                EditorGui::TILESET_DETAILS_HEIGHT_FACTOR,
                TilesetDetailsElement::new(),
            )
            .with_element(
                EditorGui::OBJECT_LIST_HEIGHT_FACTOR,
                ObjectListElement::new(),
            );

        let gui = EditorGui::new()
            .with_toolbar(left_toolbar)
            .with_toolbar(right_toolbar);

        storage::store(gui);

        Editor {
            map_resource,
            selected_tool,
            selected_layer,
            selected_tileset: None,
            selected_tile: None,
            selected_object: None,
            selected_spawn_point: None,

            selected_map_tile_index: None,

            input_scheme,
            previous_cursor_position: cursor_position,
            cursor_position,
            history: EditorHistory::new(),

            previous_input: EditorInput::default(),
            input: EditorInput::default(),
            mouse_movement: Vec2::ZERO,

            info_message: None,

            dragged_object: None,

            info_message_timer: 0.0,
            double_click_timer: Self::DOUBLE_CLICK_THRESHOLD,

            should_draw_grid: true,
            should_snap_to_grid: false,
            is_parallax_disabled: false,
        }
    }

    #[allow(dead_code)]
    fn get_selected_tile(&self) -> Option<(String, u32)> {
        if let Some(tileset_id) = self.selected_tileset.clone() {
            if let Some(tile_id) = self.selected_tile {
                let selected = (tileset_id, tile_id);
                return Some(selected);
            }
        }

        None
    }

    fn get_map(&self) -> &Map {
        &self.map_resource.map
    }

    #[allow(dead_code)]
    fn get_map_mut(&mut self) -> &mut Map {
        &mut self.map_resource.map
    }

    fn get_context(&self) -> EditorContext {
        EditorContext {
            selected_tool: self.selected_tool,
            selected_layer: self.selected_layer.clone(),
            selected_tileset: self.selected_tileset.clone(),
            selected_tile: self.selected_tile,
            selected_object: self.selected_object,
            input_scheme: self.input_scheme,
            cursor_position: self.cursor_position,
            is_user_map: self.map_resource.meta.is_user_map,
            is_tiled_map: self.map_resource.meta.is_tiled_map,
            should_snap_to_grid: self.should_snap_to_grid,
        }
    }

    fn update_context(&mut self) {
        if let Some(layer_id) = &self.selected_layer {
            if !self.get_map().draw_order.contains(layer_id) {
                self.selected_layer = None;
            }
        } else if let Some(layer_id) = self.get_map().draw_order.first() {
            self.selected_layer = Some(layer_id.clone());
        }

        if let Some(layer_id) = &self.selected_layer {
            let layer = self.get_map().layers.get(layer_id).unwrap();

            match layer.kind {
                MapLayerKind::TileLayer => {
                    self.selected_object = None;
                }
                MapLayerKind::ObjectLayer => {
                    self.selected_tileset = None;
                    self.selected_tile = None;
                }
            }
        }

        if let Some(tileset_id) = &self.selected_tileset {
            if let Some(tileset) = self.get_map().tilesets.get(tileset_id) {
                if let Some(tile_id) = self.selected_tile {
                    if tile_id >= tileset.tile_cnt {
                        self.selected_tile = None;
                    }
                }
            } else {
                self.selected_tileset = None;
                self.selected_tile = None;
            }
        }

        if let Some(tool_id) = &self.selected_tool {
            let tool = get_tool_instance_of_id(tool_id);
            let ctx = self.get_context();
            if !tool.is_available(self.get_map(), &ctx) {
                self.selected_tool = None;
            }
        }
    }

    fn clear_context(&mut self) {
        self.selected_tool = None;
        self.selected_layer = None;
        self.selected_tileset = None;
        self.selected_tile = None;
        self.selected_object = None;
    }

    fn select_tileset(&mut self, tileset_id: &str, tile_id: Option<u32>) {
        if let Some(tileset) = self.map_resource.map.tilesets.get(tileset_id) {
            self.selected_tileset = Some(tileset_id.to_string());

            if let Some(tile_id) = tile_id {
                if tile_id < tileset.first_tile_id + tileset.tile_cnt {
                    self.selected_tile = Some(tile_id);
                    return;
                }
            }

            self.selected_tile = Some(tileset.first_tile_id);
        }
    }

    // This applies an `EditorAction`. This is to be used, exclusively, in stead of, for example,
    // applying `UndoableActions` directly on the `History` of `Editor`.
    fn apply_action(&mut self, action: EditorAction) {
        //println!("Action: {:?}", action);

        let mut res = Ok(());

        match action {
            EditorAction::Batch(actions) => {
                for action in actions {
                    self.apply_action(action)
                }
            }
            EditorAction::Undo => {
                res = self.history.undo(&mut self.map_resource.map);
            }
            EditorAction::Redo => {
                res = self.history.redo(&mut self.map_resource.map);
            }
            EditorAction::SelectTool(id) => {
                self.selected_tool = id;
            }
            EditorAction::UpdateBackground { color, layers } => {
                let action = UpdateBackgroundAction::new(color, layers);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::OpenBackgroundPropertiesWindow => {
                let map = &self.map_resource.map;

                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(BackgroundPropertiesWindow::new(
                    map.background_color,
                    map.background_layers.clone(),
                ));
            }
            EditorAction::OpenCreateLayerWindow => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(CreateLayerWindow::new());
            }
            EditorAction::OpenCreateTilesetWindow => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(CreateTilesetWindow::new());
            }
            EditorAction::OpenTilesetPropertiesWindow(tileset_id) => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(TilesetPropertiesWindow::new(&tileset_id));
            }
            EditorAction::OpenCreateObjectWindow { position, layer_id } => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(CreateObjectWindow::new(position, layer_id))
            }
            EditorAction::OpenObjectPropertiesWindow { layer_id, index } => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(ObjectPropertiesWindow::new(layer_id, index))
            }
            EditorAction::OpenTilePropertiesWindow { layer_id, index } => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(TilePropertiesWindow::new(layer_id, index))
            }
            EditorAction::CloseWindow(id) => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.remove_window_id(id);
            }
            EditorAction::SelectTile { id, tileset_id } => {
                self.select_tileset(&tileset_id, Some(id));
            }
            EditorAction::UpdateTileAttributes {
                index,
                layer_id,
                attributes,
            } => {
                let action = UpdateTileAttributesAction::new(index, layer_id, attributes);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::SelectLayer(id) => {
                if self.get_map().layers.contains_key(&id) {
                    self.selected_layer = Some(id);
                }
            }
            EditorAction::SetLayerDrawOrderIndex { id, index } => {
                let action = SetLayerDrawOrderIndexAction::new(id, index);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::CreateLayer {
                id,
                kind,
                has_collision,
                index,
            } => {
                let action = CreateLayerAction::new(id, kind, has_collision, index);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::DeleteLayer(id) => {
                let action = DeleteLayerAction::new(id);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::UpdateLayer { id, is_visible } => {
                let action = UpdateLayerAction::new(id, is_visible);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::SelectTileset(id) => {
                self.select_tileset(&id, None);
            }
            EditorAction::CreateTileset { id, texture_id } => {
                let action = CreateTilesetAction::new(id, texture_id);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::DeleteTileset(id) => {
                let action = DeleteTilesetAction::new(id);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::UpdateTileset {
                id,
                texture_id,
                autotile_mask,
            } => {
                let action = UpdateTilesetAction::new(id, texture_id, autotile_mask);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::SelectObject { index, layer_id } => {
                self.selected_layer = Some(layer_id);
                self.selected_object = Some(index);
            }
            EditorAction::CreateObject {
                id,
                kind,
                position,
                layer_id,
            } => {
                let action = CreateObjectAction::new(id, kind, position, layer_id);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::DeleteObject { index, layer_id } => {
                let action = DeleteObjectAction::new(index, layer_id);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::UpdateObject {
                layer_id,
                index,
                id,
                kind,
                position,
            } => {
                let action = UpdateObjectAction::new(layer_id, index, id, kind, position);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::CreateSpawnPoint(position) => {
                let action = CreateSpawnPointAction::new(position);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::DeleteSpawnPoint(index) => {
                let action = DeleteSpawnPointAction::new(index);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::MoveSpawnPoint { index, position } => {
                let action = MoveSpawnPointAction::new(index, position);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::PlaceTile {
                id,
                layer_id,
                tileset_id,
                coords,
            } => {
                let action = PlaceTileAction::new(id, layer_id, tileset_id, coords);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::RemoveTile { layer_id, coords } => {
                let action = RemoveTileAction::new(layer_id, coords);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::OpenImportWindow(map_index) => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(ImportWindow::new(map_index));
            }
            EditorAction::Import {
                tilesets,
                background_color,
                background_layers,
            } => {
                let action = ImportAction::new(tilesets, background_color, background_layers);
                res = self
                    .history
                    .apply(Box::new(action), &mut self.map_resource.map);
            }
            EditorAction::CreateMap {
                name,
                description,
                grid_size,
                tile_size,
            } => {
                let resources = storage::get::<Resources>();
                let res = resources.create_map(&name, description.as_deref(), tile_size, grid_size);
                match res {
                    Err(err) => println!("Create Map: {}", err),
                    Ok(map_resource) => {
                        self.map_resource = map_resource;
                        self.history.clear();
                        self.clear_context();
                    }
                }
            }
            EditorAction::OpenCreateMapWindow => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(CreateMapWindow::new());
            }
            EditorAction::OpenMap(index) => {
                let resources = storage::get::<Resources>();
                let map_resource = resources.maps.get(index).cloned().unwrap();

                self.map_resource = map_resource;
                self.history.clear();
                self.clear_context();
            }
            EditorAction::OpenLoadMapWindow => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(LoadMapWindow::new());
            }
            EditorAction::SaveMap(name) => {
                let mut map_resource = self.map_resource.clone();

                if let Some(name) = name {
                    let path = Path::new(MAP_EXPORTS_DEFAULT_DIR)
                        .join(map_name_to_filename(&name))
                        .with_extension(MAP_EXPORTS_EXTENSION);

                    map_resource.meta.name = name;
                    map_resource.meta.path = path.to_string_lossy().to_string();
                }

                map_resource.meta.is_user_map = true;
                map_resource.meta.is_tiled_map = false;

                let mut resources = storage::get_mut::<Resources>();
                if resources.save_map(&map_resource).is_ok() {
                    self.map_resource = map_resource;
                }
            }
            EditorAction::OpenSaveMapWindow => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(SaveMapWindow::new(&self.map_resource.meta.name));
            }
            EditorAction::DeleteMap(index) => {
                let mut resources = storage::get_mut::<Resources>();
                resources.delete_map(index).unwrap();
            }
            EditorAction::ExitToMainMenu => {
                exit_to_main_menu();
            }
            EditorAction::QuitToDesktop => {
                quit_to_desktop();
            }
        }

        if let Err(err) = res {
            panic!("Error: {}", err);
        }

        self.update_context();
    }
}

impl Node for Editor {
    fn update(mut node: RefMut<Self>) {
        node.update_context();

        if node.input_scheme == EditorInputScheme::Mouse {
            node.previous_cursor_position = node.cursor_position;
            node.cursor_position = mouse_position().into();
        }

        let dt = get_frame_time();

        node.previous_input = node.input;
        node.input = collect_editor_input(node.input_scheme);

        {
            let movement = node.cursor_position - node.previous_cursor_position;
            node.mouse_movement += movement;
        }

        if node.info_message.is_some() {
            node.info_message_timer += dt;

            if node.info_message_timer >= Self::MESSAGE_TIMEOUT {
                node.info_message = None;
                node.info_message_timer = 0.0;
            }
        }

        if node.input.save {
            let action = if node.map_resource.meta.is_user_map {
                EditorAction::SaveMap(None)
            } else {
                EditorAction::OpenSaveMapWindow
            };

            node.apply_action(action);
        }

        if node.input.save_as {
            let action = EditorAction::OpenSaveMapWindow;
            node.apply_action(action);
        }

        if node.input.load {
            let action = EditorAction::OpenLoadMapWindow;
            node.apply_action(action);
        }

        if !node.input.action && node.double_click_timer < Self::DOUBLE_CLICK_THRESHOLD {
            node.double_click_timer =
                (node.double_click_timer + dt).clamp(0.0, Self::DOUBLE_CLICK_THRESHOLD);
        }

        if node.input.toggle_menu {
            toggle_editor_menu(&node.get_context());
        }

        if node.input.toggle_draw_grid {
            node.should_draw_grid = !node.should_draw_grid;

            node.info_message = {
                let state = if node.should_draw_grid { "ON" } else { "OFF" };

                Some(format!("Draw grid: {}", state))
            }
        }

        if node.input.toggle_snap_to_grid {
            node.should_snap_to_grid = !node.should_snap_to_grid;

            node.info_message = {
                let state = if node.should_snap_to_grid {
                    "ON"
                } else {
                    "OFF"
                };

                Some(format!("Snap to grid: {}", state))
            }
        }

        if node.input.toggle_disable_parallax {
            node.is_parallax_disabled = !node.is_parallax_disabled;

            node.info_message = {
                let state = if node.is_parallax_disabled {
                    "OFF"
                } else {
                    "ON"
                };

                Some(format!("Parallax: {}", state))
            }
        }

        if node.input.undo {
            node.apply_action(EditorAction::Undo);
        } else if node.input.redo {
            node.apply_action(EditorAction::Redo);
        }

        let cursor_world_position = scene::find_node_by_type::<EditorCamera>()
            .unwrap()
            .to_world_space(node.cursor_position);

        let (is_cursor_over_gui, is_cursor_over_context_menu) = {
            let gui = storage::get::<EditorGui>();
            let is_over_gui = gui.contains(node.cursor_position);
            let mut is_over_context_menu = false;
            if is_over_gui && gui.context_menu_contains(node.cursor_position) {
                is_over_context_menu = true;
            }

            (is_over_gui, is_over_context_menu)
        };

        if let Some(id) = &node.selected_tool {
            let res = {
                let tool = get_tool_instance_of_id(id);
                let map = node.get_map();
                let ctx = node.get_context();

                tool.update(map, &ctx)
            };

            if let Some(action) = res {
                node.apply_action(action);
            }
        }

        if node.input.action {
            if !is_cursor_over_context_menu {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.close_context_menu();
            }

            if !is_cursor_over_gui {
                if let Some(id) = &node.selected_tool {
                    let ctx = node.get_context();
                    let tool = get_tool_instance_of_id(id);
                    let params = tool.get_params();
                    if !node.previous_input.action || params.is_continuous {
                        if let Some(action) = tool.get_action(node.get_map(), &ctx) {
                            node.apply_action(action);
                        }
                    }
                } else if node.previous_input.action {
                    if node.cursor_position == node.previous_cursor_position
                        && node.dragged_object.is_none()
                    {
                        if let Some(index) = node.selected_object {
                            let layer_id = node.selected_layer.clone().unwrap();
                            let layer = node.get_map().layers.get(&layer_id).unwrap();

                            let object = layer.objects.get(index).unwrap();
                            let position = scene::find_node_by_type::<EditorCamera>()
                                .unwrap()
                                .to_screen_space(object.position);

                            let size = get_object_size(object);
                            let rect = Rect::new(position.x, position.y, size.x, size.y);

                            if rect.contains(node.cursor_position) {
                                let click_offset = node.cursor_position - position;

                                node.dragged_object = Some(DraggedObject::MapObject {
                                    id: object.id.clone(),
                                    kind: object.kind,
                                    index,
                                    layer_id,
                                    click_offset,
                                })
                            }
                        } else if let Some(index) = node.selected_spawn_point {
                            let spawn_point = node.get_map().spawn_points[index];

                            let position = scene::find_node_by_type::<EditorCamera>()
                                .unwrap()
                                .to_screen_space(spawn_point);

                            let rect = Rect::new(
                                position.x,
                                position.y,
                                SPAWN_POINT_COLLIDER_WIDTH,
                                SPAWN_POINT_COLLIDER_HEIGHT,
                            );

                            if rect.contains(node.cursor_position) {
                                let click_offset = node.cursor_position - position;

                                node.dragged_object = Some(DraggedObject::SpawnPoint {
                                    index,
                                    click_offset,
                                })
                            }
                        }
                    }
                } else {
                    let mut is_double_click = false;
                    let mut is_selecting_object = false;
                    let mut is_selecting_spawn_point = false;
                    let mut is_selecting_tile = false;

                    if node.double_click_timer < Self::DOUBLE_CLICK_THRESHOLD {
                        node.double_click_timer = Self::DOUBLE_CLICK_THRESHOLD;
                        is_double_click = true;
                    } else {
                        node.double_click_timer = 0.0;
                    }

                    let mut layer_ids = node
                        .map_resource
                        .map
                        .layers
                        .keys()
                        .cloned()
                        .collect::<Vec<String>>();

                    if let Some(selected_layer_id) = &node.selected_layer {
                        let res = layer_ids.iter().enumerate().find_map(|(i, layer_id)| {
                            if layer_id == selected_layer_id {
                                Some(i)
                            } else {
                                None
                            }
                        });

                        if let Some(i) = res {
                            layer_ids.remove(i);
                            layer_ids.insert(0, selected_layer_id.clone());
                        }
                    }

                    let mut object_index = None;
                    let mut layer_id = None;

                    'layers: for id in &layer_ids {
                        let layer = node.map_resource.map.layers.get(id).unwrap();
                        if layer.kind == MapLayerKind::ObjectLayer {
                            for (i, object) in layer.objects.iter().enumerate() {
                                let size = get_object_size(object);
                                let position = object.position + node.map_resource.map.world_offset;

                                let rect = Rect::new(position.x, position.y, size.x, size.y);

                                if rect.contains(cursor_world_position) {
                                    object_index = Some(i);
                                    layer_id = Some(id.clone());

                                    break 'layers;
                                }
                            }
                        }
                    }

                    if let Some(i) = object_index {
                        let mut should_select = true;

                        if let Some(current_index) = node.selected_object {
                            if current_index == i {
                                should_select = false;

                                if is_double_click {
                                    let layer_id = layer_id.clone().unwrap();

                                    let action = EditorAction::OpenObjectPropertiesWindow {
                                        layer_id,
                                        index: i,
                                    };

                                    node.apply_action(action);
                                } else {
                                    node.selected_object = None;
                                }
                            }
                        }

                        if should_select {
                            is_selecting_object = true;

                            let layer_id = layer_id.unwrap();

                            let action = EditorAction::SelectObject { index: i, layer_id };

                            node.apply_action(action);
                        }
                    } else {
                        for (i, spawn_point) in node.get_map().spawn_points.iter().enumerate() {
                            let position = scene::find_node_by_type::<EditorCamera>()
                                .unwrap()
                                .to_screen_space(*spawn_point);

                            let rect = Rect::new(
                                position.x,
                                position.y,
                                SPAWN_POINT_COLLIDER_WIDTH,
                                SPAWN_POINT_COLLIDER_HEIGHT,
                            );

                            if rect.contains(node.cursor_position) {
                                is_selecting_spawn_point = true;

                                let mut should_select = true;

                                if let Some(index) = node.selected_spawn_point {
                                    if index == i {
                                        node.selected_spawn_point = None;
                                        should_select = false;
                                    }
                                }

                                if should_select {
                                    node.selected_spawn_point = Some(i);
                                }

                                break;
                            }
                        }

                        if !is_selecting_spawn_point {
                            let mut tile_index = None;

                            'tile_layers: for id in &layer_ids {
                                let layer = node.get_map().layers.get(id).unwrap();
                                if layer.kind == MapLayerKind::TileLayer {
                                    let world_offset = node.get_map().world_offset;
                                    let tile_size = node.get_map().tile_size;

                                    for (x, y, tile) in node.map_resource.map.get_tiles(id, None) {
                                        if tile.is_some() {
                                            let rect = Rect::new(
                                                world_offset.x + (x as f32 * tile_size.x),
                                                world_offset.y + (y as f32 * tile_size.y),
                                                tile_size.x,
                                                tile_size.y,
                                            );
                                            if rect.contains(cursor_world_position) {
                                                let i = node.get_map().to_index(uvec2(x, y));
                                                tile_index = Some(i);
                                                layer_id = Some(id.clone());

                                                break 'tile_layers;
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some(tile_index) = tile_index {
                                let mut should_select = true;

                                if let Some(selected_tile_index) = node.selected_map_tile_index {
                                    if selected_tile_index == tile_index
                                        && layer_id.as_ref().unwrap()
                                            == node.selected_layer.as_ref().unwrap()
                                    {
                                        should_select = false;

                                        if is_double_click {
                                            let layer_id = layer_id.clone().unwrap();

                                            let action = EditorAction::OpenTilePropertiesWindow {
                                                layer_id,
                                                index: tile_index,
                                            };

                                            node.apply_action(action);
                                        } else {
                                            node.selected_map_tile_index = None;
                                        }
                                    }
                                }

                                if should_select {
                                    is_selecting_tile = true;
                                    node.selected_map_tile_index = Some(tile_index);
                                    node.selected_layer = layer_id;
                                }
                            }
                        }
                    }

                    if !is_selecting_tile && !is_selecting_object && !is_selecting_spawn_point {
                        node.selected_map_tile_index = None;
                        node.selected_object = None;
                        node.selected_spawn_point = None;
                    }
                }
            }
        } else if let Some(dragged_object) = node.dragged_object.take() {
            let map = node.get_map();

            let cursor_world_position = scene::find_node_by_type::<EditorCamera>()
                .unwrap()
                .to_world_space(node.cursor_position);

            let mut position = (cursor_world_position).clamp(
                map.world_offset,
                map.world_offset + (map.grid_size.as_f32() * map.tile_size),
            );

            if node.should_snap_to_grid {
                let coords = map.to_coords(position);
                position = map.to_position(coords);
            }

            match dragged_object {
                DraggedObject::MapObject {
                    id,
                    kind,
                    index,
                    layer_id,
                    click_offset,
                } => {
                    let position = position - click_offset;

                    let action = EditorAction::UpdateObject {
                        id,
                        kind,
                        index,
                        layer_id,
                        position,
                    };

                    node.apply_action(action);
                }
                DraggedObject::SpawnPoint {
                    index,
                    click_offset,
                } => {
                    let position = position - click_offset;

                    let action = EditorAction::MoveSpawnPoint { index, position };

                    node.apply_action(action);
                }
            }
        }

        if node.input.delete {
            if let Some(index) = node.selected_object.take() {
                let layer_id = node.selected_layer.clone().unwrap();

                let action = EditorAction::DeleteObject { index, layer_id };

                node.apply_action(action);
            } else if let Some(index) = node.selected_map_tile_index.take() {
                let layer_id = node.selected_layer.clone().unwrap();
                let coords = {
                    let grid_size = node.get_map().grid_size;
                    uvec2(index as u32 % grid_size.x, index as u32 / grid_size.x)
                };

                let action = EditorAction::RemoveTile { coords, layer_id };

                node.apply_action(action);
            } else if let Some(index) = node.selected_spawn_point.take() {
                let action = EditorAction::DeleteSpawnPoint(index);
                node.apply_action(action);
            }
        }

        if node.input.context_menu {
            let mut gui = storage::get_mut::<EditorGui>();
            gui.open_context_menu(
                node.cursor_position,
                &node.map_resource.map,
                node.get_context(),
            );
        }
    }

    fn fixed_update(mut node: RefMut<Self>) {
        if let EditorInputScheme::Gamepad { .. } = node.input_scheme {
            node.previous_cursor_position = node.cursor_position;
            let movement = node.input.cursor_move_direction * Self::CURSOR_MOVE_SPEED;
            node.cursor_position += movement;
        }

        let is_cursor_over_map = {
            let gui = storage::get::<EditorGui>();
            !gui.contains(node.cursor_position)
        };

        let screen_size = vec2(screen_width(), screen_height());

        let threshold = screen_size * Self::CAMERA_PAN_THRESHOLD;

        let mut pan_direction = node.input.camera_move_direction;

        if node.cursor_position.x <= threshold.x {
            pan_direction.x = -1.0;
        } else if node.cursor_position.x >= screen_size.x - threshold.x {
            pan_direction.x = 1.0;
        }

        if node.cursor_position.y <= threshold.y {
            pan_direction.y = -1.0;
        } else if node.cursor_position.y >= screen_size.y - threshold.y {
            pan_direction.y = 1.0;
        }

        let mut movement = pan_direction * Self::CAMERA_PAN_SPEED;

        let mut camera = scene::find_node_by_type::<EditorCamera>().unwrap();

        if movement == Vec2::ZERO && node.input.camera_mouse_move {
            movement = -node.mouse_movement / camera.scale;
        }

        node.mouse_movement = Vec2::ZERO;

        camera.position = (camera.position + movement).clamp(Vec2::ZERO, node.get_map().get_size());

        if is_cursor_over_map {
            camera.scale = (camera.scale + node.input.camera_zoom * Self::CAMERA_ZOOM_STEP)
                .clamp(Self::CAMERA_ZOOM_MIN, Self::CAMERA_ZOOM_MAX);
        }
    }

    fn draw(mut node: RefMut<Self>) {
        {
            let map = node.get_map();
            map.draw_background(None, node.is_parallax_disabled);
            map.draw(None, false);
        }

        if node.should_draw_grid {
            let map = node.get_map();
            let map_size = map.grid_size.as_f32() * map.tile_size;

            draw_rectangle_lines(
                map.world_offset.x,
                map.world_offset.y,
                map_size.x,
                map_size.y,
                Self::GRID_LINE_WIDTH,
                Self::GRID_COLOR,
            );

            for x in 0..map.grid_size.x {
                let begin = vec2(
                    map.world_offset.x + (x as f32 * map.tile_size.x),
                    map.world_offset.y,
                );

                let end = vec2(
                    begin.x,
                    begin.y + (map.grid_size.y as f32 * map.tile_size.y),
                );

                draw_line(
                    begin.x,
                    begin.y,
                    end.x,
                    end.y,
                    Self::GRID_LINE_WIDTH,
                    Self::GRID_COLOR,
                )
            }

            for y in 0..map.grid_size.y {
                let begin = vec2(
                    map.world_offset.x,
                    map.world_offset.y + (y as f32 * map.tile_size.y),
                );

                let end = vec2(
                    begin.x + (map.grid_size.x as f32 * map.tile_size.x),
                    begin.y,
                );

                draw_line(
                    begin.x,
                    begin.y,
                    end.x,
                    end.y,
                    Self::GRID_LINE_WIDTH,
                    Self::GRID_COLOR,
                )
            }
        }

        {
            let resources = storage::get::<Resources>();

            for (i, spawn_point) in node.get_map().spawn_points.iter().enumerate() {
                let mut is_selected = false;

                let mut position = *spawn_point;

                if let Some(DraggedObject::SpawnPoint {
                    index,
                    click_offset,
                }) = node.dragged_object.clone()
                {
                    if index == i {
                        let map = node.get_map();

                        let cursor_world_position = scene::find_node_by_type::<EditorCamera>()
                            .unwrap()
                            .to_world_space(node.cursor_position - click_offset);

                        position = (cursor_world_position).clamp(
                            map.world_offset,
                            map.world_offset + (map.grid_size.as_f32() * map.tile_size),
                        );

                        if node.should_snap_to_grid {
                            let coords = map.to_coords(position);
                            position = map.to_position(coords);
                        }
                    }
                }

                if let Some(index) = node.selected_spawn_point {
                    is_selected = index == i;
                }

                let texture_res = resources.textures.get("spawn_point_icon").unwrap();

                let frame_size = texture_res.meta.frame_size.unwrap_or_else(|| {
                    vec2(texture_res.texture.width(), texture_res.texture.height())
                });

                let source_rect = Rect::new(0.0, 0.0, frame_size.x, frame_size.y);

                draw_texture_ex(
                    texture_res.texture,
                    position.x,
                    position.y,
                    color::WHITE,
                    DrawTextureParams {
                        dest_size: Some(frame_size),
                        source: Some(source_rect),
                        ..Default::default()
                    },
                );

                if is_selected {
                    draw_rectangle_lines(
                        position.x,
                        position.y,
                        SPAWN_POINT_COLLIDER_WIDTH,
                        SPAWN_POINT_COLLIDER_HEIGHT,
                        4.0,
                        SELECTION_HIGHLIGHT_COLOR,
                    )
                }
            }

            let len = node.get_map().draw_order.len();
            for i in 0..len {
                let i = len as i32 - i as i32 - 1;
                if i >= 0 {
                    let layer_id = node.get_map().draw_order.get(i as usize).unwrap();
                    let layer = node.get_map().layers.get(layer_id).unwrap();

                    if layer.is_visible && layer.kind == MapLayerKind::ObjectLayer {
                        for (i, object) in layer.objects.iter().enumerate() {
                            let mut label = None;

                            let mut is_selected = false;
                            if let Some(layer_id) = &node.selected_layer {
                                if let Some(index) = node.selected_object {
                                    is_selected = *layer_id == layer.id && index == i;
                                }
                            }

                            let mut object_position =
                                node.map_resource.map.world_offset + object.position;

                            if let Some(DraggedObject::MapObject {
                                layer_id,
                                index,
                                click_offset,
                                ..
                            }) = node.dragged_object.clone()
                            {
                                if layer.id == layer_id && index == i {
                                    let map = node.get_map();

                                    let cursor_world_position =
                                        scene::find_node_by_type::<EditorCamera>()
                                            .unwrap()
                                            .to_world_space(node.cursor_position - click_offset);

                                    object_position = (cursor_world_position).clamp(
                                        map.world_offset,
                                        map.world_offset + (map.grid_size.as_f32() * map.tile_size),
                                    );

                                    if node.should_snap_to_grid {
                                        let coords = map.to_coords(object_position);
                                        object_position = map.to_position(coords);
                                    }
                                }
                            }

                            match object.kind {
                                MapObjectKind::Item => {
                                    if let Some(meta) = resources.items.get(&object.id) {
                                        if let Some(texture_res) =
                                            resources.textures.get(&meta.sprite.texture_id)
                                        {
                                            let (texture, frame_size) =
                                                (texture_res.texture, texture_res.frame_size());

                                            let row = meta
                                                .sprite
                                                .animations
                                                .iter()
                                                .find(|&a| a.id == *IDLE_ANIMATION_ID)
                                                .map(|a| a.row)
                                                .unwrap_or_default();

                                            let position = object_position + meta.sprite.offset;

                                            let tint = meta.sprite.tint.unwrap_or(color::WHITE);

                                            let dest_size =
                                                meta.sprite.scale.map(|s| s * frame_size);

                                            let source = Some(Rect::new(
                                                0.0,
                                                row as f32 * frame_size.y,
                                                frame_size.x,
                                                frame_size.y,
                                            ));

                                            draw_texture_ex(
                                                texture,
                                                position.x,
                                                position.y,
                                                tint,
                                                DrawTextureParams {
                                                    dest_size,
                                                    source,
                                                    ..Default::default()
                                                },
                                            );
                                        } else {
                                            label = Some("INVALID TEXTURE ID".to_string());
                                        }
                                    } else {
                                        label = Some("INVALID OBJECT ID".to_string());
                                    }
                                }
                                MapObjectKind::Decoration => {
                                    if let Some(params) = resources.decoration.get(&object.id) {
                                        if let Some(texture_res) =
                                            resources.textures.get(&params.sprite.texture_id)
                                        {
                                            let position = object_position + params.sprite.offset;

                                            let tint = params.sprite.tint.unwrap_or(color::WHITE);

                                            let (texture, frame_size) =
                                                (texture_res.texture, texture_res.frame_size());

                                            let dest_size =
                                                params.sprite.scale.map(|s| s * frame_size);

                                            let source =
                                                params.sprite.animations.first().map(|a| {
                                                    Rect::new(
                                                        0.0,
                                                        a.row as f32 * frame_size.y,
                                                        frame_size.x,
                                                        frame_size.y,
                                                    )
                                                });

                                            draw_texture_ex(
                                                texture,
                                                position.x,
                                                position.y,
                                                tint,
                                                DrawTextureParams {
                                                    dest_size,
                                                    source,
                                                    ..Default::default()
                                                },
                                            );
                                        } else {
                                            label = Some("INVALID TEXTURE ID".to_string());
                                        }
                                    } else {
                                        label = Some("INVALID OBJECT ID".to_string());
                                    }
                                }
                                MapObjectKind::Environment => match object.id.as_str() {
                                    "sproinger" | "crab" | "fish_school" => {
                                        let texture_id = match object.id.as_str() {
                                            "sproinger" => "sproinger",
                                            "crab" => CRAB_TEXTURE_ID,
                                            "fish_school" => FISH_SCHOOL_ICON_TEXTURE_ID,
                                            _ => unreachable!(),
                                        };
                                        let texture_res =
                                            resources.textures.get(texture_id).unwrap();

                                        let frame_size =
                                            texture_res.meta.frame_size.unwrap_or_else(|| {
                                                vec2(
                                                    texture_res.texture.width(),
                                                    texture_res.texture.height(),
                                                )
                                            });

                                        let source_rect =
                                            Rect::new(0.0, 0.0, frame_size.x, frame_size.y);

                                        draw_texture_ex(
                                            texture_res.texture,
                                            object_position.x,
                                            object_position.y,
                                            color::WHITE,
                                            DrawTextureParams {
                                                dest_size: Some(frame_size),
                                                source: Some(source_rect),
                                                ..Default::default()
                                            },
                                        );
                                    }
                                    _ => {
                                        label = Some("INVALID OBJECT ID".to_string());
                                    }
                                },
                            }

                            let size = get_object_size(object);

                            if let Some(label) = &label {
                                let params = TextParams::default();

                                draw_text_ex(
                                    label,
                                    object_position.x,
                                    object_position.y + (size.y / 2.0)
                                        - Self::OBJECT_SELECTION_RECT_PADDING,
                                    params,
                                );
                            }

                            if is_selected {
                                draw_rectangle_lines(
                                    object_position.x - Self::OBJECT_SELECTION_RECT_PADDING,
                                    object_position.y - Self::OBJECT_SELECTION_RECT_PADDING,
                                    size.x,
                                    size.y,
                                    4.0,
                                    SELECTION_HIGHLIGHT_COLOR,
                                );
                            }
                        }
                    }
                }
            }
        }

        if let Some(tile_index) = node.selected_map_tile_index {
            let grid_size = node.get_map().grid_size;
            let tile_size = node.get_map().tile_size;

            let coords = uvec2(
                tile_index as u32 % grid_size.x,
                tile_index as u32 / grid_size.x,
            );
            let position = node.get_map().to_position(coords);

            draw_rectangle_lines(
                position.x,
                position.y,
                tile_size.x,
                tile_size.y,
                5.0,
                SELECTION_HIGHLIGHT_COLOR,
            )
        }

        if let Some(label) = &node.info_message {
            push_camera_state();
            set_default_camera();

            let label_position = vec2(screen_width() / 2.0, 16.0);

            draw_aligned_text(
                label,
                label_position,
                HorizontalAlignment::Center,
                VerticalAlignment::Bottom,
                TextParams {
                    ..Default::default()
                },
            );

            pop_camera_state();
        }

        let mut res = None;

        if let Some(tool_id) = &node.selected_tool {
            let tool = get_tool_instance_of_id(tool_id);
            let ctx = node.get_context();
            res = tool.draw_cursor(node.get_map(), &ctx);
        }

        {
            let ctx = node.get_context();
            let mut gui = storage::get_mut::<EditorGui>();
            if let Some(action) = gui.draw(node.get_map(), ctx) {
                res = Some(action);
            }
        }

        if let Some(action) = res {
            node.apply_action(action);
        }
    }
}

fn get_object_size(object: &MapObject) -> Vec2 {
    let mut res = None;

    let mut label = None;

    let resources = storage::get::<Resources>();

    match object.kind {
        MapObjectKind::Item => {
            if let Some(meta) = resources.items.get(&object.id) {
                if resources.textures.get(&meta.sprite.texture_id).is_some() {
                    res = Some(meta.collider_size);
                } else {
                    label = Some("INVALID TEXTURE ID".to_string());
                }
            } else {
                label = Some("INVALID OBJECT ID".to_string())
            }
        }
        MapObjectKind::Decoration => {
            if let Some(meta) = resources.decoration.get(&object.id) {
                if let Some(texture_res) = resources.textures.get(&meta.sprite.texture_id) {
                    res = Some(texture_res.frame_size());
                } else {
                    label = Some("INVALID TEXTURE ID".to_string());
                }
            } else {
                label = Some("INVALID OBJECT ID".to_string())
            }
        }
        MapObjectKind::Environment => match object.id.as_str() {
            "sproinger" => {
                let texture_res = resources.textures.get("sproinger").unwrap();
                res = texture_res.meta.frame_size;
            }
            "crab" => {
                let texture_res = resources.textures.get(CRAB_TEXTURE_ID).unwrap();
                res = texture_res.meta.frame_size;
            }
            "fish_school" => {
                let texture_res = resources.textures.get(FISH_SCHOOL_ICON_TEXTURE_ID).unwrap();
                res = texture_res.meta.frame_size;
            }
            _ => label = Some("INVALID OBJECT ID".to_string()),
        },
    }

    if let Some(label) = &label {
        let params = TextParams::default();
        let measure = measure_text(
            label,
            Some(params.font),
            params.font_size,
            params.font_scale,
        );
        res = Some(vec2(measure.width, measure.height));
    }

    res.unwrap_or_else(|| {
        vec2(
            Editor::OBJECT_SELECTION_RECT_SIZE,
            Editor::OBJECT_SELECTION_RECT_SIZE,
        )
    }) + (vec2(
        Editor::OBJECT_SELECTION_RECT_PADDING,
        Editor::OBJECT_SELECTION_RECT_PADDING,
    ) * 2.0)
}
