use core::prelude::*;

use core::gui::combobox::ComboBoxVec;
use core::map::MapObject;
use core::{
    gui::{ComboBoxBuilder, ComboBoxValue},
    map::{Map, MapObjectKind},
};
use crate::items::items;
use crate::macroquad::hash;
use crate::macroquad::ui::{Ui, widgets};

use super::{ButtonParams, EditorAction, EditorContext, Window, WindowParams};

pub struct ObjectPropertiesWindow {
    params: WindowParams,
    layer_id: String,
    index: usize,
    object: Option<MapObject>,
}

impl ObjectPropertiesWindow {
    pub fn new(layer_id: String, index: usize) -> Self {
        let params = WindowParams {
            title: Some("Object Properties".to_string()),
            size: vec2(300.0, 300.0),
            ..Default::default()
        };

        ObjectPropertiesWindow {
            params,
            layer_id,
            index,
            object: None,
        }
    }
}

impl Window for ObjectPropertiesWindow {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn get_buttons(&self, _map: &Map, _ctx: &EditorContext) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let mut action = None;

        if let Some(object) = &self.object {
            let batch = self.get_close_action().then(EditorAction::UpdateObject {
                layer_id: self.layer_id.clone(),
                index: self.index,
                id: object.id.clone(),
                kind: object.kind,
                position: object.position,
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
        let id = hash!("update_object_window");

        let mut object = self.object.clone().unwrap_or({
            let mut res = None;

            if let Some(layer) = map.layers.get(&self.layer_id) {
                res = layer.objects.get(self.index);
            }

            res.cloned().unwrap()
        });

        {
            let size = vec2(72.0, 28.0);

            let mut x_str = format!("{:.1}", object.position.x);
            let mut y_str = format!("{:.1}", object.position.y);

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

            object.position = vec2(x, y);
        }

        ComboBoxBuilder::new(hash!(id, "type_input"))
            .with_ratio(0.8)
            .with_label("Type")
            .build(ui, &mut object.kind);

        let item_ids = match object.kind {
            MapObjectKind::Item => items()
                .keys()
                .map(|k| k.as_str())
                .collect::<Vec<&str>>(),
            MapObjectKind::Environment => vec!["sproinger"],
            MapObjectKind::Decoration => iter_decoration()
                .map(|(k, _)| k.as_str())
                .collect::<Vec<&str>>(),
        };

        let mut item_id_value = {
            let index = item_ids
                .iter()
                .enumerate()
                .find_map(|(i, id)| if *id == object.id { Some(i) } else { None })
                .unwrap_or_default();

            ComboBoxVec::new(index, &item_ids)
        };

        ComboBoxBuilder::new(hash!("id_input"))
            .with_ratio(0.8)
            .with_label("Variant")
            .build(ui, &mut item_id_value);

        object.id = item_id_value.get_value();

        self.object = Some(object);

        None
    }
}
