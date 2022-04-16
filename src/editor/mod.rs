use std::{ops::ControlFlow, path::Path};

mod state;
mod util;
mod view;
mod windows;

mod actions;
use actions::UiAction;

mod input;

mod history;

use history::ActionHistory;
pub use input::EditorInputScheme;

use crate::{
    editor::{
        state::{EditorTool, TileSelection},
        windows::{CreateTilesetWindow, MenuWindow},
    },
    map::{Map, MapLayerKind},
    resources::{map_name_to_filename, MAP_EXPORTS_DEFAULT_DIR, MAP_EXPORTS_EXTENSION},
    Resources,
};

use macroquad::{
    experimental::scene::{Node, RefMut},
    prelude::{collections::storage, render_target, RenderTarget},
};

use self::{
    view::LevelView,
    windows::{CreateLayerResult, CreateTilesetResult, MenuResult, SaveMapResult},
};

use crate::resources::MapResource;

pub struct Editor {
    state: state::State,
}

/// Used to interface with macroquad. Necessary because using `node: RefMut<Self>` really limits
/// what can be done in regards to borrowing.
pub struct EditorNode {
    editor: Editor,

    accept_mouse_input: bool,
    accept_kb_input: bool,
    input_scheme: EditorInputScheme,
}

impl EditorNode {
    const CAMERA_PAN_SPEED: f32 = 5.0;

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

        node.editor.state.level_view.position +=
            input.camera_move_direction * Self::CAMERA_PAN_SPEED;

        let target_size = vec2(
            node.editor.state.level_render_target.texture.width(),
            node.editor.state.level_render_target.texture.height(),
        );
        let zoom = vec2(
            node.editor.state.level_view.scale / target_size.x,
            node.editor.state.level_view.scale / target_size.y,
        ) * 2.;
        let camera = Some(Camera2D {
            offset: vec2(-1., -1.),
            target: node.editor.state.level_view.position,
            zoom,
            render_target: Some(node.editor.state.level_render_target),
            ..Camera2D::default()
        });

        scene::set_camera(0, camera);

        if input.toggle_menu {
            node.editor.state.menu_window = if node.editor.state.menu_window.is_some() {
                None
            } else {
                Some(Default::default())
            };
        }

        if input.undo {
            node.editor.state.apply_action(UiAction::Undo);
        }
        if input.redo {
            node.editor.state.apply_action(UiAction::Redo);
        }
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

impl Editor {
    const CAMERA_PAN_THRESHOLD: f32 = 0.025;

    const CAMERA_ZOOM_STEP: f32 = 0.1;
    const CAMERA_ZOOM_MIN: f32 = 0.1;
    const CAMERA_ZOOM_MAX: f32 = 2.5;

    const CURSOR_MOVE_SPEED: f32 = 5.0;

    const OBJECT_SELECTION_RECT_SIZE: f32 = 75.0;
    const OBJECT_SELECTION_RECT_PADDING: f32 = 8.0;

    const MESSAGE_TIMEOUT: f32 = 2.5;

    pub fn new(map_resource: MapResource) -> Self {
        Self {
            state: state::State::new(map_resource),
        }
    }

    pub fn ui(&mut self, egui_ctx: &egui::Context) {
        if let Some(action) = self.state.ui(egui_ctx) {
            self.state.apply_action(action);
        }
    }

    pub fn draw(&self) {
        let map = &self.state.map_resource.map;
        {
            map.draw_background(None, !self.state.is_parallax_enabled);
            map.draw(None, false);
        }

        if self.state.should_draw_grid {
            self::draw_grid(map);
        }
    }
}

fn draw_grid(map: &Map) {
    const GRID_LINE_WIDTH: f32 = 1.0;
    const GRID_COLOR: macroquad::prelude::Color = macroquad::prelude::Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.25,
    };

    use macroquad::prelude::*;

    let map_size = map.grid_size.as_f32() * map.tile_size;

    draw_rectangle_lines(
        map.world_offset.x,
        map.world_offset.y,
        map_size.x,
        map_size.y,
        GRID_LINE_WIDTH,
        GRID_COLOR,
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

        draw_line(begin.x, begin.y, end.x, end.y, GRID_LINE_WIDTH, GRID_COLOR)
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

        draw_line(begin.x, begin.y, end.x, end.y, GRID_LINE_WIDTH, GRID_COLOR)
    }
}
