use super::{EditorAction, EditorContext, EditorTool, EditorToolParams};

use crate::map::{Map, MapLayerKind};

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
        let mut res = None;

        if let Some(layer_id) = ctx.selected_layer.clone() {
            let coords = map.to_coords(ctx.cursor_position);
            res = Some(EditorAction::RemoveTile { layer_id, coords });
        }

        res
    }

    fn is_available(&self, map: &Map, ctx: &EditorContext) -> bool {
        if let Some(layer_id) = &ctx.selected_layer {
            let layer = map.layers.get(layer_id).unwrap();
            return layer.kind == MapLayerKind::TileLayer;
        }

        false
    }
}
