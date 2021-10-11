use macroquad::{
    prelude::*,
    ui::{hash, widgets, Ui},
};

use crate::map::{Map, MapObjectKind};

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
                .label("Object ID")
                .ui(ui, &mut self.id);

            let mut object_kind = match self.kind {
                MapObjectKind::Item => 0,
                MapObjectKind::Weapon => 1,
                MapObjectKind::SpawnPoint => 2,
                MapObjectKind::Environment => 3,
                MapObjectKind::Decoration => 4,
            };

            widgets::ComboBox::new(hash!(id, "kind_input"), &["Item", "Weapon", "Spawn point", "Environment", "Decoration"])
                .ratio(0.4)
                .label("Object type")
                .ui(ui, &mut object_kind);

            self.kind = match object_kind {
                0 => MapObjectKind::Item,
                1 => MapObjectKind::Weapon,
                2 => MapObjectKind::SpawnPoint,
                3 => MapObjectKind::Environment,
                4 => MapObjectKind::Decoration,
                _ => unreachable!(),
            }
        }

        None
    }
}
