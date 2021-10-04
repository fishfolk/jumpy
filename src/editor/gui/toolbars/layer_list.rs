use macroquad::{
    ui::{
        Ui,
        widgets,
    },
    experimental::{
        collections::storage,
    },
    prelude::*,
};

use crate::{
    gui::GuiResources,
    map::{
        Map,
        MapLayerKind,
        ObjectLayerKind,
    },
};

use super::{
    EditorAction,
    EditorDrawParams,
    Toolbar,
    ToolbarElement,
    ToolbarElementParams,
};
use crate::editor::gui::ButtonParams;

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

        LayerListElement {
            params,
        }
    }
}

impl ToolbarElement for LayerListElement {
    fn get_params(&self) -> &ToolbarElementParams {
        &self.params
    }

    fn get_buttons(&self, map: &Map, draw_params: &EditorDrawParams) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let mut delete_action = None;
        let mut move_up_action = None;
        let mut move_down_action = None;

        if let Some(layer_id) = &draw_params.selected_layer {
            let mut index = None;

            {
                let mut i = 0;
                for id in &map.draw_order {
                    if id == layer_id {
                        index = Some(i);
                        break;
                    }

                    i += 1;
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

    fn draw(&mut self, ui: &mut Ui, size: Vec2, map: &Map, draw_params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let entry_size = vec2(size.x, Toolbar::LIST_ENTRY_HEIGHT);
        let mut position = Vec2::ZERO;

        let gui_resources = storage::get::<GuiResources>();
        ui.push_skin(&gui_resources.editor_skins.menu);

        for layer_id in &map.draw_order {
            let layer = map.layers.get(layer_id).unwrap();
            let kind = &layer.kind;

            let is_selected = if let Some(selected_id) = &draw_params.selected_layer {
                layer_id == selected_id
            } else {
                false
            };

            if is_selected {
                ui.push_skin(&gui_resources.editor_skins.menu_selected);
            }

            let button = widgets::Button::new("")
                .size(entry_size)
                .position(position)
                .ui(ui);

            ui.label(position, layer_id);

            {
                let suffix = match kind {
                    MapLayerKind::TileLayer => "T",
                    MapLayerKind::ObjectLayer(kind) => {
                        match kind {
                            ObjectLayerKind::None => "O",
                            ObjectLayerKind::Items => "I",
                            ObjectLayerKind::SpawnPoints => "S",
                        }
                    }
                };

                let suffix_size = ui.calc_size(suffix);
                let position = vec2(size.x - suffix_size.x, position.y);

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
}