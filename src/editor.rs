mod camera;

pub use camera::EditorCamera;

mod gui;

use gui::{
    EditorGui,
    EditorGuiElement,
    menus::{
        ContextMenuEntry,
        RIGHT_MENUBAR_WIDTH,
        MENU_ENTRY_HEIGHT,
        get_layer_list_height,
    },
};

mod actions;

pub use actions::UndoableAction;
use actions::{
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
    EditorInput,
    collect_editor_input,
};

use macroquad::{
    experimental::{
        scene::{
            Node,
            Handle,
            RefMut,
        },
    },
    prelude::*,
};

use crate::{
    map::{
        Map,
    },
};

pub struct Editor {
    map: Map,
    current_layer: Option<String>,
    gui: EditorGui,
    input_scheme: EditorInputScheme,
    cursor_position: Option<Vec2>,
    history: EditorHistory,
}

impl Editor {
    const CAMERA_PAN_THRESHOLD: f32 = 0.025;

    const CAMERA_PAN_SPEED: f32 = 5.0;
    const CAMERA_ZOOM_STEP: f32 = 0.05;
    const CAMERA_ZOOM_MAX: f32 = 3.0;

    const CURSOR_MOVE_SPEED: f32 = 5.0;

    pub fn new(input_scheme: EditorInputScheme, map: Map) -> Self {
        let current_layer = map.draw_order.first().cloned();

        let cursor_position = match input_scheme {
            EditorInputScheme::Keyboard => None,
            EditorInputScheme::Gamepad(..) => Some(vec2(
                screen_width() / 2.0,
                screen_height() / 2.0,
            )),
        };

        Editor {
            map,
            current_layer,
            gui: EditorGui::new(),
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

    fn get_context_at(&self, point: Vec2) -> EditorGuiElement {
        if let Some(context_menu) = &self.gui.context_menu {
            if context_menu.contains(point) {
                return EditorGuiElement::ContextMenu;
            }
        }

        if point.x >= screen_width() - RIGHT_MENUBAR_WIDTH {
            if point.y <= get_layer_list_height() {
                let i = (point.y - MENU_ENTRY_HEIGHT) as usize / MENU_ENTRY_HEIGHT as usize;
                if i < self.map.layers.len() {
                    return EditorGuiElement::LayerListEntry(i);
                }

                return EditorGuiElement::LayerList;
            }

            return EditorGuiElement::RightMenuBar;
        }

        EditorGuiElement::None
    }

    fn get_context_menu_entries(&self, context: EditorGuiElement) -> Vec<ContextMenuEntry> {
        use EditorGuiElement::*;
        let mut entries =
            match context {
                RightMenuBar => {
                    vec!()
                }
                LayerList => {
                    vec!(
                        ContextMenuEntry::action("Create layer", EditorAction::OpenCreateLayerMenu(self.map.layers.len())),
                    )
                }
                LayerListEntry(i) => {
                    let layer_id = self.map.draw_order.get(i).cloned().unwrap();

                    let mut entries = vec!(
                        ContextMenuEntry::action("Create layer", EditorAction::OpenCreateLayerMenu(i)),
                        ContextMenuEntry::action("Delete layer", EditorAction::DeleteLayer(layer_id.clone())),
                    );

                    if i > 0 {
                        let action = EditorAction::SetLayerDrawOrderIndex {
                            id: layer_id.clone(),
                            index: i - 1,
                        };

                        let entry = ContextMenuEntry::action("Move layer up", action);

                        entries.push(entry);
                    }

                    if i + 1 < self.map.draw_order.len() {
                        let action = EditorAction::SetLayerDrawOrderIndex {
                            id: layer_id,
                            index: i + 1,
                        };

                        let entry = ContextMenuEntry::action("Move layer down", action);

                        entries.push(entry);
                    }

                    entries
                }
                ContextMenu | None => {
                    vec!()
                }
            };

        if self.history.is_empty() == false {
            let mut common = vec!();

            if self.history.has_undo() {
                let action = ContextMenuEntry::action("Undo", EditorAction::Undo);
                common.push(action);
            }

            if self.history.has_redo() {
                let action = ContextMenuEntry::action("Redo", EditorAction::Redo);
                common.push(action);
            }

            entries.append(&mut common);
        }

        entries
    }

    fn apply_action(&mut self, action: EditorAction) -> Result {
        match action {
            EditorAction::Undo => {
                return self.history.undo(&mut self.map);
            }
            EditorAction::Redo => {
                return self.history.redo(&mut self.map);
            }
            EditorAction::OpenCreateLayerMenu(mut index) => {
                self.gui.open_create_layer_menu(index);
            }
            EditorAction::CloseCreateLayerMenu => {
                self.gui.close_create_layer_menu();
            }
            EditorAction::SelectLayer(id) => {
                self.current_layer = Some(id);
            }
            EditorAction::SetLayerDrawOrderIndex { id, index } => {
                let action = SetLayerDrawOrderIndex::new(id, index);
                return self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::CreateLayer { id, kind, draw_order_index } => {
                let action = CreateLayer::new(id.clone(), kind, draw_order_index);
                return self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::DeleteLayer(id) => {
                let action = DeleteLayer::new(id.clone());
                return self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::CreateTileset { id, texture_id } => {
                let action = CreateTileset::new(id, texture_id);
                return self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::DeleteTileset(id) => {
                let action = DeleteTileset::new(id);
                return self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::PlaceTile { id, layer_id, tileset_id, coords } => {
                let action = PlaceTile::new(id, layer_id, tileset_id, coords);
                return self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::RemoveTile { layer_id, coords } => {
                let action = RemoveTile::new(layer_id, coords);
                return self.history.apply(Box::new(action), &mut self.map);
            }
        }

        Ok(())
    }
}

impl Node for Editor {
    fn update(mut node: RefMut<Self>) {
        if let Some(current_layer) = &node.current_layer {
            if node.map.draw_order.contains(current_layer) == false {
                node.current_layer = None;
            }
        } else {
            if let Some(id) = node.map.draw_order.first().cloned() {
                node.current_layer = Some(id);
            }
        }

        let input = collect_editor_input(node.input_scheme);

        let cursor_position = node.get_cursor_position();
        let cursor_context = node.get_context_at(cursor_position);

        if input.action {
            if cursor_context != EditorGuiElement::ContextMenu {
                node.gui.close_context_menu();
            }
        }

        if input.context_menu {
            let entries = node.get_context_menu_entries(cursor_context);
            node.gui.open_context_menu(cursor_position, &entries);
        }
    }

    fn fixed_update(mut node: RefMut<Self>) {
        let input = collect_editor_input(node.input_scheme);

        if let Some(cursor_position) = node.cursor_position {
            let cursor_position = cursor_position + input.cursor_move * Self::CURSOR_MOVE_SPEED;
            node.cursor_position = Some(cursor_position);
        }

        let cursor_position = node.get_cursor_position();

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

        if cursor_position.y >= screen_size.y - threshold.y {
            pan_direction.y = -1.0;
        } else if cursor_position.y <= threshold.y {
            pan_direction.y = 1.0;
        }

        let mut camera = scene::find_node_by_type::<EditorCamera>().unwrap();

        let movement = pan_direction * Self::CAMERA_PAN_SPEED;
        camera.position = (camera.position + movement).clamp(Vec2::ZERO, node.map.get_size());

        camera.scale = (camera.scale + input.camera_zoom * Self::CAMERA_ZOOM_STEP).clamp(0.0, Self::CAMERA_ZOOM_MAX);
    }

    fn draw(mut node: RefMut<Self>) {
        node.map.draw(None);

        let mut layers = Vec::new();
        for layer_id in &node.map.draw_order {
            let layer = node.map.layers.get(layer_id).unwrap();
            let layer_info = (layer_id.clone(), layer.kind.clone());
            layers.push(layer_info)
        }

        let current_layer = node.current_layer.clone();
        if let Some(action) = node.gui.draw(current_layer, &layers) {
            match node.apply_action(action) {
                Ok(_) => {
                    node.gui.close_context_menu();
                }
                Err(err) => {
                    panic!("EditorAction Error: {}", err)
                }
            }
        }
    }
}