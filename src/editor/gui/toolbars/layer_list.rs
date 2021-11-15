use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{widgets, Ui},
};

use crate::{
    gui::GuiResources,
    map::{Map, MapLayerKind},
};

use super::{
    ButtonParams, EditorAction, EditorContext, Toolbar, ToolbarElement, ToolbarElementParams,
};
use crate::gui::ELEMENT_MARGIN;

pub struct LayerListElement {
    params: ToolbarElementParams,
}

impl LayerListElement {
    pub fn new() -> Self {
        let params = ToolbarElementParams {
            header: Some("Layers".to_string()),
            has_buttons: true,
            has_margins: false,
        };

        LayerListElement { params }
    }
}

impl ToolbarElement for LayerListElement {
    fn get_params(&self) -> &ToolbarElementParams {
        &self.params
    }

    fn draw(
        &mut self,
        ui: &mut Ui,
        size: Vec2,
        map: &Map,
        ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let mut res = None;

        let entry_size = vec2(size.x, Toolbar::LIST_ENTRY_HEIGHT);
        let mut position = Vec2::ZERO;

        let gui_resources = storage::get::<GuiResources>();
        ui.push_skin(&gui_resources.skins.list_box);

        for layer_id in &map.draw_order {
            let layer = map.layers.get(layer_id).unwrap();

            let is_selected = if let Some(selected_id) = &ctx.selected_layer {
                layer_id == selected_id
            } else {
                false
            };

            if is_selected {
                ui.push_skin(&gui_resources.skins.list_box_selected);
            }

            let button = widgets::Button::new("")
                .size(entry_size)
                .position(position)
                .ui(ui);

            ui.label(position, layer_id);

            if layer.kind == MapLayerKind::ObjectLayer {
                let suffix = "(Obj)";

                let suffix_size = ui.calc_size(suffix);
                let position = vec2(size.x - suffix_size.x - ELEMENT_MARGIN, position.y);

                ui.label(position, suffix);
            }

            if button {
                res = Some(EditorAction::SelectLayer(layer_id.clone()));
            }

            if is_selected {
                ui.pop_skin();
            }

            position.y += entry_size.y;
        }

        ui.pop_skin();

        res
    }

    fn get_buttons(&self, map: &Map, ctx: &EditorContext) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let mut delete_action = None;
        let mut move_up_action = None;
        let mut move_down_action = None;

        if let Some(layer_id) = &ctx.selected_layer {
            let mut index = None;

            {
                for (i, id) in map.draw_order.iter().enumerate() {
                    if id == layer_id {
                        index = Some(i);
                        break;
                    }
                }
            }

            delete_action = Some(EditorAction::DeleteLayer(layer_id.clone()));

            if let Some(index) = index {
                if index > 0 {
                    move_up_action = Some(EditorAction::SetLayerDrawOrderIndex {
                        id: layer_id.clone(),
                        index: index - 1,
                    });
                }

                if index + 1 < map.draw_order.len() {
                    move_down_action = Some(EditorAction::SetLayerDrawOrderIndex {
                        id: layer_id.clone(),
                        index: index + 1,
                    });
                }
            }
        }

        res.push(ButtonParams {
            label: "+",
            action: Some(EditorAction::OpenCreateLayerWindow),
            ..Default::default()
        });

        res.push(ButtonParams {
            label: "-",
            action: delete_action,
            ..Default::default()
        });

        res.push(ButtonParams {
            label: "Up",
            action: move_up_action,
            ..Default::default()
        });

        res.push(ButtonParams {
            label: "Down",
            action: move_down_action,
            ..Default::default()
        });

        res
    }
}

impl Default for LayerListElement {
    fn default() -> Self {
        Self::new()
    }
}
