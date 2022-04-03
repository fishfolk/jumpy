use crate::{map::MapLayerKind, resources::MapResource};

use super::actions::{EditorAction, EditorActionExt};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EditorTool {
    Cursor,
    TilePlacer,
    ObjectPlacer,
    SpawnPointPlacer,
    Eraser,
}

pub struct EditorData {
    pub selected_tool: EditorTool,
    pub map_resource: MapResource,
    pub selected_layer: Option<String>,
    pub selected_tileset: Option<String>,
}

impl EditorData {
    pub fn new(map_resource: MapResource) -> Self {
        Self {
            map_resource,
            selected_tool: EditorTool::Cursor,
            selected_layer: None,
            selected_tileset: None,
        }
    }

    pub fn selected_layer_type(&self) -> Option<MapLayerKind> {
        self.selected_layer
            .as_ref()
            .and_then(|id| self.map_resource.map.layers.get(id))
            .map(|layer| layer.kind)
    }
}

/// UI-related functions
impl EditorData {
    pub fn ui(&self, egui_ctx: &egui::Context) -> Option<EditorAction> {
        // Draw toolbar
        let mut action = self.draw_toolbar(egui_ctx);
        // Draw side panel
        action.then_do(self.draw_side_panel(egui_ctx));

        action
    }

    fn draw_toolbar(&self, egui_ctx: &egui::Context) -> Option<EditorAction> {
        let mut action = None;

        egui::SidePanel::new(egui::containers::panel::Side::Left, "Tools").show(egui_ctx, |ui| {
            let tool = &self.selected_tool;

            let mut add_tool = |tool_name, tool_variant| {
                if ui
                    .add(egui::SelectableLabel::new(tool == &tool_variant, tool_name))
                    .clicked()
                {
                    action.then_do_some(EditorAction::SelectTool(tool_variant));
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

    fn draw_side_panel(&self, egui_ctx: &egui::Context) -> Option<EditorAction> {
        let mut action = None;

        egui::SidePanel::new(egui::containers::panel::Side::Right, "Side panel").show(
            egui_ctx,
            |ui| {
                action = self.draw_layer_info(ui);
                ui.separator();
                match self.selected_layer_type() {
                    Some(MapLayerKind::TileLayer) => {
                        self.draw_tileset_info(ui).map(|act| action = Some(act));
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

    fn draw_layer_info(&self, ui: &mut egui::Ui) -> Option<EditorAction> {
        ui.heading("Layers");
        self.draw_layer_list(ui) // Draw layer list
            .then(self.draw_layer_utils(ui)) // Draw layer util buttons (+ - Up Down)
    }

    fn draw_layer_list(&self, ui: &mut egui::Ui) -> Option<EditorAction> {
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
                        action.then_do_some(EditorAction::SelectLayer(layer_name.clone()));
                    }
                    let mut is_visible = layer.is_visible;
                    if ui.checkbox(&mut is_visible, "Visible").clicked() {
                        action.then_do_some(EditorAction::UpdateLayer {
                            id: layer_name.clone(),
                            is_visible,
                        });
                    }
                });
            }
        });

        action
    }

    fn draw_layer_utils(&self, ui: &mut egui::Ui) -> Option<EditorAction> {
        let mut action = None;
        let map = &self.map_resource.map;

        ui.horizontal(|ui| {
            if ui.button("+").clicked() {
                action.then_do_some(EditorAction::OpenCreateLayerWindow);
            }

            ui.add_enabled_ui(self.selected_layer.is_some(), |ui| {
                if ui.button("-").clicked() {
                    action.then_do_some(EditorAction::DeleteLayer(
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
                    action.then_do_some(EditorAction::SetLayerDrawOrderIndex {
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
                    action.then_do_some(EditorAction::SetLayerDrawOrderIndex {
                        id: self.selected_layer.as_ref().unwrap().clone(),
                        index: selected_layer_idx + 1,
                    });
                }
            });
        });

        action
    }

    fn draw_tileset_info(&self, ui: &mut egui::Ui) -> Option<EditorAction> {
        let mut action = None;

        ui.heading("Tilesets");
        ui.group(|ui| {
            for (tileset_name, _tileset) in self.map_resource.map.tilesets.iter() {
                let is_selected = self.selected_tileset.as_ref() == Some(tileset_name);

                if ui.selectable_label(is_selected, tileset_name).clicked() {
                    action.then_do_some(EditorAction::SelectTileset(tileset_name.clone()));
                }
            }
        });
        ui.horizontal(|ui| {
            ui.button("+");
            ui.button("-");
            ui.button("Edit");
        });

        action
    }
}
