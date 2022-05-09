mod objects;
mod tools;

use macroquad::prelude::collections::storage;

use crate::{
    editor::{
        actions::UiAction,
        state::{EditorTool, ObjectSettings},
        util::{EguiCompatibleVec, EguiTextureHandler, MqCompatibleVec, Resizable},
        view::UiLevelView,
    },
    map::{MapLayerKind, MapObjectKind},
    Resources,
};

use super::super::Editor;

impl Editor {
    pub(super) fn handle_level_view(&mut self, egui_ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(egui_ctx, |ui| {
                let mut view = self.draw_level_tiles(ui);
                let mut clicked_add_object = false;

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
                            _ => (),
                        }
                    });
                    if let Some(MapLayerKind::ObjectLayer) = self.selected_layer_type() {
                        if ui.button("Add object").clicked() {
                            clicked_add_object = true;
                            ui.close_menu()
                        }
                    }
                });

                if view.response.dragged_by(egui::PointerButton::Middle) {
                    let drag_delta = egui_ctx.input().pointer.delta();

                    // TODO: take level scale/"tiles per pixel" into account
                    self.level_view.position -= drag_delta.into_macroquad();
                }

                if clicked_add_object {
                    let position = view
                        .screen_to_world_pos(view.ctx().input().pointer.interact_pos().unwrap());

                    self.object_being_placed =
                        if let Some(settings) = self.object_being_placed.take() {
                            Some(ObjectSettings {
                                position,
                                ..settings
                            })
                        } else {
                            Some(ObjectSettings {
                                position,
                                kind: MapObjectKind::Item,
                                id: None,
                            })
                        };
                }

                self.handle_objects(ui, &view);
                self.handle_spawnpoints(&view);
                self.draw_level_overlays(ui, &view);

                let (width, height) = (
                    view.response.rect.width() as u32,
                    view.response.rect.height() as u32,
                );
                self.level_render_target.resize_if_appropiate(width, height);
            });
    }

    fn handle_spawnpoints(&mut self, view: &UiLevelView) {
        let texture = &storage::get::<Resources>().textures["spawn_point_icon"];
        let texture_id = texture.texture.egui_id();
        let texture_size = texture.meta.size.into_egui();

        for spawnpoint in self.map_resource.map.spawn_points.iter() {
            // This position is the bottom midpoint of the destination rect
            let pos = view.world_to_screen_pos(spawnpoint.into_egui().to_pos2());

            let dest = egui::Rect::from_min_size(
                pos - egui::vec2(texture_size.x / 2., texture_size.y),
                texture_size,
            );

            let mut mesh = egui::Mesh::with_texture(texture_id);
            mesh.add_rect_with_uv(
                dest,
                egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)),
                egui::Color32::WHITE,
            );
            view.painter().add(egui::Shape::mesh(mesh));
        }

        if self.selected_tool == EditorTool::SpawnPointPlacer && view.response.clicked() {
            let pos = view
                .screen_to_world_pos(view.ctx().input().pointer.interact_pos().unwrap())
                .to_vec2()
                .into_macroquad();
            self.apply_action(UiAction::CreateSpawnPoint(pos));
        }
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
            self.handle_tool(view, cursor_tile_pos);

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
