mod objects;

use macroquad::prelude::collections::storage;

use crate::{
    editor::{
        actions::UiAction,
        state::EditorTool,
        util::{EguiCompatibleVec, EguiTextureHandler, Resizable},
        view::LevelView,
    },
    Resources,
};

use super::super::Editor;

impl Editor {
    pub(super) fn draw_level(&mut self, egui_ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(egui_ctx, |ui| {
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

                self.draw_objects(egui_ctx, ui, &response, &painter);
                self.draw_level_overlays(egui_ctx, ui, &response, &painter);

                let (width, height) = (response.rect.width() as u32, response.rect.height() as u32);
                self.level_render_target.resize_if_appropiate(width, height);
            });
    }

    fn draw_level_overlays(
        &mut self,
        egui_ctx: &egui::Context,
        ui: &mut egui::Ui,
        level_response: &egui::Response,
        painter: &egui::Painter,
    ) {
        let level_contains_cursor = ui
            .input()
            .pointer
            .hover_pos()
            .map(|pos| level_response.rect.contains(pos))
            .unwrap_or(false);

        if level_contains_cursor {
            let tile_size = self.map_resource.map.tile_size.into_egui();

            let cursor_screen_pos = ui.input().pointer.interact_pos().unwrap();
            let cursor_px_pos = screen_to_world_pos(
                cursor_screen_pos,
                level_response.rect.min.to_vec2(),
                &self.level_view,
            );
            let cursor_tile_pos = (cursor_px_pos.to_vec2() / tile_size).floor().to_pos2();

            self.draw_level_placement_overlay(egui_ctx, level_response, painter, cursor_tile_pos);

            let level_top_left = level_response.rect.min;
            self.draw_level_pointer_pos_overlay(
                egui_ctx,
                ui,
                level_top_left,
                cursor_px_pos,
                cursor_tile_pos,
            );

            self.draw_level_object_placement_overlay(
                egui_ctx,
                level_response,
                painter,
                cursor_tile_pos,
            );
        }
    }

    fn draw_level_placement_overlay(
        &mut self,
        egui_ctx: &egui::Context,
        level_response: &egui::Response,
        painter: &egui::Painter,
        cursor_tile_pos: egui::Pos2,
    ) {
        match self.selected_tool {
            EditorTool::TilePlacer => self.draw_level_tile_placement_overlay(
                egui_ctx,
                level_response,
                painter,
                cursor_tile_pos,
            ),
            // TODO: Spawnpoint placement overlay
            // TODO: Object placement overlay
            _ => (),
        };
    }

    fn draw_level_tile_placement_overlay(
        &mut self,
        egui_ctx: &egui::Context,
        level_response: &egui::Response,
        painter: &egui::Painter,
        cursor_tile_pos: egui::Pos2,
    ) {
        let tile_size = self.map_resource.map.tile_size.into_egui();
        let level_top_left = level_response.rect.min;

        if cursor_tile_pos.x >= 0. && cursor_tile_pos.y >= 0. {
            if let (Some(selected_tile), Some(selected_layer)) =
                (&self.selected_tile, &self.selected_layer)
            {
                let tileset = &self.map_resource.map.tilesets[&selected_tile.tileset];
                let texture_id = storage::get::<Resources>().textures[&tileset.texture_id]
                    .texture
                    .egui_id();
                let tileset_uv_tile_size = egui::Vec2::splat(1.)
                    / egui::vec2(tileset.grid_size.x as f32, tileset.grid_size.y as f32);
                let tileset_x =
                    (selected_tile.tile_id % tileset.grid_size.x) as f32 * tileset_uv_tile_size.x;
                let tileset_y =
                    (selected_tile.tile_id / tileset.grid_size.x) as f32 * tileset_uv_tile_size.y;
                let uv = egui::Rect::from_min_size(
                    egui::Pos2 {
                        x: tileset_x,
                        y: tileset_y,
                    },
                    tileset_uv_tile_size,
                );

                let mut tile_mesh = egui::Mesh::with_texture(texture_id);
                tile_mesh.add_rect_with_uv(
                    egui::Rect::from_min_size(
                        world_to_screen_pos(
                            (cursor_tile_pos.to_vec2() * tile_size).to_pos2(),
                            level_top_left,
                            &self.level_view,
                        ),
                        tile_size,
                    ),
                    uv,
                    egui::Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 200),
                );

                painter.add(egui::Shape::mesh(tile_mesh));

                if level_response.clicked() || level_response.dragged() {
                    let input = egui_ctx.input();
                    if input.pointer.button_down(egui::PointerButton::Primary) {
                        let id = selected_tile.tile_id;
                        let layer_id = selected_layer.clone();
                        let tileset_id = selected_tile.tileset.clone();
                        self.apply_action(UiAction::PlaceTile {
                            id,
                            layer_id,
                            tileset_id,
                            coords: macroquad::math::UVec2::new(
                                cursor_tile_pos.x as u32,
                                cursor_tile_pos.y as u32,
                            ),
                        });
                    } else if input.pointer.button_down(egui::PointerButton::Secondary) {
                        let layer_id = selected_layer.clone();
                        self.apply_action(UiAction::RemoveTile {
                            layer_id,
                            coords: macroquad::math::UVec2::new(
                                cursor_tile_pos.x as u32,
                                cursor_tile_pos.y as u32,
                            ),
                        });
                    }
                }
            }
        }
    }

    fn draw_level_pointer_pos_overlay(
        &self,
        egui_ctx: &egui::Context,
        ui: &mut egui::Ui,
        level_top_left: egui::Pos2,
        cursor_px_pos: egui::Pos2,
        cursor_tile_pos: egui::Pos2,
    ) {
        egui::containers::Area::new("pointer pos overlay")
            .order(egui::Order::Tooltip)
            .fixed_pos(
                level_top_left
                    + egui::vec2(
                        ui.spacing().window_margin.left,
                        ui.spacing().window_margin.top,
                    ),
            )
            .interactable(false)
            .drag_bounds(egui::Rect::EVERYTHING) // disable clip rect
            .show(egui_ctx, |ui| {
                egui::Frame::popup(&egui_ctx.style())
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

// TODO: Factor in level view scale
fn screen_to_world_pos(
    p: egui::Pos2,
    level_top_left: egui::Vec2,
    level_view: &LevelView,
) -> egui::Pos2 {
    p - level_top_left + level_view.position.into_egui()
}

fn world_to_screen_pos(
    p: egui::Pos2,
    level_top_left: egui::Pos2,
    level_view: &LevelView,
) -> egui::Pos2 {
    p + level_top_left.to_vec2() - level_view.position.into_egui()
}
