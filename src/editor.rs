mod camera;

pub use camera::EditorCamera;

pub mod gui;

use gui::{
    GuiElement,
    CreateLayerWindow,
    CreateTilesetWindow,
    EditorDrawParams,
    EditorGui,
    context_menu::{
        ContextMenuEntry,
    },
};

mod actions;
use actions::{
    UndoableAction,
    Result,
    EditorAction,
    SetLayerDrawOrderIndex,
    CreateLayer,
    DeleteLayer,
    CreateTileset,
    DeleteTileset,
    PlaceTile,
    RemoveTile,
};

mod input;

mod history;

use history::EditorHistory;
pub use input::EditorInputScheme;

use input::{
    collect_editor_input,
};

use macroquad::{
    experimental::{
        scene::{
            Node,
            RefMut,
        },
        collections::storage,
    },
    prelude::*,
};

use crate::{
    map::{
        Map,
        MapLayerKind,
    },
};

pub struct Editor {
    map: Map,
    selected_layer: Option<String>,
    selected_tileset: Option<String>,
    selected_tile: Option<u32>,
    input_scheme: EditorInputScheme,
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
        let selected_layer = map.draw_order.first().cloned();
        let mut selected_tileset = None;
        let mut selected_tile = None;

        if map.tilesets.is_empty() == false {
            for (key, tileset) in &map.tilesets {
                selected_tileset = Some(key.clone());
                selected_tile = Some(tileset.first_tile_id);
                break;
            }
        }

        let cursor_position = match input_scheme {
            EditorInputScheme::Keyboard => None,
            EditorInputScheme::Gamepad(..) => Some(vec2(
                screen_width() / 2.0,
                screen_height() / 2.0,
            )),
        };

        let gui = EditorGui::new();
        storage::store(gui);

        Editor {
            map,
            selected_layer,
            selected_tileset,
            selected_tile,
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

    fn get_selected_tile(&self) -> Option<(u32, String)> {
        if let Some(tileset_id) = self.selected_tileset.clone() {
            if let Some(tile_id) = self.selected_tile.clone() {
                let selected = (tile_id, tileset_id);
                return Some(selected);
            }
        }

        None
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
            EditorAction::SelectTile { id, tileset_id } => {
                if let Some(tileset) = self.map.tilesets.get(&tileset_id) {
                    if id < tileset.first_tile_id + tileset.tile_cnt {
                        self.selected_tileset = Some(tileset_id);
                        self.selected_tile = Some(id);
                    }
                }
            }
            EditorAction::OpenCreateLayerWindow => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(CreateLayerWindow::new());
            }
            EditorAction::SelectLayer(id) => {
                if self.map.layers.contains_key(&id) {
                    self.selected_layer = Some(id);
                }
            }
            EditorAction::SetLayerDrawOrderIndex { id, index } => {
                let action = SetLayerDrawOrderIndex::new(id, index);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::CreateLayer { id, kind, draw_order_index } => {
                let action = CreateLayer::new(id.clone(), kind, draw_order_index);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::DeleteLayer(id) => {
                if let Some(selected_id) = &self.selected_layer {
                    if selected_id == &id {
                        self.selected_layer = None;
                    }
                }

                let action = DeleteLayer::new(id.clone());
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::OpenCreateTilesetWindow => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(CreateTilesetWindow::new());
            }
            EditorAction::SelectTileset(id) => {
                if self.map.tilesets.contains_key(&id) {
                    self.selected_tileset = Some(id);
                }
            }
            EditorAction::CreateTileset { id, texture_id } => {
                let action = CreateTileset::new(id, texture_id);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::DeleteTileset(id) => {
                if let Some(selected_id) = &self.selected_tileset {
                    if selected_id == &id {
                        self.selected_tileset = None;
                        self.selected_tile = None;
                    }
                }

                let action = DeleteTileset::new(id);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::PlaceTile { id, layer_id, tileset_id, coords } => {
                let action = PlaceTile::new(id, layer_id, tileset_id, coords);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::RemoveTile { layer_id, coords } => {
                let action = RemoveTile::new(layer_id, coords);
                res = self.history.apply(Box::new(action), &mut self.map);
            }
        }

        if let Err(err) = res {
            panic!("EditorAction Error: {}", err);
        }
    }
}

impl Node for Editor {
    fn update(mut node: RefMut<Self>) {
        if let Some(current_layer) = &node.selected_layer {
            if node.map.draw_order.contains(current_layer) == false {
                node.selected_layer = None;
            }
        } else {
            if let Some(id) = node.map.draw_order.first().cloned() {
                node.selected_layer = Some(id);
            }
        }

        let input = collect_editor_input(node.input_scheme);

        let cursor_position = node.get_cursor_position();
        let element_at_cursor = {
            let gui = storage::get_mut::<EditorGui>();
            gui.get_element_at(cursor_position)
        };

        let cursor_world_position = {
            let camera = scene::find_node_by_type::<EditorCamera>().unwrap();
            camera.to_world_space(cursor_position)
        };

        if input.undo {
            node.apply_action(EditorAction::Undo);
        } else if input.redo {
            node.apply_action(EditorAction::Redo);
        }

        if input.action {
            if element_at_cursor.is_none() || element_at_cursor.unwrap() != GuiElement::ContextMenu {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.close_context_menu();
            }

            if element_at_cursor.is_none() {
                if let Some(layer_id) = node.selected_layer.clone() {
                    if let Some(layer_kind) = node.map.get_layer_kind(&layer_id) {
                        match layer_kind {
                            MapLayerKind::TileLayer => {
                                if let Some(selected) = node.get_selected_tile() {
                                    let (id, tileset_id) = selected;
                                    let coords = node.map.to_coords(cursor_world_position);

                                    let action = EditorAction::PlaceTile {
                                        id,
                                        layer_id,
                                        tileset_id,
                                        coords,
                                    };

                                    node.apply_action(action);
                                }
                            }
                            MapLayerKind::ObjectLayer(..) => {
                                // TODO: Implement object layers
                            }
                        }
                    }
                }
            }
        }

        if input.context_menu  {
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

        let screen_size = vec2(
            screen_width(),
            screen_height(),
        );

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
            camera.scale = (camera.scale + input.camera_zoom * Self::CAMERA_ZOOM_STEP).clamp(Self::CAMERA_ZOOM_MIN, Self::CAMERA_ZOOM_MAX);
        }
    }

    fn draw(mut node: RefMut<Self>) {
        node.map.draw(None);

        let params = EditorDrawParams {
            selected_layer: node.selected_layer.clone(),
            selected_tileset: node.selected_tileset.clone(),
            selected_tile: node.selected_tile,
        };

        let res = {
            let mut gui = storage::get_mut:: < EditorGui>();
            gui.draw(&node.map, params)
        };

        if let Some(action) = res {
            node.apply_action(action);

            let mut gui = storage::get_mut:: < EditorGui>();
            gui.close_context_menu();
        }
    }
}