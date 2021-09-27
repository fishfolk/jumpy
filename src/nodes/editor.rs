use macroquad::{
    experimental::{
        scene::{
            self,
            Node,
            Handle,
            RefMut,
        }
    },
    prelude::*,
};

use crate::{
    input::{
        EditorInput,
        EditorInputScheme,
        collect_editor_input,
    },
    gui::editor::{
        EditorGui,
        ContextMenuEntry,
    },
};

use super::{
    EditorCamera,
};

#[derive(Debug, Copy, Clone)]
pub enum EditorAction {
    Undo,
    Redo,
}

pub struct Editor {
    gui: EditorGui,
    input_scheme: EditorInputScheme,
    last_input: EditorInput,
    cursor_position: Option<Vec2>,
    undo_stack: Vec<EditorAction>,
    redo_stack: Vec<EditorAction>,
}

impl Editor {
    const CAMERA_MOVE_THRESHOLD_FACTOR: f32 = 0.2;

    const CAMERA_MOVE_SPEED: f32 = 0.05;
    const CAMERA_ZOOM_SPEED: f32 = 0.1;
    const CAMERA_ZOOM_MAX: f32 = 3.0;

    const CURSOR_MOVE_SPEED: f32 = 0.05;

    pub fn new(input_scheme: EditorInputScheme) -> Self {
        let cursor_position = match input_scheme {
            EditorInputScheme::Keyboard => None,
            EditorInputScheme::Gamepad(..) => Some(vec2(
                screen_width() / 2.0,
                screen_height() / 2.0,
            )),
        };

        let last_input = collect_editor_input(input_scheme);

        Editor {
            gui: EditorGui::new(),
            input_scheme,
            last_input,
            cursor_position,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn apply_action(&mut self, action: EditorAction) {
        use EditorAction::*;
        match action {
            Undo => {
                if let Some(action) = self.undo_stack.pop() {
                    self.redo_stack.push(action);
                }
            }
            Redo => {
                if let Some(action) = self.redo_stack.pop() {
                    self.undo_stack.push(action);
                }
            }
        }
    }
}

impl Node for Editor {
    fn update(mut node: RefMut<Self>) {
        node.last_input = collect_editor_input(node.input_scheme);
    }

    fn fixed_update(mut node: RefMut<Self>) {
        let input = node.last_input;

        let cursor_position =
            if let Some(cursor_position) = node.cursor_position {
                let cursor_position = cursor_position + input.cursor_move * Self::CURSOR_MOVE_SPEED;
                node.cursor_position = Some(cursor_position);
                cursor_position
            } else {
                let (x, y) = mouse_position();
                vec2(x, y)
            };

        if input.context_menu {
            node.gui.show_context_menu(cursor_position, &[
                ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                ContextMenuEntry::new_action("Redo", EditorAction::Redo),
                ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                ContextMenuEntry::new_sub_menu("Sub Menu", &[
                    ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                    ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                    ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                    ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                    ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                    ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                    ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                    ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                    ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                    ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                    ContextMenuEntry::new_sub_menu("Sub Menu", &[
                        ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                        ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                        ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                        ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                        ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                        ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                        ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                        ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                        ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                        ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                        ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                        ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                    ]),

                ]),
                ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                ContextMenuEntry::new_action("Undo", EditorAction::Undo),
                ContextMenuEntry::new_action("Undo", EditorAction::Undo),
            ]);
        }

        let mut camera = scene::find_node_by_type::<EditorCamera>().unwrap();
        let screen_size = vec2(
            screen_width(),
            screen_height(),
        );

        let threshold = screen_size * Self::CAMERA_MOVE_THRESHOLD_FACTOR;

        let mut direction = input.camera_move;

        if cursor_position.x <= threshold.x {
            direction.x = -1.0;
        } else if cursor_position.x >= screen_size.x - threshold.x {
            direction.x = 1.0;
        }

        if cursor_position.y <= threshold.y {
            direction.y = -1.0;
        } else if cursor_position.y >= screen_size.y - threshold.y {
            direction.y = 1.0;
        }

        // TODO: LERP viewport edge to cursor position when movement is due to cursor being over move threshold
        camera.position += input.camera_move * Self::CAMERA_MOVE_SPEED;

        camera.zoom = (camera.zoom + input.camera_zoom * Self::CAMERA_ZOOM_SPEED).clamp(0.0, Self::CAMERA_ZOOM_MAX);

        if input.action {

        }
    }

    fn draw(mut node: RefMut<Self>) {
        if let Some(action) = node.gui.draw()  {
            node.apply_action(action);
        } else if node.last_input.action && node.gui.is_mouse_over_context_menu() == false {
            node.gui.hide_context_menu();
        }
    }
}