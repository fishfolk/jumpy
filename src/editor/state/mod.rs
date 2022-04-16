mod ui;

use std::path::Path;

use macroquad::prelude::{collections::storage, RenderTarget};

use crate::{
    editor::actions,
    map::{MapLayerKind, MapObjectKind},
    resources::{
        map_name_to_filename, MapResource, MAP_EXPORTS_DEFAULT_DIR, MAP_EXPORTS_EXTENSION,
    },
    Resources,
};

use super::{actions::UiAction, history::ActionHistory, view::LevelView, windows};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EditorTool {
    Cursor,
    TilePlacer,
    ObjectPlacer,
    SpawnPointPlacer,
    Eraser,
}

pub struct TileSelection {
    pub tileset: String,
    pub tile_id: u32,
}

pub enum SelectableEntityKind {
    Object { layer_id: String, index: usize },
    SpawnPoint { index: usize },
}

pub struct SelectableEntity {
    pub kind: SelectableEntityKind,
    pub click_offset: egui::Vec2,
}

impl SelectableEntityKind {
    pub fn is_object(&self, layer_id: &str, index: usize) -> bool {
        match self {
            SelectableEntityKind::Object {
                layer_id: l,
                index: i,
            } => l == layer_id && i == &index,
            SelectableEntityKind::SpawnPoint { .. } => false,
        }
    }
}

#[derive(Debug)]
pub struct ObjectSettings {
    pub position: egui::Pos2,
    pub kind: MapObjectKind,
    pub id: Option<String>,
}

pub struct State {
    pub selected_tool: EditorTool,
    pub map_resource: MapResource,
    pub selected_layer: Option<String>,
    pub selected_tile: Option<TileSelection>,
    pub is_parallax_enabled: bool,
    pub should_draw_grid: bool,
    pub selected_map_entity: Option<SelectableEntity>,
    pub object_being_placed: Option<ObjectSettings>,

    create_layer_window: Option<windows::CreateLayerWindow>,
    create_tileset_window: Option<windows::CreateTilesetWindow>,
    pub menu_window: Option<windows::MenuWindow>,
    save_map_window: Option<windows::SaveMapWindow>,

    history: ActionHistory,

    pub level_view: LevelView,

    pub level_render_target: RenderTarget,
}

impl State {
    pub fn new(map_resource: MapResource) -> Self {
        Self {
            map_resource,
            selected_tool: EditorTool::Cursor,
            is_parallax_enabled: true,
            should_draw_grid: true,
            selected_layer: None,
            selected_tile: None,
            selected_map_entity: None,
            object_being_placed: None,

            create_layer_window: None,
            create_tileset_window: None,
            menu_window: None,
            save_map_window: None,

            history: ActionHistory::new(),

            level_view: LevelView {
                position: Default::default(),
                scale: 1.,
            },

            level_render_target: macroquad::prelude::render_target(1, 1),
        }
    }

    pub fn apply_action(&mut self, action: UiAction) {
        dbg!(&action);

        match action {
            UiAction::Batch(batch) => batch
                .into_iter()
                .for_each(|action| self.apply_action(action)),
            UiAction::Undo => self.history.undo(&mut self.map_resource.map).unwrap(),
            UiAction::Redo => self.history.redo(&mut self.map_resource.map).unwrap(),
            UiAction::SelectTool(tool) => self.selected_tool = tool,
            UiAction::OpenCreateLayerWindow => self.create_layer_window = Some(Default::default()),
            UiAction::OpenCreateTilesetWindow => {
                self.create_tileset_window = Some(Default::default());
            }
            UiAction::OpenSaveMapWindow => {
                self.save_map_window = Some(Default::default());
            }
            UiAction::CreateLayer {
                id,
                kind,
                has_collision,
                index,
            } => {
                let action = actions::CreateLayer::new(id, kind, has_collision, index);
                self.history
                    .apply(action, &mut self.map_resource.map)
                    .unwrap();
            }
            UiAction::CreateTileset { id, texture_id } => {
                let action = actions::CreateTileset::new(id, texture_id);
                self.history
                    .apply(action, &mut self.map_resource.map)
                    .unwrap();
            }
            UiAction::DeleteLayer(id) => {
                let action = actions::DeleteLayer::new(id);
                self.history
                    .apply(action, &mut self.map_resource.map)
                    .unwrap();
                self.selected_layer = None;
            }
            UiAction::DeleteTileset(id) => {
                let action = actions::DeleteTileset::new(id);
                self.history
                    .apply(action, &mut self.map_resource.map)
                    .unwrap();
                self.selected_tile = None;
            }
            UiAction::UpdateLayer { id, is_visible } => {
                let action = actions::UpdateLayer::new(id, is_visible);
                self.history
                    .apply(action, &mut self.map_resource.map)
                    .unwrap();
            }
            UiAction::SetLayerDrawOrderIndex { id, index } => {
                let action = actions::SetLayerDrawOrderIndex::new(id, index);
                self.history
                    .apply(action, &mut self.map_resource.map)
                    .unwrap();
            }
            UiAction::SelectLayer(id) => {
                if self.map_resource.map.layers[&id].kind == MapLayerKind::ObjectLayer
                    && self.selected_tool == EditorTool::TilePlacer
                {
                    self.selected_tool = EditorTool::Cursor;
                }
                self.selected_layer = Some(id);
            }
            UiAction::SelectTileset(id) => {
                self.selected_tile = Some(TileSelection {
                    tileset: id,
                    tile_id: 0,
                });
            }
            UiAction::SelectTile { id, tileset_id } => {
                self.selected_tile = Some(TileSelection {
                    tileset: tileset_id,
                    tile_id: id,
                });
                self.selected_tool = EditorTool::TilePlacer;
            }
            UiAction::PlaceTile {
                id,
                coords,
                layer_id,
                tileset_id,
            } => {
                let action = actions::PlaceTile::new(id, layer_id, tileset_id, coords);
                self.history
                    .apply(action, &mut self.map_resource.map)
                    .unwrap();
            }
            UiAction::RemoveTile { coords, layer_id } => {
                let action = actions::RemoveTile::new(layer_id, coords);
                self.history
                    .apply(action, &mut self.map_resource.map)
                    .unwrap();
            }
            UiAction::MoveObject {
                index,
                layer_id,
                position,
            } => {
                let action = actions::MoveObject::new(layer_id, index, position);
                self.history
                    .apply(action, &mut self.map_resource.map)
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
                self.selected_map_entity = Some(SelectableEntity {
                    click_offset: cursor_offset,
                    kind: SelectableEntityKind::Object { index, layer_id },
                })
            }
            UiAction::DeselectObject => self.selected_map_entity = None,
            UiAction::SaveMap { name } => {
                let mut map_resource = self.map_resource.clone();

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
                    self.map_resource = map_resource;
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
                    .apply(action, &mut self.map_resource.map)
                    .unwrap();
            }

            _ => todo!(),
        }
    }

    pub fn selected_layer_type(&self) -> Option<MapLayerKind> {
        self.selected_layer
            .as_ref()
            .and_then(|id| self.map_resource.map.layers.get(id))
            .map(|layer| layer.kind)
    }
}
