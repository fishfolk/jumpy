use std::any::TypeId;
use std::path::Path;

use crate::{exit_to_main_menu, quit_to_desktop, Resources};

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
    Result, SetLayerDrawOrderIndexAction, SetTilesetAutotileMaskAction, UndoableAction,
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

use input::collect_editor_input;

use crate::editor::actions::{UpdateBackgroundAction, UpdateObjectAction};
use crate::editor::gui::windows::{
    BackgroundPropertiesWindow, LoadMapWindow, ObjectPropertiesWindow, SaveMapWindow,
};
use crate::gui::SELECTED_OBJECT_HIGHLIGHT_COLOR;
use crate::map::{MapObject, MapObjectKind};
use macroquad::{
    color,
    experimental::{
        collections::storage,
        scene::{Node, RefMut},
    },
    prelude::*,
};

use super::map::{Map, MapLayerKind};
use crate::resources::{map_name_to_filename, MapResource};

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
            is_user_map: false,
            is_tiled_map: false,
        }
    }
}

pub struct Editor {
    map_resource: MapResource,
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

    const OBJECT_SELECTION_RECT_SIZE: f32 = 75.0;
    const OBJECT_SELECTION_RECT_PADDING: f32 = 8.0;

    pub fn new(input_scheme: EditorInputScheme, map_resource: MapResource) -> Self {
        add_tool_instance(TilePlacementTool::new());
        add_tool_instance(ObjectPlacementTool::new());
        add_tool_instance(EraserTool::new());

        let selected_tool = None;

        let selected_layer = map_resource.map.draw_order.first().cloned();

        let cursor_position = match input_scheme {
            EditorInputScheme::Keyboard => None,
            EditorInputScheme::Gamepad(..) => {
                Some(vec2(screen_width() / 2.0, screen_height() / 2.0))
            }
        };

        let tool_selector_element = ToolSelectorElement::new()
            .with_tool::<TilePlacementTool>()
            .with_tool::<ObjectPlacementTool>()
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
            cursor_position: self.get_cursor_position(),
            is_user_map: self.map_resource.meta.is_user_map,
            is_tiled_map: self.map_resource.meta.is_tiled_map,
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
            EditorAction::SelectTool(index) => {
                self.selected_tool = Some(index);
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
            EditorAction::CloseWindow(id) => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.remove_window_id(id);
            }
            EditorAction::SelectTile { id, tileset_id } => {
                self.select_tileset(&tileset_id, Some(id));
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
            EditorAction::SetTilesetAutotileMask { id, autotile_mask } => {
                let action = SetTilesetAutotileMaskAction::new(id, autotile_mask);
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
            EditorAction::CreateMap { .. } => {
                unimplemented!(
                    "Map creation from editor is not implemented. Use main menu to create new maps"
                );
            }
            EditorAction::OpenCreateMapWindow => {
                unimplemented!(
                    "Map creation from editor is not implemented. Use main menu to create new maps"
                );
            }
            EditorAction::LoadMap(index) => {
                let resources = storage::get::<Resources>();
                let map_resource = resources.maps.get(index).cloned().unwrap();

                self.map_resource = map_resource;
                self.history.clear();
            }
            EditorAction::OpenLoadMapWindow => {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.add_window(LoadMapWindow::new());
            }
            EditorAction::SaveMap(name) => {
                let mut map_resource = self.map_resource.clone();

                if let Some(name) = name {
                    let path = Path::new(Resources::MAP_EXPORTS_DEFAULT_DIR)
                        .join(map_name_to_filename(&name))
                        .with_extension(Resources::MAP_EXPORTS_EXTENSION);

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

        let input = collect_editor_input(node.input_scheme);

        if input.undo {
            node.apply_action(EditorAction::Undo);
        } else if input.redo {
            node.apply_action(EditorAction::Redo);
        }

        let cursor_position = node.get_cursor_position();
        let cursor_world_position = scene::find_node_by_type::<EditorCamera>()
            .unwrap()
            .to_world_space(cursor_position);

        if input.action {
            let (is_cursor_over_gui, is_cursor_over_context_menu) = {
                let gui = storage::get::<EditorGui>();
                let is_over_gui = gui.contains(cursor_position);
                let mut is_over_context_menu = false;
                if is_over_gui && gui.context_menu_contains(cursor_position) {
                    is_over_context_menu = true;
                }

                (is_over_gui, is_over_context_menu)
            };

            if !is_cursor_over_context_menu {
                let mut gui = storage::get_mut::<EditorGui>();
                gui.close_context_menu();
            }

            if !is_cursor_over_gui {
                if let Some(id) = &node.selected_tool {
                    let ctx = node.get_context();
                    let tool = get_tool_instance_of_id(id);
                    if let Some(action) = tool.get_action(node.get_map(), &ctx) {
                        node.apply_action(action);
                    }
                } else {
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

                    {
                        'layers: for layer_id in layer_ids {
                            let layer = node.map_resource.map.layers.get(&layer_id).unwrap();
                            if layer.kind == MapLayerKind::ObjectLayer {
                                for (i, object) in layer.objects.iter().enumerate() {
                                    let size = get_object_size(object);
                                    let position =
                                        object.position + node.map_resource.map.world_offset;

                                    let rect = Rect::new(position.x, position.y, size.x, size.y);

                                    if rect.contains(cursor_world_position) {
                                        let action = EditorAction::SelectObject {
                                            index: i,
                                            layer_id: layer.id.clone(),
                                        };

                                        node.apply_action(action);

                                        break 'layers;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if input.context_menu {
            let mut gui = storage::get_mut::<EditorGui>();
            gui.open_context_menu(cursor_position, &node.map_resource.map, node.get_context());
        }
    }

    fn fixed_update(mut node: RefMut<Self>) {
        let input = collect_editor_input(node.input_scheme);

        if input.toggle_menu {
            toggle_editor_menu(&node.get_context());
        }

        if let Some(cursor_position) = node.cursor_position {
            let cursor_position = cursor_position + input.cursor_move * Self::CURSOR_MOVE_SPEED;
            node.cursor_position = Some(cursor_position);
        }

        let cursor_position = node.get_cursor_position();
        let is_cursor_over_map = {
            let gui = storage::get::<EditorGui>();
            !gui.contains(cursor_position)
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

        camera.position = (camera.position + movement).clamp(Vec2::ZERO, node.get_map().get_size());

        if is_cursor_over_map {
            camera.scale = (camera.scale + input.camera_zoom * Self::CAMERA_ZOOM_STEP)
                .clamp(Self::CAMERA_ZOOM_MIN, Self::CAMERA_ZOOM_MAX);
        }
    }

    fn draw(mut node: RefMut<Self>) {
        node.get_map_mut().draw(None);

        {
            let resources = storage::get::<Resources>();

            for layer in node.get_map().layers.values() {
                if layer.kind == MapLayerKind::ObjectLayer {
                    for (i, object) in layer.objects.iter().enumerate() {
                        let mut label = None;

                        let mut is_selected = false;
                        if let Some(layer_id) = &node.selected_layer {
                            if let Some(index) = node.selected_object {
                                is_selected = layer_id == &layer.id && index == i;
                            }
                        }

                        let object_position = node.map_resource.map.world_offset + object.position;

                        match object.kind {
                            MapObjectKind::Item => {
                                if let Some(params) = resources.items.get(&object.id) {
                                    if let Some(texture_res) =
                                        resources.textures.get(&params.sprite.texture_id)
                                    {
                                        let position = object_position + params.sprite.offset;

                                        let frame_size = texture_res
                                            .meta
                                            .sprite_size
                                            .map(|v| v.as_f32())
                                            .unwrap_or_else(|| {
                                                vec2(
                                                    texture_res.texture.width(),
                                                    texture_res.texture.height(),
                                                )
                                            });

                                        let source_rect = {
                                            let grid_size = vec2(
                                                texture_res.texture.width() / frame_size.x,
                                                texture_res.texture.height() / frame_size.y,
                                            )
                                            .as_u32();

                                            let i = params.sprite.index as u32;
                                            let coords = uvec2(i % grid_size.y, i / grid_size.y);

                                            Rect::new(
                                                coords.x as f32 * frame_size.x,
                                                coords.y as f32 * frame_size.y,
                                                frame_size.x,
                                                frame_size.y,
                                            )
                                        };

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
                                    } else {
                                        label = Some("INVALID TEXTURE ID".to_string());
                                    }
                                } else {
                                    label = Some("INVALID OBJECT ID".to_string());
                                }
                            }
                            MapObjectKind::Decoration => {
                                let texture_res =
                                    resources.textures.get("default_decorations").unwrap();

                                let frame_size = texture_res
                                    .meta
                                    .sprite_size
                                    .map(|v| v.as_f32())
                                    .unwrap_or_else(|| {
                                        vec2(
                                            texture_res.texture.width(),
                                            texture_res.texture.height(),
                                        )
                                    });

                                let mut source_rect = None;
                                if &object.id == "pot" {
                                    source_rect = Some(Rect::new(
                                        0.0,
                                        frame_size.y,
                                        frame_size.x,
                                        frame_size.y,
                                    ));
                                } else if &object.id == "seaweed" {
                                    source_rect =
                                        Some(Rect::new(0.0, 0.0, frame_size.x, frame_size.y));
                                }

                                if source_rect.is_some() {
                                    draw_texture_ex(
                                        texture_res.texture,
                                        object_position.x,
                                        object_position.y,
                                        color::WHITE,
                                        DrawTextureParams {
                                            dest_size: Some(frame_size),
                                            source: source_rect,
                                            ..Default::default()
                                        },
                                    );
                                } else {
                                    label = Some("INVALID OBJECT ID".to_string());
                                }
                            }
                            MapObjectKind::Environment => {
                                if &object.id == "sproinger" {
                                    let texture_res = resources.textures.get("sproinger").unwrap();

                                    let frame_size = texture_res
                                        .meta
                                        .sprite_size
                                        .map(|v| v.as_f32())
                                        .unwrap_or_else(|| {
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
                                } else {
                                    label = Some("INVALID OBJECT ID".to_string());
                                }
                            }
                            MapObjectKind::SpawnPoint => {
                                label = Some("Spawn Point".to_string());
                            }
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
                                SELECTED_OBJECT_HIGHLIGHT_COLOR,
                            );
                        }
                    }
                }
            }
        }

        let res = {
            let ctx = node.get_context();
            let mut gui = storage::get_mut::<EditorGui>();
            gui.draw(node.get_map(), ctx)
        };

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
            if let Some(params) = resources.items.get(&object.id) {
                if resources.textures.get(&params.sprite.texture_id).is_some() {
                    res = Some(params.collider_size.as_f32());
                } else {
                    label = Some("INVALID TEXTURE ID".to_string());
                }
            } else {
                label = Some("INVALID OBJECT ID".to_string())
            }
        }
        MapObjectKind::Decoration => {
            if &object.id == "pot" || &object.id == "seaweed" {
                let texture_res = resources.textures.get("default_decorations").unwrap();
                res = texture_res.meta.sprite_size.map(|s| s.as_f32());
            } else {
                label = Some("INVALID OBJECT ID".to_string())
            }
        }
        MapObjectKind::Environment => {
            if &object.id == "sproinger" {
                let texture_res = resources.textures.get("sproinger").unwrap();
                res = texture_res.meta.sprite_size.map(|s| s.as_f32());
            } else {
                label = Some("INVALID OBJECT ID".to_string())
            }
        }
        MapObjectKind::SpawnPoint => {
            label = Some("Spawn Point".to_string());
        }
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
