use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{hash, widgets, Ui},
};

use crate::editor::gui::combobox::ComboBoxVec;
use crate::{
    editor::gui::{ComboBoxBuilder, ComboBoxValue},
    map::{Map, MapObjectKind},
    Resources,
};

use super::{ButtonParams, EditorAction, EditorContext, Window, WindowParams};

pub struct CreateObjectWindow {
    params: WindowParams,
    id: Option<String>,
    kind: MapObjectKind,
    position: Vec2,
    layer_id: String,
}

impl CreateObjectWindow {
    pub fn new(position: Vec2, layer_id: String) -> Self {
        let params = WindowParams {
            title: Some("Create Object".to_string()),
            size: vec2(300.0, 300.0),
            ..Default::default()
        };

        CreateObjectWindow {
            params,
            id: None,
            kind: MapObjectKind::Item,
            position,
            layer_id,
        }
    }
}

impl Window for CreateObjectWindow {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn get_buttons(&self, _map: &Map, _ctx: &EditorContext) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        if let Some(id) = self.id.clone() {
            let action = self.get_close_action().then(EditorAction::CreateObject {
                id,
                kind: self.kind,
                position: self.position,
                layer_id: self.layer_id.clone(),
            });

            res.push(ButtonParams {
                label: "Create",
                action: Some(action),
                ..Default::default()
            });
        }

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
        _map: &Map,
        _ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let id = hash!("create_object_window");

        {
            let size = vec2(72.0, 28.0);

            let mut x_str = format!("{:.1}", self.position.x);
            let mut y_str = format!("{:.1}", self.position.y);

            widgets::InputText::new(hash!(id, "position_x_input"))
                .size(size)
                .ui(ui, &mut x_str);

            ui.same_line(0.0);

            ui.label(None, "x");

            ui.same_line(0.0);

            widgets::InputText::new(hash!(id, "position_y_input"))
                .size(size)
                .ui(ui, &mut y_str);

            ui.separator();
            ui.separator();
            ui.separator();
            ui.separator();

            let x = if let Ok(x) = x_str.parse::<f32>() {
                (x * 100.0).round() / 100.0
            } else {
                0.0
            };

            let y = if let Ok(y) = y_str.parse::<f32>() {
                (y * 100.0).round() / 100.0
            } else {
                0.0
            };

            self.position = vec2(x, y);
        }

        ComboBoxBuilder::new(hash!(id, "type_input"))
            .with_ratio(0.8)
            .with_label("Type")
            .build(ui, &mut self.kind);

        let resources = storage::get::<Resources>();
        let item_ids = match self.kind {
            MapObjectKind::Item => resources
                .items
                .values()
                .map(|item| item.id.as_str())
                .collect::<Vec<&str>>(),
            MapObjectKind::Environment => {
                vec!["sproinger"]
            }
            MapObjectKind::Decoration => {
                vec!["pot", "seaweed"]
            }
        };

        let mut item_id_value = if let Some(current_id) = &self.id {
            let index = item_ids
                .iter()
                .enumerate()
                .find_map(|(i, id)| if id == current_id { Some(i) } else { None })
                .unwrap_or_default();

            ComboBoxVec::new(index, &item_ids)
        } else {
            ComboBoxVec::new(0, &item_ids)
        };

        ComboBoxBuilder::new(hash!("id_input"))
            .with_ratio(0.8)
            .with_label("Variant")
            .build(ui, &mut item_id_value);

        self.id = Some(item_id_value.get_value());

        None
    }
}
