use std::ops::ControlFlow;

mod camera;
pub mod state;
mod windows;
pub use state::*;

pub use camera::EditorCamera;

mod actions;
use actions::UiAction;

mod input;

mod history;

use history::ActionHistory;
pub use input::EditorInputScheme;

use crate::{editor::windows::CreateTilesetWindow, map::Map};

use macroquad::{
    experimental::scene::{Node, RefMut},
    prelude::{collections::storage, render_target, RenderTarget},
};

use self::windows::{CreateLayerResult, CreateTilesetResult};

use crate::resources::MapResource;

pub struct Editor {
    state: EditorState,

    level_view: LevelView,

    history: ActionHistory,

    create_layer_window: Option<windows::CreateLayerWindow>,
    create_tileset_window: Option<windows::CreateTilesetWindow>,

    level_render_target: RenderTarget,
}

/// Used to interface with macroquad. Necessary because using `node: RefMut<Self>` really limits
/// what can be done in regards to borrowing.
pub struct EditorNode {
    editor: Editor,

    accept_mouse_input: bool,
    input_scheme: EditorInputScheme,
}

impl EditorNode {
    const CAMERA_PAN_SPEED: f32 = 5.0;

    pub fn new(editor: Editor, input_scheme: EditorInputScheme) -> Self {
        Self {
            editor,
            accept_mouse_input: true,
            input_scheme,
        }
    }
}

impl Node for EditorNode {
    fn fixed_update(mut node: RefMut<Self>) {
        use macroquad::prelude::*;

        let input = node.input_scheme.collect_input();

        node.editor.level_view.position += input.camera_move_direction * Self::CAMERA_PAN_SPEED;

        let camera = Some(Camera2D {
            offset: vec2(0.0, 0.0),
            target: node.editor.level_view.position,
            zoom: vec2(
                node.editor.level_view.scale / node.editor.level_render_target.texture.width(),
                node.editor.level_view.scale / node.editor.level_render_target.texture.height(),
            ),
            render_target: Some(node.editor.level_render_target),
            ..Camera2D::default()
        });

        scene::set_camera(0, camera);
    }

    fn draw(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        node.editor.draw();

        egui_macroquad::ui(|egui_ctx| {
            node.editor.ui(egui_ctx);
            node.accept_mouse_input = !egui_ctx.wants_pointer_input()
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

    const DOUBLE_CLICK_THRESHOLD: f32 = 0.25;

    const MESSAGE_TIMEOUT: f32 = 2.5;

    pub fn new(map_resource: MapResource) -> Self {
        Self {
            state: EditorState::new(map_resource),
            history: ActionHistory::new(),
            create_layer_window: None,
            create_tileset_window: None,
            level_view: LevelView {
                position: Default::default(),
                scale: 1.,
            },
            level_render_target: render_target(1, 1),
        }
    }

    pub fn apply_action(&mut self, action: UiAction) {
        dbg!(&action);

        match action {
            UiAction::Batch(batch) => batch
                .into_iter()
                .for_each(|action| self.apply_action(action)),
            UiAction::Undo => self.history.undo(&mut self.state.map_resource.map).unwrap(),
            UiAction::Redo => self.history.redo(&mut self.state.map_resource.map).unwrap(),
            UiAction::SelectTool(tool) => self.state.selected_tool = tool,
            UiAction::OpenCreateLayerWindow => self.create_layer_window = Some(Default::default()),
            UiAction::OpenCreateTilesetWindow => {
                self.create_tileset_window = Some(CreateTilesetWindow::new())
            }
            UiAction::CreateLayer {
                id,
                kind,
                has_collision,
                index,
            } => {
                let action = actions::CreateLayer::new(id, kind, has_collision, index);
                self.history
                    .apply(action, &mut self.state.map_resource.map)
                    .unwrap();
            }
            UiAction::CreateTileset { id, texture_id } => {
                let action = actions::CreateTileset::new(id, texture_id);
                self.history
                    .apply(action, &mut self.state.map_resource.map)
                    .unwrap();
            }
            UiAction::DeleteLayer(id) => {
                let action = actions::DeleteLayer::new(id);
                self.history
                    .apply(action, &mut self.state.map_resource.map)
                    .unwrap();
                self.state.selected_layer = None;
            }
            UiAction::DeleteTileset(id) => {
                let action = actions::DeleteTileset::new(id);
                self.history
                    .apply(action, &mut self.state.map_resource.map)
                    .unwrap();
                self.state.selected_tile = None;
            }
            UiAction::UpdateLayer { id, is_visible } => {
                let action = actions::UpdateLayer::new(id, is_visible);
                self.history
                    .apply(action, &mut self.state.map_resource.map)
                    .unwrap();
            }
            UiAction::SetLayerDrawOrderIndex { id, index } => {
                let action = actions::SetLayerDrawOrderIndex::new(id, index);
                self.history
                    .apply(action, &mut self.state.map_resource.map)
                    .unwrap();
            }
            UiAction::SelectLayer(id) => {
                self.state.selected_layer = Some(id);
            }
            UiAction::SelectTileset(id) => {
                self.state.selected_tile = Some(TileSelection {
                    tileset: id,
                    tile_id: 0,
                });
            }
            UiAction::SelectTile { id, tileset_id } => {
                self.state.selected_tile = Some(TileSelection {
                    tileset: tileset_id,
                    tile_id: id,
                });
                self.state.selected_tool = EditorTool::TilePlacer;
            }

            _ => todo!(),
        }
    }

    pub fn ui(&mut self, egui_ctx: &egui::Context) {
        if let Some(action) =
            self.state
                .ui(egui_ctx, &mut self.level_render_target, &self.level_view)
        {
            self.apply_action(action);
        }

        if let Some(window) = &mut self.create_layer_window {
            match window.ui(egui_ctx, &self.state.map_resource.map) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(CreateLayerResult::Create {
                    has_collision,
                    layer_kind,
                    layer_name,
                }) => {
                    self.apply_action(UiAction::CreateLayer {
                        has_collision,
                        kind: layer_kind,
                        index: None,
                        id: layer_name,
                    });
                    self.create_layer_window = None;
                }
                ControlFlow::Break(CreateLayerResult::Close) => {
                    self.create_layer_window = None;
                }
            }
        }

        if let Some(window) = &mut self.create_tileset_window {
            match window.ui(egui_ctx, &self.state.map_resource.map) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(CreateTilesetResult::Create {
                    tileset_name,
                    texture,
                }) => {
                    self.apply_action(UiAction::CreateTileset {
                        id: tileset_name,
                        texture_id: texture,
                    });
                    self.create_tileset_window = None;
                }
                ControlFlow::Break(CreateTilesetResult::Close) => {
                    self.create_tileset_window = None;
                }
            }
        }
    }

    pub fn draw(&self) {
        use macroquad::prelude::*;

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
