mod ui;

use crate::{map::MapLayerKind, resources::MapResource};

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

pub enum DraggableEntityKind {
    Object { layer_id: String, index: usize },
    SpawnPoint { index: usize },
}

pub struct DraggableEntity {
    kind: DraggableEntityKind,
    click_offset: egui::Vec2,
}

/// Contains the editor state, i.e. the data whose change is tracked by the [`ActionHistory`] of the
/// editor.
pub struct State {
    pub selected_tool: EditorTool,
    pub map_resource: MapResource,
    pub selected_layer: Option<String>,
    pub selected_tile: Option<TileSelection>,
    pub is_parallax_enabled: bool,
    pub should_draw_grid: bool,
    pub entity_being_dragged: Option<DraggableEntity>,
}

impl State {
    pub fn new(map_resource: MapResource) -> Self {
        Self {
            map_resource,
            selected_tool: EditorTool::Cursor,
            selected_layer: None,
            selected_tile: None,
            is_parallax_enabled: true,
            should_draw_grid: true,
            entity_being_dragged: None,
        }
    }

    pub fn selected_layer_type(&self) -> Option<MapLayerKind> {
        self.selected_layer
            .as_ref()
            .and_then(|id| self.map_resource.map.layers.get(id))
            .map(|layer| layer.kind)
    }
}
