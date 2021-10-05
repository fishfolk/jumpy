use macroquad::{
    prelude::*,
    ui::{hash, widgets, Ui},
};

use crate::map::{CollisionKind, Map, MapLayerKind, ObjectLayerKind};

use super::{ButtonParams, EditorAction, EditorDrawParams, Window, WindowParams};

pub struct CreateLayerWindow {
    params: WindowParams,
    layer_id: String,
    layer_kind: MapLayerKind,
    layer_collision: CollisionKind,
}

impl CreateLayerWindow {
    pub fn new() -> Self {
        let params = WindowParams {
            title: Some("Create Layer".to_string()),
            size: vec2(350.0, 350.0),
            ..Default::default()
        };

        CreateLayerWindow {
            params,
            layer_id: "Unnamed Layer".to_string(),
            layer_kind: MapLayerKind::TileLayer,
            layer_collision: CollisionKind::None,
        }
    }
}

impl Window for CreateLayerWindow {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn get_buttons(&self, map: &Map, _draw_params: &EditorDrawParams) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let is_existing_id = map.draw_order.iter().any(|id| id == &self.layer_id);

        let mut action = None;
        if !is_existing_id {
            let batch = self.get_close_action().then(EditorAction::CreateLayer {
                id: self.layer_id.clone(),
                kind: self.layer_kind,
                draw_order_index: None,
            });

            action = Some(batch);
        }

        res.push(ButtonParams {
            label: "Create",
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
        _map: &Map,
        _params: &EditorDrawParams,
    ) -> Option<EditorAction> {
        let id = hash!("create_layer_window");

        {
            let size = vec2(173.0, 25.0);

            widgets::InputText::new(hash!(id, "name_input"))
                .size(size)
                .ratio(1.0)
                .label("Name")
                .ui(ui, &mut self.layer_id);
        }

        ui.separator();
        ui.separator();
        ui.separator();
        ui.separator();

        let mut layer_kind = match self.layer_kind {
            MapLayerKind::TileLayer => 0,
            MapLayerKind::ObjectLayer(kind) => match kind {
                ObjectLayerKind::Items => 1,
                ObjectLayerKind::SpawnPoints => 2,
                _ => unreachable!(),
            },
        };

        widgets::ComboBox::new(hash!(id, "type_input"), &["Tiles", "Items", "Spawn Points"])
            .ratio(0.4)
            .label("Type")
            .ui(ui, &mut layer_kind);

        self.layer_kind = match layer_kind {
            0 => MapLayerKind::TileLayer,
            1 => MapLayerKind::ObjectLayer(ObjectLayerKind::Items),
            2 => MapLayerKind::ObjectLayer(ObjectLayerKind::SpawnPoints),
            _ => unreachable!(),
        };

        if self.layer_kind == MapLayerKind::TileLayer {
            let mut collision = match self.layer_collision {
                CollisionKind::None => 0,
                CollisionKind::Solid => 1,
                CollisionKind::Barrier => 1,
            };

            widgets::ComboBox::new(hash!(id, "collision_input"), &["None", "Solid"])
                .ratio(0.4)
                .label("Collision")
                .ui(ui, &mut collision);

            self.layer_collision = match collision {
                0 => CollisionKind::None,
                _ => CollisionKind::Solid,
            };
        }

        None
    }
}

impl Default for CreateLayerWindow {
    fn default() -> Self {
        Self::new()
    }
}
