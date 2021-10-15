use macroquad::prelude::*;

use super::{EditorAction, EditorContext, EditorTool, EditorToolParams};

use crate::{
    editor::EditorCamera,
    map::{Map, MapLayerKind},
};

#[derive(Default)]
pub struct TilePlacementTool {
    params: EditorToolParams,
}

impl TilePlacementTool {
    pub fn new() -> Self {
        let params = EditorToolParams {
            name: "Tile Placement Tool".to_string(),
            ..Default::default()
        };

        TilePlacementTool { params }
    }
}

impl EditorTool for TilePlacementTool {
    fn get_params(&self) -> &EditorToolParams {
        &self.params
    }

    fn get_action(&mut self, map: &Map, ctx: &EditorContext) -> Option<EditorAction> {
        if let Some(layer_id) = &ctx.selected_layer {
            let camera = scene::find_node_by_type::<EditorCamera>().unwrap();
            let world_position = camera.to_world_space(ctx.cursor_position);

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

        None
    }

    fn is_available(&self, map: &Map, ctx: &EditorContext) -> bool {
        if let Some(layer_id) = &ctx.selected_layer {
            let layer = map.layers.get(layer_id).unwrap();
            return layer.kind == MapLayerKind::TileLayer;
        }

        false
    }
}

#[derive(Default)]
pub struct ObjectPlacementTool {
    params: EditorToolParams,
}

impl ObjectPlacementTool {
    pub fn new() -> Self {
        let params = EditorToolParams {
            name: "Object Placement Tool".to_string(),
            ..Default::default()
        };

        ObjectPlacementTool { params }
    }
}

impl EditorTool for ObjectPlacementTool {
    fn get_params(&self) -> &EditorToolParams {
        &self.params
    }

    fn get_action(&mut self, _map: &Map, _ctx: &EditorContext) -> Option<EditorAction> {
        // if let Some(layer_id) = &ctx.selected_layer {
        //let layer = map.layers.get(layer_id).unwrap();
        //let camera = scene::find_node_by_type::<EditorCamera>().unwrap();
        // let world_position = camera.to_world_space(ctx.cursor_position);

        // TODO: Implement object layers
        // }

        None
    }

    fn is_available(&self, map: &Map, ctx: &EditorContext) -> bool {
        if let Some(layer_id) = &ctx.selected_layer {
            let layer = map.layers.get(layer_id).unwrap();
            return layer.kind == MapLayerKind::ObjectLayer;
        }

        false
    }
}
