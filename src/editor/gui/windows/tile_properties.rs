use ff_core::prelude::*;

use ff_core::map::Map;

use ff_core::gui::Checkbox;
use ff_core::macroquad::hash;
use ff_core::macroquad::ui::Ui;

use super::{ButtonParams, EditorAction, EditorContext, Window, WindowParams};

const JUMPTHROUGH_ATTRIBUTE: &str = "jumpthrough";

pub struct TilePropertiesWindow {
    params: WindowParams,
    layer_id: String,
    index: usize,
    attributes: Option<Vec<String>>,
}

impl TilePropertiesWindow {
    pub fn new(layer_id: String, index: usize) -> Self {
        let params = WindowParams {
            title: Some("Tile Properties".to_string()),
            size: vec2(300.0, 200.0),
            ..Default::default()
        };

        TilePropertiesWindow {
            params,
            layer_id,
            index,
            attributes: None,
        }
    }
}

impl Window for TilePropertiesWindow {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn get_buttons(&self, _map: &Map, _ctx: &EditorContext) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let mut action = None;

        if let Some(attributes) = self.attributes.clone() {
            let batch = self
                .get_close_action()
                .then(EditorAction::UpdateTileAttributes {
                    layer_id: self.layer_id.clone(),
                    index: self.index,
                    attributes,
                });

            action = Some(batch);
        }

        res.push(ButtonParams {
            label: "Save",
            action,
            ..Default::default()
        });

        res.push(ButtonParams {
            label: "Cancel",
            action: Some(self.get_close_action()),
            ..Default::default()
        });

        res
    }

    fn draw(
        &mut self,
        ui: &mut Ui,
        _size: Vec2,
        map: &Map,
        _ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let id = hash!("update_tile_window");

        if self.attributes.is_none() {
            if let Some(layer) = map.layers.get(&self.layer_id) {
                if let Some(Some(tile)) = layer.tiles.get(self.index) {
                    self.attributes = Some(tile.attributes.clone());
                }
            }
        }

        let mut is_jumpthrough;

        if let Some(attributes) = &mut self.attributes {
            let was_jumpthrough = attributes.contains(&(JUMPTHROUGH_ATTRIBUTE.to_string()));
            is_jumpthrough = was_jumpthrough;

            Checkbox::new(hash!(id, "jumpthrough_input"), None, "Platform")
                .ui(ui, &mut is_jumpthrough);

            if is_jumpthrough && !was_jumpthrough {
                attributes.push(JUMPTHROUGH_ATTRIBUTE.to_string());
            } else if !is_jumpthrough && was_jumpthrough {
                attributes.retain(|s| s != JUMPTHROUGH_ATTRIBUTE);
            }
        }

        None
    }
}
