use macroquad::{
    prelude::*,
    ui::{hash, widgets, Ui},
};

use crate::map::{Map, MapLayerKind};

use super::{ButtonParams, EditorAction, EditorContext, Window, WindowParams};
use crate::editor::gui::ComboBoxBuilder;

pub struct CreateLayerWindow {
    params: WindowParams,
    id: String,
    kind: MapLayerKind,
    has_collision: bool,
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
            id: "Unnamed Layer".to_string(),
            kind: MapLayerKind::TileLayer,
            has_collision: false,
        }
    }
}

impl Window for CreateLayerWindow {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn draw(
        &mut self,
        ui: &mut Ui,
        _size: Vec2,
        _map: &Map,
        _ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let id = hash!("create_layer_window");

        {
            let size = vec2(173.0, 25.0);

            widgets::InputText::new(hash!(id, "name_input"))
                .size(size)
                .ratio(1.0)
                .label("Name")
                .ui(ui, &mut self.id);
        }

        ui.separator();
        ui.separator();
        ui.separator();
        ui.separator();

        ComboBoxBuilder::new(hash!(id, "type_input"))
            .with_label("Type")
            .with_ratio(0.8)
            .build(ui, &mut self.kind);

        if self.kind == MapLayerKind::TileLayer {
            widgets::Checkbox::new(hash!(id, "collision_input"))
                .ratio(0.4)
                .label("Collision")
                .ui(ui, &mut self.has_collision);
        }

        None
    }

    fn get_buttons(&self, map: &Map, _ctx: &EditorContext) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let is_existing_id = map.draw_order.iter().any(|id| id == &self.id);

        let mut action = None;
        if !is_existing_id {
            let batch = self.get_close_action().then(EditorAction::CreateLayer {
                id: self.id.clone(),
                kind: self.kind,
                has_collision: self.has_collision,
                index: None,
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
}

impl Default for CreateLayerWindow {
    fn default() -> Self {
        Self::new()
    }
}
