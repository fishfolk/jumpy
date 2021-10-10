use macroquad::prelude::*;

use super::{EditorAction, EditorContext, EditorTool, EditorToolParams};

use crate::{
    editor::EditorCamera,
    map::{
        Map,
        MapLayerKind,
    },
};

#[derive(Default)]
pub struct PlacementTool {
    params: EditorToolParams,
}

impl PlacementTool {
    pub fn new() -> Self {
        let params = EditorToolParams {
            name: "Placement Tool".to_string(),
            ..Default::default()
        };

        PlacementTool { params }
    }
}

impl EditorTool for PlacementTool {
    fn get_params(&self) -> &EditorToolParams {
        &self.params
    }

    fn get_action(&mut self, map: &Map, ctx: &EditorContext) -> Option<EditorAction> {
        if let Some(layer_id) = &ctx.selected_layer {
            let layer = map.layers.get(layer_id).unwrap();
            let camera = scene::find_node_by_type::<EditorCamera>().unwrap();
            let world_position = camera.to_world_space(ctx.cursor_position);

            match layer.kind {
                MapLayerKind::TileLayer => {
                    if let Some(tileset_id) = &ctx.selected_tileset {
                        if let Some(tile_id) = ctx.selected_tile {
                            let coords = map.to_coords(world_position);

                            return Some(EditorAction::PlaceTile {
                                id: tile_id,
                                layer_id: layer_id.clone(),
                                tileset_id: tileset_id.clone(),
                                coords,
                            });
                        }
                    }
                }
                MapLayerKind::ObjectLayer => {
                    // TODO: Implement object layers
                }
            }
        }

        None
    }
}
