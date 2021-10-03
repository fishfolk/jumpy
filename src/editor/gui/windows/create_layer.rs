use macroquad::{
    ui::{
        Ui,
        hash,
        widgets,
    },
    prelude::*,
};

use crate::{
    map::{
        Map,
        MapLayerKind,
        ObjectLayerKind,
        CollisionKind,
    },
    editor::{
        EditorAction,
        EditorDrawParams,
    },
};

use super::{
    Window,
    WindowParams,
    WindowResult,
};

pub struct CreateLayerWindow {
    params: WindowParams,
    layer_id: String,
    layer_kind: MapLayerKind,
    layer_collision: CollisionKind,
}

impl CreateLayerWindow {
    pub fn new() -> Box<Self> {
        let params = WindowParams {
            title: Some("Create Layer".to_string()),
            size: vec2(350.0, 350.0),
            ..Default::default()
        };

        Box::new(CreateLayerWindow {
            params,
            layer_id: "Unnamed Layer".to_string(),
            layer_kind: MapLayerKind::TileLayer,
            layer_collision: CollisionKind::None,
        })
    }
}

impl Window for CreateLayerWindow {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }
    
    fn draw(&mut self, ui: &mut Ui, _size: Vec2, map: &Map, _params: &EditorDrawParams) -> Option<WindowResult> {
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
            MapLayerKind::ObjectLayer(kind) => {
                match kind {
                    ObjectLayerKind::Items => 1,
                    ObjectLayerKind::SpawnPoints => 2,
                    _ => unreachable!(),
                }
            }
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

        let is_existing_id = map.draw_order
            .iter()
            .find(|id| *id == &self.layer_id)
            .is_some();

        if is_existing_id {
            ui.label(None, "A layer with this name already exist!");
        } else {
            ui.label(None, "")
        }

        if ui.button(None, "Create") && is_existing_id == false {
            let action = EditorAction::CreateLayer {
                id: self.layer_id.clone(),
                kind: self.layer_kind,
                draw_order_index: None,
            };

            return Some(WindowResult::Action(action));
        }

        ui.same_line(0.0);

        if ui.button(None, "Cancel") {
            return Some(WindowResult::Cancel);
        }

        None
    }
}
