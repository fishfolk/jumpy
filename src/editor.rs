use std::any::TypeId;

mod camera;

pub use camera::EditorCamera;

pub mod gui;

use gui::{
    CreateLayerWindow, CreateObjectWindow, CreateTilesetWindow, EditorGui, GuiElement,
    TilesetPropertiesWindow,
    toolbars::{Toolbar, TilesetDetailsElement, ToolbarPosition, LayerListElement, TilesetListElement, ObjectListElement, ToolSelectorElement},
};

mod actions;

use actions::{
    CreateLayerAction, CreateObjectAction, CreateTilesetAction, DeleteLayerAction,
    DeleteObjectAction, DeleteTilesetAction, EditorAction, PlaceTileAction, RemoveTileAction,
    Result, SetLayerDrawOrderIndexAction, SetTilesetAutotileMaskAction, UndoableAction,
};

mod input;

mod history;
mod tools;

pub use tools::{
    add_tool_instance, get_tool_instance, get_tool_instance_of_id, EraserTool, PlacementTool, DEFAULT_TOOL_ICON_TEXTURE_ID,
};

use history::EditorHistory;
pub use input::EditorInputScheme;

use input::collect_editor_input;

use macroquad::{
    experimental::{
        collections::storage,
        scene::{Node, RefMut},
    },
    prelude::*,
};

use super::map::{
    Map,
    MapLayerKind,
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
}

impl Default for EditorContext {
    fn default() -> Self {
        EditorContext {
            selected_tool: None,
            selected_layer: None,
            selected_tileset: None,
            selected_tile: None,
            selected_object: None,
            input_scheme: EditorInputScheme::Keyboard,
            cursor_position: Vec2::ZERO,
        }
    }
}

pub struct Editor {
    map: Map,
    selected_tool: Option<TypeId>,
    selected_layer: Option<String>,
    selected_tileset: Option<String>,
    selected_tile: Option<u32>,
    selected_object: Option<usize>,
    input_scheme: EditorInputScheme,
    // This will hold the gamepad cursor position and be `None` if not using a gamepad.
    // Use the `get_cursor_position` method to get the actual cursor position, as that will return
    // the mouse cursor position, if no gamepad is used and this is set to `None`.
    cursor_position: Option<Vec2>,
    history: EditorHistory,
}

impl Editor {
    const CAMERA_PAN_THRESHOLD: f32 = 0.025;

    const CAMERA_PAN_SPEED: f32 = 5.0;
    const CAMERA_ZOOM_STEP: f32 = 0.05;
    const CAMERA_ZOOM_MIN: f32 = 0.5;
    const CAMERA_ZOOM_MAX: f32 = 1.5;

    const CURSOR_MOVE_SPEED: f32 = 5.0;

    pub fn new(input_scheme: EditorInputScheme, map: Map) -> Self {
        add_tool_instance(PlacementTool::new());
        add_tool_instance(EraserTool::new());

        let selected_tool = None;

        let selected_layer = map.draw_order.first().cloned();

        let cursor_position = match input_scheme {
            EditorInputScheme::Keyboard => None,
            EditorInputScheme::Gamepad(..) => {
                Some(vec2(screen_width() / 2.0, screen_height() / 2.0))
            }
        };

        let mut gui = EditorGui::new();

        let tool_selector_element = ToolSelectorElement::new()
            .with_tool::<PlacementTool>()
            .with_tool::<EraserTool>();

        let left_toolbar = Toolbar::new(ToolbarPosition::Left, EditorGui::LEFT_TOOLBAR_WIDTH)
            .with_element(
                EditorGui::TOOL_SELECTOR_HEIGHT_FACTOR,
                tool_selector_element);

        gui.add_toolbar(left_toolbar);

        let right_toolbar = Toolbar::new(ToolbarPosition::Right, EditorGui::RIGHT_TOOLBAR_WIDTH)
            .with_element(EditorGui::LAYER_LIST_HEIGHT_FACTOR, LayerListElement::new())
            .with_element(EditorGui::TILESET_LIST_HEIGHT_FACTOR, TilesetListElement::new())
            .with_element(
                EditorGui::TILESET_DETAILS_HEIGHT_FACTOR,
                TilesetDetailsElement::new())
            .with_element(EditorGui::OBJECT_LIST_HEIGHT_FACTOR, ObjectListElement::new());

        gui.add_toolbar(right_toolbar);

        storage::store(gui);

        Editor {
            map,
            selected_tool,
            selected_layer,
            selected_tileset: None,
            selected_tile: None,
            selected_object: None,
            input_scheme,
            cursor_position,
            history: EditorHistory::new(),
        }
    }

    fn get_cursor_position(&self) -> Vec2 {
        if let Some(cursor_position) = self.cursor_position {
            return cursor_position;
        }

        let (x, y) = mouse_position();
        vec2(x, y)
    }

    fn get_selected_tile(&self) -> Option<(String, u32)> {
        if let Some(tileset_id) = self.selected_tileset.clone() {
            if let Some(tile_id) = self.selected_tile {
                let selected = (tileset_id, tile_id);
                return Some(selected);
            }
        }

        None
    }

    fn get_context(&self) -> EditorContext {
        EditorContext {
            selected_tool: self.selected_tool,
            selected_layer: self.selected_layer.clone(),
            selected_tileset: self.selected_tileset.clone(),
            selected_tile: self.selected_tile,
            selected_object: self.selected_object,
            input_scheme: self.input_scheme,
            cursor_position: self.get_cursor_position(),
        }
    }

    fn update_context(&mut self) {
        if let Some(layer_id) = &self.selected_layer {
            if !self.map.draw_order.contains(layer_id) {
                self.selected_layer = None;
            }
        } else if let Some(layer_id) = self.map.draw_order.first() {
            self.selected_layer = Some(layer_id.clone());
        }

        if let Some(layer_id) = &self.selected_layer {
            let layer = self.map.layers.get(layer_id).unwrap();

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
            if let Some(tileset) = self.map.tilesets.get(tileset_id) {
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
            if !tool.is_available(&self.map, &ctx) {
                self.selected_tool = None;
            }
        }
    }

    fn select_tileset(&mut self, tileset_id: &str, tile_id: Option<u32>) {
        if let Some(tileset) = self.map.tilesets.get(tileset_id) {
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
                res = self.history.undo(&mut self.map);
            }
            EditorAction::Redo => {
                res = self.history.redo(&mut self.map);
            }
            EditorAction::SelectTool(index) => {
                self.selected_tool = Some(index);
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
            EditorAction::OpenObjectPropertiesWindow { .. } => {}
            EditorAction::CloseWindow(id) => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.remove_window_id(id);
            }
            EditorAction::SelectTile { id, tileset_id } => {
                self.select_tileset(&tileset_id, Some(id));
            }
            EditorAction::SelectLayer(id) => {
                if self.map.layers.contains_key(&id) {
                    self.selected_layer = Some(id);
                }
            }
            EditorAction::SetLayerDrawOrderIndex { id, index } => {
                let action = SetLayerDrawOrderIndexAction::new(id, index);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::CreateLayer {
                id,
                kind,
                has_collision,
                index,
            } => {
                let action = CreateLayerAction::new(id, kind, has_collision, index);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::DeleteLayer(id) => {
                let action = DeleteLayerAction::new(id);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::SelectTileset(id) => {
                self.select_tileset(&id, None);
            }
            EditorAction::CreateTileset { id, texture_id } => {
                let action = CreateTilesetAction::new(id, texture_id);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::DeleteTileset(id) => {
                let action = DeleteTilesetAction::new(id);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::SetTilesetAutotileMask { id, autotile_mask } => {
                let action = SetTilesetAutotileMaskAction::new(id, autotile_mask);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::SelectObject { index, layer_id } => {
                self.selected_layer = Some(layer_id);
                self.selected_object = Some(index);
            }
            EditorAction::CreateObject {
                name,
                position,
                size,
                layer_id,
            } => {
                let action = CreateObjectAction::new(name, position, size, layer_id);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::DeleteObject { index, layer_id } => {
                let action = DeleteObjectAction::new(index, layer_id);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::PlaceTile {
                id,
                layer_id,
                tileset_id,
                coords,
            } => {
                let action = PlaceTileAction::new(id, layer_id, tileset_id, coords);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::RemoveTile { layer_id, coords } => {
                let action = RemoveTileAction::new(layer_id, coords);
                res = self.history.apply(Box::new(action), &mut self.map);
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

        let input = collect_editor_input(node.input_scheme);

        if input.undo {
            node.apply_action(EditorAction::Undo);
        } else if input.redo {
            node.apply_action(EditorAction::Redo);
        }

        let cursor_position = node.get_cursor_position();
        let element_at_cursor = {
            let gui = storage::get::<EditorGui>();
            gui.get_element_at(cursor_position)
        };

        if input.action {
            if element_at_cursor.is_none() || element_at_cursor.unwrap() != GuiElement::ContextMenu
            {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.close_context_menu();
            }

            if element_at_cursor.is_none() {
                if let Some(id) = &node.selected_tool {
                    let ctx = node.get_context();
                    let tool = get_tool_instance_of_id(id);
                    if let Some(action) = tool.get_action(&node.map, &ctx) {
                        node.apply_action(action);
                    }
                }
            }
        }

        if input.context_menu {
            let mut gui = storage::get_mut::<EditorGui>();
            gui.open_context_menu(cursor_position);
        }
    }

    fn fixed_update(mut node: RefMut<Self>) {
        let input = collect_editor_input(node.input_scheme);

        if let Some(cursor_position) = node.cursor_position {
            let cursor_position = cursor_position + input.cursor_move * Self::CURSOR_MOVE_SPEED;
            node.cursor_position = Some(cursor_position);
        }

        let cursor_position = node.get_cursor_position();
        let element_at_cursor = {
            let gui = storage::get::<EditorGui>();
            gui.get_element_at(cursor_position)
        };

        let screen_size = vec2(screen_width(), screen_height());

        let threshold = screen_size * Self::CAMERA_PAN_THRESHOLD;

        let mut pan_direction = input.camera_pan;

        if cursor_position.x <= threshold.x {
            pan_direction.x = -1.0;
        } else if cursor_position.x >= screen_size.x - threshold.x {
            pan_direction.x = 1.0;
        }

        if cursor_position.y <= threshold.y {
            pan_direction.y = -1.0;
        } else if cursor_position.y >= screen_size.y - threshold.y {
            pan_direction.y = 1.0;
        }

        let mut camera = scene::find_node_by_type::<EditorCamera>().unwrap();
        let movement = pan_direction * Self::CAMERA_PAN_SPEED;

        camera.position = (camera.position + movement).clamp(Vec2::ZERO, node.map.get_size());

        if element_at_cursor.is_none() {
            camera.scale = (camera.scale + input.camera_zoom * Self::CAMERA_ZOOM_STEP)
                .clamp(Self::CAMERA_ZOOM_MIN, Self::CAMERA_ZOOM_MAX);
        }
    }

    fn draw(mut node: RefMut<Self>) {
        node.map.draw(None);

        let res = {
            let ctx = node.get_context();
            let mut gui = storage::get_mut::<EditorGui>();
            gui.draw(&node.map, ctx)
        };

        if let Some(action) = res {
            node.apply_action(action);
        }
    }
}
