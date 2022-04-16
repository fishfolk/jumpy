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

    level_view: LevelView,

    history: ActionHistory,

    create_layer_window: Option<windows::CreateLayerWindow>,
    create_tileset_window: Option<windows::CreateTilesetWindow>,
    menu_window: Option<windows::MenuWindow>,
    save_map_window: Option<windows::SaveMapWindow>,

    level_render_target: RenderTarget,
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

        node.editor.level_view.position += input.camera_move_direction * Self::CAMERA_PAN_SPEED;

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

        scene::set_camera(0, camera);

        if input.toggle_menu {
            node.editor.menu_window = if node.editor.menu_window.is_some() {
                None
            } else {
                Some(MenuWindow::new(
                    node.editor.state.map_resource.meta.is_user_map,
                ))
            };
        }

        if input.undo {
            node.editor.apply_action(UiAction::Undo);
        }
        if input.redo {
            node.editor.apply_action(UiAction::Redo);
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
            history: ActionHistory::new(),
            level_view: LevelView {
                position: Default::default(),
                scale: 1.,
            },
            level_render_target: render_target(1, 1),

            create_layer_window: None,
            create_tileset_window: None,
            menu_window: None,
            save_map_window: None,
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
                self.create_tileset_window = Some(CreateTilesetWindow::new());
            }
            UiAction::OpenSaveMapWindow => {
                self.save_map_window = Some(windows::SaveMapWindow::new());
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
                if self.state.map_resource.map.layers[&id].kind == MapLayerKind::ObjectLayer
                    && self.state.selected_tool == EditorTool::TilePlacer
                {
                    self.state.selected_tool = EditorTool::Cursor;
                }
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
            UiAction::PlaceTile {
                id,
                coords,
                layer_id,
                tileset_id,
            } => {
                let action = actions::PlaceTile::new(id, layer_id, tileset_id, coords);
                self.history
                    .apply(action, &mut self.state.map_resource.map)
                    .unwrap();
            }
            UiAction::RemoveTile { coords, layer_id } => {
                let action = actions::RemoveTile::new(layer_id, coords);
                self.history
                    .apply(action, &mut self.state.map_resource.map)
                    .unwrap();
            }
            UiAction::MoveObject {
                index,
                layer_id,
                position,
            } => {
                let action = actions::MoveObject::new(layer_id, index, position);
                self.history
                    .apply(action, &mut self.state.map_resource.map)
                    .unwrap();
            }
            UiAction::ExitToMainMenu => {
                crate::exit_to_main_menu();
            }
            UiAction::QuitToDesktop => {
                crate::quit_to_desktop();
            }
            UiAction::SelectObject {
                layer_id,
                index,
                cursor_offset,
            } => {
                self.state.selected_map_entity = Some(state::SelectableEntity {
                    click_offset: cursor_offset,
                    kind: state::SelectableEntityKind::Object { index, layer_id },
                })
            }
            UiAction::DeselectObject => self.state.selected_map_entity = None,
            UiAction::SaveMap { name } => {
                let mut map_resource = self.state.map_resource.clone();

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
                    self.state.map_resource = map_resource;
                }
            }
            UiAction::CreateObject {
                id,
                kind,
                layer_id,
                position,
            } => {
                let action = actions::CreateObject::new(id, kind, position, layer_id);
                self.history
                    .apply(action, &mut self.state.map_resource.map)
                    .unwrap();
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

        if let Some(window) = &mut self.save_map_window {
            match window.ui(egui_ctx) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(SaveMapResult::Save { name }) => {
                    self.apply_action(UiAction::SaveMap { name: Some(name) });
                    self.save_map_window = None;
                }
                ControlFlow::Break(SaveMapResult::Close) => {
                    self.save_map_window = None;
                }
            }
        } else if let Some(window) = &mut self.create_tileset_window {
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
        } else if let Some(window) = &mut self.menu_window {
            match window.ui(egui_ctx) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(MenuResult::OpenCreateMapWindow) => {
                    self.apply_action(UiAction::OpenCreateMapWindow)
                }
                ControlFlow::Break(MenuResult::OpenLoadMapWindow) => {
                    self.apply_action(UiAction::OpenLoadMapWindow)
                }
                ControlFlow::Break(MenuResult::OpenSaveMapWindow) => {
                    self.apply_action(UiAction::OpenSaveMapWindow)
                }
                ControlFlow::Break(MenuResult::SaveMap) => {
                    self.apply_action(UiAction::SaveMap { name: None })
                }
                ControlFlow::Break(MenuResult::ExitToMainMenu) => {
                    self.apply_action(UiAction::ExitToMainMenu)
                }
                ControlFlow::Break(MenuResult::QuitToDesktop) => {
                    self.apply_action(UiAction::QuitToDesktop)
                }
            }
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
