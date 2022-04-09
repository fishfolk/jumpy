mod layers;
mod tileset;

use crate::{
    editor::actions::{UiAction, UiActionExt},
    map::MapLayerKind,
};

use super::super::State;

impl State {
    pub(super) fn draw_side_panel(&self, egui_ctx: &egui::Context) -> Option<UiAction> {
        let mut action = None;

        egui::SidePanel::new(egui::containers::panel::Side::Right, "Side panel").show(
            egui_ctx,
            |ui| {
                action = self.draw_layer_info(ui);
                ui.separator();
                match self.selected_layer_type() {
                    Some(MapLayerKind::TileLayer) => {
                        action.then_do(self.draw_tileset_info(ui));
                    }
                    Some(MapLayerKind::ObjectLayer) => {
                        // TODO
                    }
                    None => (),
                }
            },
        );

        action
    }
}
