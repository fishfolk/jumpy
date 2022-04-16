use crate::{
    editor::{
        actions::{UiAction, UiActionExt},
        state::Editor,
    },
    map::MapLayerKind,
};

impl Editor {
    pub(super) fn draw_layer_info(&mut self, ui: &mut egui::Ui) {
        ui.heading("Layers");
        self.draw_layer_list(ui); // Draw layer list
        self.draw_layer_utils(ui); // Draw layer util buttons (+ - Up Down)
    }

    fn draw_layer_list(&mut self, ui: &mut egui::Ui) {
        let mut action = None;

        ui.group(|ui| {
            let map = &self.map_resource.map;
            for (layer_name, layer) in map.draw_order.iter().map(|id| (id, &map.layers[id])) {
                ui.horizontal(|ui| {
                    let layer_label = ui.selectable_label(
                        self.selected_layer.as_ref() == Some(layer_name),
                        format!(
                            "({}) {}",
                            match layer.kind {
                                MapLayerKind::TileLayer => "T",
                                MapLayerKind::ObjectLayer => "O",
                            },
                            layer_name
                        ),
                    );
                    if layer_label.clicked() {
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

        if let Some(action) = action {
            self.apply_action(action);
        }
    }

    fn draw_layer_utils(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("+").clicked() {
                self.create_layer_window = Some(Default::default());
            }

            ui.add_enabled_ui(self.selected_layer.is_some(), |ui| {
                if ui.button("-").clicked() {
                    let layer_id = self.selected_layer.as_ref().unwrap().clone();
                    self.apply_action(UiAction::DeleteLayer(layer_id));
                }
                let selected_layer_idx = self
                    .selected_layer
                    .as_ref()
                    .and_then(|layer| {
                        self.map_resource
                            .map
                            .draw_order
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
                    let id = self.selected_layer.as_ref().unwrap().clone();
                    self.apply_action(UiAction::SetLayerDrawOrderIndex {
                        id,
                        index: selected_layer_idx - 1,
                    });
                }

                if ui
                    .add_enabled(
                        selected_layer_idx + 1 < self.map_resource.map.draw_order.len(),
                        egui::Button::new("Down"),
                    )
                    .clicked()
                {
                    let id = self.selected_layer.as_ref().unwrap().clone();
                    self.apply_action(UiAction::SetLayerDrawOrderIndex {
                        id,
                        index: selected_layer_idx + 1,
                    });
                }
            });
        });
    }
}
