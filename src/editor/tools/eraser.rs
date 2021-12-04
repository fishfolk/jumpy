use macroquad::{color, prelude::*};

use super::{EditorAction, EditorContext, EditorTool, EditorToolParams};

use crate::{
    editor::EditorCamera,
    map::{Map, MapLayerKind},
};

#[derive(Default)]
pub struct EraserTool {
    params: EditorToolParams,
}

impl EraserTool {
    pub fn new() -> Self {
        let params = EditorToolParams {
            name: "Eraser Tool".to_string(),
            ..Default::default()
        };

        EraserTool { params }
    }
}

impl EditorTool for EraserTool {
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
        if let Some(layer_id) = &ctx.selected_layer {
            let layer = map.layers.get(layer_id).unwrap();

            if layer.kind == MapLayerKind::TileLayer {
                let cursor_world_position= scene::find_node_by_type::<EditorCamera>()
                    .unwrap()
                    .to_world_space(ctx.cursor_position);

                let coords = map.to_coords(cursor_world_position);
                let position = map.to_position(coords);

                let outline_color = if layer.tiles[map.to_index(coords)].is_some() {
                    color::YELLOW
                } else {
                    color::RED
                };

                draw_rectangle_lines(
                    position.x,
                    position.y,
                    map.tile_size.x,
                    map.tile_size.y,
                    2.0,
                    outline_color,
                );
            }
        }

        None
    }
}
