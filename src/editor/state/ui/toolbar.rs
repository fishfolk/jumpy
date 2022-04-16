use crate::{
    editor::{
        actions::{UiAction, UiActionExt},
        state::EditorTool,
    },
    map::MapLayerKind,
};

use super::super::State;

impl State {
    pub(super) fn draw_toolbar(&mut self, egui_ctx: &egui::Context) {
        egui::SidePanel::new(egui::containers::panel::Side::Left, "Tools").show(egui_ctx, |ui| {
            let selected_layer_type = self.selected_layer_type();
            let mut add_tool = |tool_name, tool_variant| {
                if ui
                    .add(egui::SelectableLabel::new(
                        self.selected_tool == tool_variant,
                        tool_name,
                    ))
                    .clicked()
                {
                    self.apply_action(UiAction::SelectTool(tool_variant));
                }
            };

            add_tool("Cursor", EditorTool::Cursor);
            match selected_layer_type {
                Some(MapLayerKind::TileLayer) => {
                    add_tool("Tiles", EditorTool::TilePlacer);
                    add_tool("Eraser", EditorTool::Eraser);
                }
                Some(MapLayerKind::ObjectLayer) => add_tool("Objects", EditorTool::ObjectPlacer),
                None => (),
            }
        });
    }
}
