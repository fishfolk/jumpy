use crate::{
    editor::{
        actions::{UiAction, UiActionExt},
        state::EditorTool,
    },
    map::MapLayerKind,
};

use super::super::State;

impl State {
    pub(super) fn draw_toolbar(&self, egui_ctx: &egui::Context) -> Option<UiAction> {
        let mut action = None;

        egui::SidePanel::new(egui::containers::panel::Side::Left, "Tools").show(egui_ctx, |ui| {
            let tool = &self.selected_tool;

            let mut add_tool = |tool_name, tool_variant| {
                if ui
                    .add(egui::SelectableLabel::new(tool == &tool_variant, tool_name))
                    .clicked()
                {
                    action.then_do_some(UiAction::SelectTool(tool_variant));
                }
            };

            add_tool("Cursor", EditorTool::Cursor);
            match self.selected_layer_type() {
                Some(MapLayerKind::TileLayer) => {
                    add_tool("Tiles", EditorTool::TilePlacer);
                    add_tool("Eraser", EditorTool::Eraser);
                }
                Some(MapLayerKind::ObjectLayer) => add_tool("Objects", EditorTool::ObjectPlacer),
                None => (),
            }
        });

        action
    }
}
