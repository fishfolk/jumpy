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

            let layer_btn = widgets::Button::new("")
                .size(entry_size)
                .position(position)
                .ui(ui);

            let label = if layer.kind == MapLayerKind::ObjectLayer {
                format!("(O) {}", layer_id)
            } else {
                format!("(T) {}", layer_id)
            };

            ui.label(position, &label);

            if layer_btn {
                res = Some(EditorAction::SelectLayer(layer_id.clone()));
            }

            if is_selected {
                ui.pop_skin();
            }

            ui.push_skin(&gui_resources.skins.list_box_no_bg);

            {
                let btn_size = vec2(50.0, entry_size.y);
                let btn_position = vec2(position.x + entry_size.x - btn_size.x - ELEMENT_MARGIN, position.y);

                let label = if layer.is_visible {
                    "[hide]"
                } else {
                    "[show]"
                };

                let visibility_btn = widgets::Button::new("")
                    .size(btn_size)
                    .position(btn_position)
                    .ui(ui);

                widgets::Label::new(label)
                    .position(btn_position)
                    .ui(ui);

                if visibility_btn {
                    let action = EditorAction::UpdateLayer {
                        id: layer_id.clone(),
                        is_visible: !layer.is_visible,
                    };

                    res = Some(action);
                }
            }

            ui.pop_skin();

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
