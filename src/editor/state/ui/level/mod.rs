mod objects;
mod tools;

use crate::{
    editor::{
        state::EditorTool,
        util::{EguiCompatibleVec, EguiTextureHandler, Resizable},
        view::UiLevelView,
    },
    map::MapLayerKind,
};

use super::super::Editor;

impl Editor {
    pub(super) fn handle_level_view(&mut self, egui_ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(egui_ctx, |ui| {
                let mut view = self.draw_level_tiles(ui);

                view.response = view.response.context_menu(|ui| {
                    ui.menu_button("Select tool", |ui| {
                        if ui.button("Cursor").clicked() {
                            self.selected_tool = EditorTool::Cursor;
                            ui.close_menu()
                        }
                        if ui.button("Spawnpoint Placer").clicked() {
                            self.selected_tool = EditorTool::SpawnPointPlacer;
                            ui.close_menu()
                        }
                        match self.selected_layer_type() {
                            Some(MapLayerKind::TileLayer) => {
                                if ui.button("Tile Placer").clicked() {
                                    self.selected_tool = EditorTool::TilePlacer;
                                    ui.close_menu()
                                }
                                if ui.button("Eraser").clicked() {
                                    self.selected_tool = EditorTool::Eraser;
                                    ui.close_menu()
                                }
                            }
                            Some(MapLayerKind::ObjectLayer) => {
                                if ui.button("Object Placer").clicked() {
                                    self.selected_tool = EditorTool::ObjectPlacer;
                                    ui.close_menu()
                                }
                            }
                            None => (),
                        }
                    });
                });

                self.handle_objects(ui, &view);
                self.draw_level_overlays(ui, &view);

                let (width, height) = (
                    view.response.rect.width() as u32,
                    view.response.rect.height() as u32,
                );
                self.level_render_target.resize_if_appropiate(width, height);
            });
    }

    fn draw_level_tiles(&self, ui: &mut egui::Ui) -> UiLevelView {
        let texture_id = self.level_render_target.texture.egui_id();

        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let mut level_mesh = egui::Mesh::with_texture(texture_id);
        level_mesh.add_rect_with_uv(
            response.rect,
            egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)),
            egui::Color32::WHITE,
        );
        painter.add(egui::Shape::mesh(level_mesh));

        UiLevelView::new(self.level_view, response, painter)
    }

    fn draw_level_overlays(&mut self, ui: &mut egui::Ui, view: &UiLevelView) {
        let level_contains_cursor = ui
            .input()
            .pointer
            .hover_pos()
            .map(|pos| view.response.rect.contains(pos))
            .unwrap_or(false);

        if level_contains_cursor {
            let tile_size = self.map_resource.map.tile_size.into_egui();

            let cursor_screen_pos = ui.input().pointer.interact_pos().unwrap();
            let cursor_px_pos = view.screen_to_world_pos(cursor_screen_pos);
            let cursor_tile_pos = (cursor_px_pos.to_vec2() / tile_size).floor().to_pos2();

            // TODO: Move outside
            self.handle_tool(&view, cursor_tile_pos);

            self.draw_level_pointer_pos_overlay(ui, view, cursor_px_pos, cursor_tile_pos);

            self.draw_level_object_placement_overlay(view);
        }
    }

    fn draw_level_pointer_pos_overlay(
        &self,
        ui: &mut egui::Ui,
        view: &UiLevelView,
        cursor_px_pos: egui::Pos2,
        cursor_tile_pos: egui::Pos2,
    ) {
        egui::containers::Area::new("pointer pos overlay")
            .order(egui::Order::Tooltip)
            .fixed_pos(
                view.level_top_left()
                    + egui::vec2(
                        ui.spacing().window_margin.left,
                        ui.spacing().window_margin.top,
                    ),
            )
            .interactable(false)
            .drag_bounds(egui::Rect::EVERYTHING) // disable clip rect
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(&ui.ctx().style())
                    .show(ui, |ui| {
                        ui.label(format!(
                            "Cursor position: ({}, {}) in pixels: ({:.2}, {:.2})",
                            cursor_tile_pos.x, cursor_tile_pos.y, cursor_px_pos.x, cursor_px_pos.y,
                        ))
                    })
                    .inner
            });
    }
}
