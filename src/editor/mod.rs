use std::ops::ControlFlow;

mod camera;
pub mod data;
mod windows;
pub use data::*;

pub use camera::EditorCamera;

mod actions;
use actions::{UiAction, UiActionExt};

mod input;

mod history;

use history::ActionHistory;
pub use input::EditorInputScheme;

use crate::{editor::windows::CreateTilesetWindow, map::MapObjectKind, Resources};

use macroquad::{
    experimental::scene::{Node, RefMut},
    prelude::*,
};

use self::windows::{CreateLayerResult, CreateTilesetResult};

use crate::resources::MapResource;

pub struct Editor {
    ctx: EditorData,

    history: ActionHistory,

    create_layer_window: Option<windows::CreateLayerWindow>,
    create_tileset_window: Option<windows::CreateTilesetWindow>,
}

/// Used to interface with macroquad. Necessary because using `node: RefMut<Self>` really limits
/// what can be done in regards to borrowing.
pub struct EditorNode {
    editor: Editor,
}

impl EditorNode {
    pub fn new(editor: Editor) -> Self {
        Self { editor }
    }
}

impl Editor {
    pub fn new(map_resource: MapResource) -> Self {
        Self {
            ctx: EditorData::new(map_resource),
            history: ActionHistory::new(),
            create_layer_window: None,
            create_tileset_window: None,
        }
    }

    pub fn apply_action(&mut self, action: UiAction) {
        dbg!(&action);

        match action {
            UiAction::Batch(batch) => batch
                .into_iter()
                .for_each(|action| self.apply_action(action)),
            UiAction::Undo => self.history.undo(&mut self.ctx.map_resource.map).unwrap(),
            UiAction::Redo => self.history.redo(&mut self.ctx.map_resource.map).unwrap(),
            UiAction::SelectTool(tool) => self.ctx.selected_tool = tool,
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
                    .apply(action, &mut self.ctx.map_resource.map)
                    .unwrap();
            }
            UiAction::CreateTileset { id, texture_id } => {
                let action = actions::CreateTileset::new(id, texture_id);
                self.history
                    .apply(action, &mut self.ctx.map_resource.map)
                    .unwrap();
            }
            UiAction::DeleteLayer(id) => {
                let action = actions::DeleteLayer::new(id);
                self.history
                    .apply(action, &mut self.ctx.map_resource.map)
                    .unwrap();
                self.ctx.selected_layer = None;
            }
            UiAction::DeleteTileset(id) => {
                let action = actions::DeleteTileset::new(id);
                self.history
                    .apply(action, &mut self.ctx.map_resource.map)
                    .unwrap();
                self.ctx.selected_tileset = None;
            }
            UiAction::UpdateLayer { id, is_visible } => {
                let action = actions::UpdateLayer::new(id, is_visible);
                self.history
                    .apply(action, &mut self.ctx.map_resource.map)
                    .unwrap();
            }
            UiAction::SetLayerDrawOrderIndex { id, index } => {
                let action = actions::SetLayerDrawOrderIndex::new(id, index);
                self.history
                    .apply(action, &mut self.ctx.map_resource.map)
                    .unwrap();
            }
            UiAction::SelectLayer(id) => {
                self.ctx.selected_layer = Some(id);
            }
            UiAction::SelectTileset(id) => {
                self.ctx.selected_tileset = Some(id);
            }

            _ => todo!(),
        }
    }

    pub fn ui(&mut self, egui_ctx: &egui::Context) {
        if let Some(action) = self.ctx.ui(egui_ctx) {
            self.apply_action(action);
        }

        if let Some(window) = &mut self.create_layer_window {
            match window.ui(egui_ctx, &self.ctx.map_resource.map) {
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
            match window.ui(egui_ctx, &self.ctx.map_resource.map) {
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
}

impl Node for EditorNode {
    fn draw(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        egui_macroquad::ui(|egui_ctx| node.editor.ui(egui_ctx));

        egui_macroquad::draw();
    }
}

#[derive(Debug, Clone)]
enum DraggedObject {
    MapObject {
        id: String,
        kind: MapObjectKind,
        index: usize,
        layer_id: String,
        click_offset: Vec2,
    },
    SpawnPoint {
        index: usize,
        click_offset: Vec2,
    },
}
