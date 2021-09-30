mod camera;

pub use camera::EditorCamera;

mod gui;

use gui::{
    EditorGui,
    EditorGuiElement,
    menus::ContextMenuEntry,
};

mod actions;

pub use actions::UndoableAction;
use actions::{
    Result,
    EditorAction,
    CreateLayer,
    DeleteLayer,
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

    fn get_context_menu_entries(&self, context: EditorGuiElement) -> Vec<ContextMenuEntry> {
        match context {
            _ => {
                vec!(
                    ContextMenuEntry::action("Undo", EditorAction::Undo),
                    ContextMenuEntry::action("Redo", EditorAction::Redo),
                    ContextMenuEntry::sub_menu("Redo", &[
                        ContextMenuEntry::action("Undo", EditorAction::Undo),
                        ContextMenuEntry::action("Redo", EditorAction::Redo),
                    ]),
                )
            }
        }
    }

    fn apply_action(&mut self, action: EditorAction) -> Result {
        match action {
            EditorAction::Undo => {
                return self.history.undo(&mut self.map);
            }
            EditorAction::Redo => {
                return self.history.redo(&mut self.map);
            }
            EditorAction::SelectLayer(id) => {
                self.current_layer = Some(id);
            }
            EditorAction::CreateLayer { id, kind, draw_order_index } => {
                let action = CreateLayer::new(id, kind, draw_order_index);
                return self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::DeleteLayer(id) => {
                let action = DeleteLayer::new(id);
                return self.history.apply(Box::new(action), &mut self.map);
            }
            EditorAction::CreateTileset { id, texture_id} => {
                let action
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
        let input = collect_editor_input(node.input_scheme);

        let cursor_position = node.get_cursor_position();
        let element_at_cursor = node.gui.get_element_at(cursor_position);

        if input.action {
            if element_at_cursor != EditorGuiElement::ContextMenu {
                node.gui.close_context_menu();
            }
        }

        if input.context_menu {
            let entries = node.get_context_menu_entries(element_at_cursor);
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

        // TODO: LERP viewport edge to cursor position when movement is due to cursor being over move threshold
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
            if let Err(err) = node.apply_action(action) {
                println!("EditorAction Error: {}", err);
            }
        }
    }
}