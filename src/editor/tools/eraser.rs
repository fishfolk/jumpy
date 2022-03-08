use core::prelude::*;

use super::{EditorAction, EditorContext, EditorTool, EditorToolParams};

use crate::{
    editor::EditorCamera,
    map::{Map, MapLayerKind},
};
use crate::macroquad::experimental::scene;

#[derive(Default)]
pub struct EraserTool {
    params: EditorToolParams,
}

impl EraserTool {
    pub fn new() -> Self {
        let params = EditorToolParams {
            name: "Erase Tiles".to_string(),
            icon_texture_id: "eraser_tool_icon".to_string(),
            is_continuous: true,
        };

        EraserTool { params }
    }
}

impl EditorTool for EraserTool {
    fn get_params(&self) -> &EditorToolParams {
        &self.params
    }

    fn get_action(&mut self, map: &Map, ctx: &EditorContext) -> Option<EditorAction> {
        let cursor_world_position = scene::find_node_by_type::<EditorCamera>()
            .unwrap()
            .to_world_space(ctx.cursor_position);

        if map.contains(cursor_world_position) {
            if let Some(layer_id) = &ctx.selected_layer {
                let layer = map.layers.get(layer_id).unwrap();
                let camera = scene::find_node_by_type::<EditorCamera>().unwrap();
                let world_position = camera.to_world_space(ctx.cursor_position);

                match layer.kind {
                    MapLayerKind::TileLayer => {
                        let coords = map.to_coords(world_position);

                        return Some(EditorAction::RemoveTile {
                            layer_id: layer_id.clone(),
                            coords,
                        });
                    }
                    MapLayerKind::ObjectLayer => {
                        // TODO: Implement object layers
                    }
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

    fn draw_cursor(&mut self, map: &Map, ctx: &EditorContext) -> Option<EditorAction> {
        let cursor_world_position = scene::find_node_by_type::<EditorCamera>()
            .unwrap()
            .to_world_space(ctx.cursor_position);

        if map.contains(cursor_world_position) {
            if let Some(layer_id) = &ctx.selected_layer {
                let layer = map.layers.get(layer_id).unwrap();

                if layer.kind == MapLayerKind::TileLayer {
                    let coords = map.to_coords(cursor_world_position);
                    let position = map.to_position(coords);

                    let outline_color = if layer.tiles[map.to_index(coords)].is_some() {
                        colors::YELLOW
                    } else {
                        colors::RED
                    };

                    draw_rectangle_outline(
                        position.x,
                        position.y,
                        map.tile_size.width,
                        map.tile_size.height,
                        2.0,
                        outline_color,
                    );
                }
            }
        }

        None
    }
}
