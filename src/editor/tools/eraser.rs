use macroquad::prelude::*;

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
}
