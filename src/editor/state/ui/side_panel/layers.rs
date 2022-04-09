use crate::{
    editor::{
        actions::{UiAction, UiActionExt},
        state::State,
    },
    map::MapLayerKind,
};

impl State {
    pub(super) fn draw_layer_info(&self, ui: &mut egui::Ui) -> Option<UiAction> {
        ui.heading("Layers");
        self.draw_layer_list(ui) // Draw layer list
            .then(self.draw_layer_utils(ui)) // Draw layer util buttons (+ - Up Down)
    }

    fn draw_layer_list(&self, ui: &mut egui::Ui) -> Option<UiAction> {
        let mut action = None;
        let map = &self.map_resource.map;

        ui.group(|ui| {
            for (layer_name, layer) in map.draw_order.iter().map(|id| (id, &map.layers[id])) {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(
                            self.selected_layer.as_ref() == Some(layer_name),
                            format!(
                                "({}) {}",
                                match layer.kind {
                                    MapLayerKind::TileLayer => "T",
                                    MapLayerKind::ObjectLayer => "O",
                                },
                                layer_name
                            ),
                        )
                        .clicked()
                    {
                        action.then_do_some(UiAction::SelectLayer(layer_name.clone()));
                    }
                    let mut is_visible = layer.is_visible;
                    if ui.checkbox(&mut is_visible, "Visible").clicked() {
                        action.then_do_some(UiAction::UpdateLayer {
                            id: layer_name.clone(),
                            is_visible,
                        });
                    }
                });
            }
        });

        action
    }

    fn draw_layer_utils(&self, ui: &mut egui::Ui) -> Option<UiAction> {
        let mut action = None;
        let map = &self.map_resource.map;

        ui.horizontal(|ui| {
            if ui.button("+").clicked() {
                action.then_do_some(UiAction::OpenCreateLayerWindow);
            }

            ui.add_enabled_ui(self.selected_layer.is_some(), |ui| {
                if ui.button("-").clicked() {
                    action.then_do_some(UiAction::DeleteLayer(
                        self.selected_layer.as_ref().unwrap().clone(),
                    ));
                }
                let selected_layer_idx = self
                    .selected_layer
                    .as_ref()
                    .and_then(|layer| {
                        map.draw_order
                            .iter()
                            .enumerate()
                            .find(|(_idx, id)| &layer == id)
                            .map(|(idx, _)| idx)
                    })
                    .unwrap_or(usize::MAX - 1);

                if ui
                    .add_enabled(selected_layer_idx > 0, egui::Button::new("Up"))
                    .clicked()
                {
                    action.then_do_some(UiAction::SetLayerDrawOrderIndex {
                        id: self.selected_layer.as_ref().unwrap().clone(),
                        index: selected_layer_idx - 1,
                    });
                }

                if ui
                    .add_enabled(
                        selected_layer_idx + 1 < map.draw_order.len(),
                        egui::Button::new("Down"),
                    )
                    .clicked()
                {
                    action.then_do_some(UiAction::SetLayerDrawOrderIndex {
                        id: self.selected_layer.as_ref().unwrap().clone(),
                        index: selected_layer_idx + 1,
                    });
                }
            });
        });

        action
    }
}
