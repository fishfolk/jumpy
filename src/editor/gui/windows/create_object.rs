use macroquad::{
    prelude::*,
    ui::{hash, widgets, Ui},
};

use crate::{
    editor::gui::ComboBoxBuilder,
    map::{Map, MapObjectKind},
};

use super::{ButtonParams, EditorAction, EditorContext, Window, WindowParams};

pub struct CreateObjectWindow {
    params: WindowParams,
    id: String,
    kind: MapObjectKind,
    position: Vec2,
    size: Option<Vec2>,
    layer_id: String,
}

impl CreateObjectWindow {
    pub fn new(position: Vec2, layer_id: String) -> Self {
        let params = WindowParams {
            title: Some("Create Object".to_string()),
            size: vec2(350.0, 350.0),
            ..Default::default()
        };

        CreateObjectWindow {
            params,
            id: "".to_string(),
            kind: MapObjectKind::Item,
            position,
            size: None,
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

        let action = self.get_close_action().then(EditorAction::CreateObject {
            id: self.id.clone(),
            kind: self.kind,
            position: self.position,
            size: self.size,
            layer_id: self.layer_id.clone(),
        });

        res.push(ButtonParams {
            label: "Create",
            action: Some(action),
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
        _map: &Map,
        _ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let id = hash!("create_object_window");

        {
            let size = vec2(173.0, 25.0);

            widgets::InputText::new(hash!(id, "id_input"))
                .size(size)
                .ratio(1.0)
                .label("ID")
                .ui(ui, &mut self.id);

            ComboBoxBuilder::new(hash!(id, "type_input"))
                .with_ratio(0.8)
                .with_label("Type")
                .build(ui, &mut self.kind);
        }

        None
    }
}
