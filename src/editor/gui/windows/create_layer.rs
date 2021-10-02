use std::ops::Deref;

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
    map::{
        Map,
        MapLayerKind,
        ObjectLayerKind,
        CollisionKind,
    },
    Resources,
    editor::{
        EditorAction,
        EditorDrawParams,
    }
};

use super::{
    WindowPosition,
    WindowBuilder,
};

pub struct CreateLayerWindow {
    position: WindowPosition,
    size: Vec2,
    layer_id: String,
    layer_kind: MapLayerKind,
    layer_collision: CollisionKind,
}

impl CreateLayerWindow {
    pub fn new() -> Self {
        CreateLayerWindow {
            position: WindowPosition::Centered,
            size: vec2(350.0, 350.0),
            layer_id: "Unnamed Layer".to_string(),
            layer_kind: MapLayerKind::TileLayer,
            layer_collision: CollisionKind::None,
        }
    }

    pub fn get_rect(&self) -> Rect {
        let position = self.position.to_absolute(self.size);
        Rect::new(position.x, position.y, self.size.x, self.size.y)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        let rect = self.get_rect();
        rect.contains(point)
    }

    pub fn draw(&mut self, ui: &mut Ui, map: &Map, _params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        WindowBuilder::new(self.size)
            .with_position(self.position, true)
            .with_title("Create Layer")
            .build(ui, |ui| {
                {
                    let size = vec2(173.0, 25.0);

                    widgets::InputText::new(hash!())
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

                widgets::ComboBox::new(hash!(), &["Tiles", "Items", "Spawn Points"])
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

                    widgets::ComboBox::new(hash!(), &["None", "Solid"])
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
                    res = Some(EditorAction::CreateLayer {
                        id: self.layer_id.clone(),
                        kind: self.layer_kind,
                        draw_order_index: None,
                    });
                }

                ui.same_line(0.0);

                if ui.button(None, "Cancel") {
                    res = Some(EditorAction::CloseCreateLayerWindow);
                }
            });

        res
    }
}
