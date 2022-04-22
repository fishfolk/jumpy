mod state;
mod util;
mod view;
mod windows;

mod actions;
use actions::UiAction;

mod input;

mod history;

pub use input::EditorInputScheme;
pub use state::Editor;

use macroquad::experimental::scene::{Node, RefMut};

/// Used to interface with macroquad. Necessary because using `node: RefMut<Self>` really limits
/// what can be done in regards to borrowing.
pub struct EditorNode {
    editor: Editor,

    accept_mouse_input: bool,
    accept_kb_input: bool,
    input_scheme: EditorInputScheme,
}

impl EditorNode {
    pub fn new(editor: Editor, input_scheme: EditorInputScheme) -> Self {
        Self {
            editor,
            accept_mouse_input: true,
            accept_kb_input: true,
            input_scheme,
        }
    }
}

impl Node for EditorNode {
    fn fixed_update(mut node: RefMut<Self>) {
        use macroquad::prelude::*;

        let input = node
            .input_scheme
            .collect_input(node.accept_kb_input, node.accept_mouse_input);

        let target_size = vec2(
            node.editor.level_render_target.texture.width(),
            node.editor.level_render_target.texture.height(),
        );
        let zoom = vec2(
            node.editor.level_view.scale / target_size.x,
            node.editor.level_view.scale / target_size.y,
        ) * 2.;
        let camera = Some(Camera2D {
            offset: vec2(-1., -1.),
            target: node.editor.level_view.position,
            zoom,
            render_target: Some(node.editor.level_render_target),
            ..Camera2D::default()
        });

        node.editor.process_input(&input);

        scene::set_camera(0, camera);
    }

    fn draw(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        node.editor.draw();

        egui_macroquad::ui(|egui_ctx| {
            node.editor.ui(egui_ctx);
            node.accept_mouse_input = !egui_ctx.wants_pointer_input();
            node.accept_kb_input = !egui_ctx.wants_keyboard_input();
        });

        egui_macroquad::draw();
    }
}
