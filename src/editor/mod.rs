use std::ops::ControlFlow;

mod camera;
pub mod data;
mod windows;
pub use data::*;

pub use camera::EditorCamera;

mod actions;

use actions::{
    CreateLayerAction, DeleteLayerAction, SetLayerDrawOrderIndexAction, UiAction, UndoableAction,
};

mod input;

mod history;

use history::ActionHistory;
pub use input::EditorInputScheme;

use crate::editor::actions::UpdateLayerAction;

use crate::map::MapObjectKind;

use macroquad::{
    experimental::scene::{Node, RefMut},
    prelude::*,
};

use self::windows::CreateLayerResult;

use crate::resources::MapResource;

pub struct Editor {
    ctx: EditorData,

    history: ActionHistory,

    create_layer_window: Option<windows::CreateLayerWindow>,
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
        }
    }

    pub fn apply_action(&mut self, action: UiAction) {
        dbg!("Applying action:", &action);

        match action {
            UiAction::Batch(batch) => batch
                .into_iter()
                .for_each(|action| self.apply_action(action)),
            UiAction::Undo => self.history.undo(&mut self.ctx.map_resource.map).unwrap(),
            UiAction::Redo => self.history.redo(&mut self.ctx.map_resource.map).unwrap(),
            UiAction::SelectTool(tool) => self.ctx.selected_tool = tool,
            UiAction::OpenCreateLayerWindow => self.create_layer_window = Some(Default::default()),
            UiAction::CreateLayer {
                id,
                kind,
                has_collision,
                index,
            } => {
                let action = CreateLayerAction::new(id, kind, has_collision, index);
                self.history
                    .apply(action, &mut self.ctx.map_resource.map)
                    .unwrap();
            }
            UiAction::DeleteLayer(id) => {
                let action = DeleteLayerAction::new(id);
                self.history
                    .apply(action, &mut self.ctx.map_resource.map)
                    .unwrap();
                self.ctx.selected_layer = None;
            }
            UiAction::UpdateLayer { id, is_visible } => {
                let action = UpdateLayerAction::new(id, is_visible);
                self.history
                    .apply(action, &mut self.ctx.map_resource.map)
                    .unwrap();
            }
            UiAction::SetLayerDrawOrderIndex { id, index } => {
                let action = SetLayerDrawOrderIndexAction::new(id, index);
                self.history
                    .apply(action, &mut self.ctx.map_resource.map)
                    .unwrap();
            }
            UiAction::SelectLayer(id) => {
                self.ctx.selected_layer = Some(id);
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
