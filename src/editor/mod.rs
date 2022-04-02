use std::any::TypeId;
use std::ops::ControlFlow;
use std::path::Path;

use crate::{exit_to_main_menu, quit_to_desktop, Resources};

mod camera;
mod windows;

pub use camera::EditorCamera;

mod actions;

use actions::{
    CreateLayerAction, CreateObjectAction, CreateTilesetAction, DeleteLayerAction,
    DeleteObjectAction, DeleteTilesetAction, EditorAction, PlaceTileAction, RemoveTileAction,
    SetLayerDrawOrderIndexAction, UndoableAction, UpdateTilesetAction,
};

mod input;

mod history;

use history::EditorHistory;
pub use input::EditorInputScheme;

use crate::editor::actions::{
    CreateSpawnPointAction, DeleteSpawnPointAction, ImportAction, MoveSpawnPointAction,
    UpdateBackgroundAction, UpdateLayerAction, UpdateObjectAction, UpdateTileAttributesAction,
};
use crate::editor::input::{collect_editor_input, EditorInput};
use crate::gui::SELECTION_HIGHLIGHT_COLOR;
use crate::map::{MapObject, MapObjectKind};
use crate::player::IDLE_ANIMATION_ID;
use macroquad::{
    color,
    experimental::{
        collections::storage,
        scene::{Node, RefMut},
    },
    prelude::*,
};

use core::text::{draw_aligned_text, HorizontalAlignment, VerticalAlignment};

use self::windows::CreateLayerResult;

use super::map::{Map, MapLayerKind};
use crate::resources::{
    map_name_to_filename, MapResource, MAP_EXPORTS_DEFAULT_DIR, MAP_EXPORTS_EXTENSION,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EditorTool {
    Cursor,
    TilePlacer,
    ObjectPlacer,
    SpawnPointPlacer,
    Eraser,
}

pub struct NewEditorContext {
    pub selected_tool: EditorTool,
    pub map_resource: MapResource,
    pub selected_layer: Option<String>,
}

impl NewEditorContext {
    pub fn new(map_resource: MapResource) -> Self {
        Self {
            map_resource,
            selected_tool: EditorTool::Cursor,
            selected_layer: None,
        }
    }

    pub fn ui(&self, egui_ctx: &egui::Context) -> Option<EditorAction> {
        let mut action = None;

        egui::SidePanel::new(egui::containers::panel::Side::Left, "Tools").show(egui_ctx, |ui| {
            let tool = &self.selected_tool;

            let mut add_tool = |tool_name, tool_variant| {
                ui.add(egui::SelectableLabel::new(tool == &tool_variant, tool_name))
                    .clicked()
                    .then(|| action = Some(EditorAction::SelectTool(tool_variant)));
            };
            add_tool("Cursor", EditorTool::Cursor);
            add_tool("Tiles", EditorTool::TilePlacer);
            add_tool("Objects", EditorTool::ObjectPlacer);
            add_tool("Eraser", EditorTool::Eraser);
        });
        egui::SidePanel::new(egui::containers::panel::Side::Right, "Layers").show(egui_ctx, |ui| {
            ui.heading("Layers");
            for (layer_name, layer) in self.map_resource.map.layers.iter() {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "({}) {}",
                        match layer.kind {
                            MapLayerKind::TileLayer => "T",
                            MapLayerKind::ObjectLayer => "O",
                        },
                        layer_name
                    ));
                    let mut is_visible = layer.is_visible;
                    if ui.checkbox(&mut is_visible, "Visible").clicked() {
                        action = Some(EditorAction::UpdateLayer {
                            id: layer_name.clone(),
                            is_visible,
                        });
                    }
                });
            }
            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    action = Some(EditorAction::OpenCreateLayerWindow);
                }
                ui.button("-");
                ui.button("Up");
                ui.button("Down");
            });
            ui.separator();
            ui.heading("Tilesets");
            ui.horizontal(|ui| {
                ui.button("+");
                ui.button("-");
                ui.button("Edit");
            });
        });

        action
    }
}

pub struct Editor {
    ctx: NewEditorContext,

    history: EditorHistory,

    create_layer_window: Option<windows::CreateLayerWindow>,
}

impl Editor {
    pub fn new(map_resource: MapResource) -> Self {
        Self {
            ctx: NewEditorContext::new(map_resource),
            history: EditorHistory::new(),
            create_layer_window: None,
        }
    }

    pub fn apply_action(&mut self, action: EditorAction) {
        dbg!("Applying action:", &action);
        match action {
            EditorAction::Batch(batch) => batch
                .into_iter()
                .for_each(|action| self.apply_action(action)),
            EditorAction::Undo => self.history.undo(&mut self.ctx.map_resource.map).unwrap(),
            EditorAction::Redo => self.history.redo(&mut self.ctx.map_resource.map).unwrap(),
            EditorAction::SelectTool(tool) => self.ctx.selected_tool = tool,
            EditorAction::OpenCreateLayerWindow => {
                self.create_layer_window = Some(Default::default())
            }
            EditorAction::CreateLayer {
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
            EditorAction::UpdateLayer { id, is_visible } => {
                let action = UpdateLayerAction::new(id, is_visible);
                self.history
                    .apply(action, &mut self.ctx.map_resource.map)
                    .unwrap();
            }

            _ => todo!(),
        }
    }
}

impl Node for Editor {
    fn draw(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        egui_macroquad::ui(|egui_ctx| {
            if let Some(action) = node.ctx.ui(egui_ctx) {
                node.apply_action(action);
            }

            if let Some(window) = &mut node.create_layer_window {
                match window.ui(egui_ctx) {
                    ControlFlow::Continue(()) => (),
                    ControlFlow::Break(CreateLayerResult::Create {
                        has_collision,
                        layer_kind,
                        layer_name,
                    }) => {
                        node.apply_action(EditorAction::CreateLayer {
                            has_collision,
                            kind: layer_kind,
                            index: None,
                            id: layer_name,
                        });
                        node.create_layer_window = None;
                    }
                    ControlFlow::Break(CreateLayerResult::Close) => {
                        node.create_layer_window = None;
                    }
                }
            }
        });

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
