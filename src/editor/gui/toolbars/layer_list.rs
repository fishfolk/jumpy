use macroquad::{
    ui::{
        Id,
        Ui,
        hash,
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

pub struct LayerList {
    params: ToolbarElementParams,
}

impl LayerList {
    pub fn new() -> Box<Self> {
        let params = ToolbarElementParams {
            id: hash!("layer_list"),
            header: Some("Layers".to_string()),
            has_menubar: true,
            has_margins: false,
        };

        Box::new(LayerList {
            params,
        })
    }
}

impl ToolbarElement for LayerList {
    fn get_params(&self) -> ToolbarElementParams {
        self.params.clone()
    }

    fn draw(&mut self, ui: &mut Ui, size: Vec2, map: &Map, draw_params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let entry_size = vec2(size.x, Toolbar::LIST_ENTRY_HEIGHT);
        let mut position = Vec2::ZERO;

        let gui_resources = storage::get::<GuiResources>();
        ui.push_skin(&gui_resources.editor_skins.menu);

        for id in &map.draw_order {
            let layer = map.layers.get(id).unwrap();
            let kind = &layer.kind;

            let is_selected = if let Some(selected_id) = &draw_params.selected_layer {
                id == selected_id
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

            ui.label(position, id);

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
                res = Some(EditorAction::SelectLayer(id.clone()));
            }

            if is_selected {
                ui.pop_skin();
            }

            position.y += entry_size.y;
        }

        ui.pop_skin();

        res
    }

    fn draw_menubar(&mut self, ui: &mut Ui, size: Vec2, map: &Map, draw_params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let mut position = Vec2::ZERO;

        let button_size = vec2(size.x * 0.25, Toolbar::MENUBAR_HEIGHT);

        let create_btn = widgets::Button::new("+")
            .size(button_size)
            .position(position)
            .ui(ui);

        if create_btn {
            res = Some(EditorAction::OpenCreateLayerWindow);
        }

        position.x += button_size.x;

        let delete_btn = widgets::Button::new("-")
            .size(button_size)
            .position(position)
            .ui(ui);

        if delete_btn {
            if let Some(layer_id) = draw_params.selected_layer.clone() {
                res = Some(EditorAction::DeleteLayer(layer_id));
            }
        }

        position.x += button_size.x;

        let up_btn = widgets::Button::new("Up")
            .size(button_size)
            .position(position)
            .ui(ui);

        if up_btn {
            if let Some(layer_id) = &draw_params.selected_layer {
                let mut i = 0;
                for id in &map.draw_order {
                    if id == layer_id && i > 0 {
                        res = Some(EditorAction::SetLayerDrawOrderIndex {
                            id: layer_id.clone(),
                            index: i - 1,
                        });

                        break;
                    }

                    i += 1;
                }
            }
        }

        position.x += button_size.x;

        let down_btn = widgets::Button::new("Down")
            .size(button_size)
            .position(position)
            .ui(ui);

        if down_btn {
            if let Some(layer_id) = &draw_params.selected_layer {
                let mut i = 0;
                for id in &map.draw_order {
                    if id == layer_id && i + 1 < map.draw_order.len() {
                        res = Some(EditorAction::SetLayerDrawOrderIndex {
                            id: layer_id.clone(),
                            index: i + 1,
                        });

                        break;
                    }

                    i += 1;
                }
            }
        }

        res
    }
}