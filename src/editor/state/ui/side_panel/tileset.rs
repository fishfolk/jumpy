use macroquad::prelude::collections::storage;

use crate::{
    editor::{
        actions::{UiAction, UiActionExt},
        state::EditorTool,
        util::EguiTextureHandler,
    },
    resources::TextureResource,
    Resources,
};

use super::State;

impl State {
    pub(super) fn draw_tileset_info(&self, ui: &mut egui::Ui) -> Option<UiAction> {
        ui.heading("Tilesets");
        self.draw_tileset_list(ui) // Draw tileset list
            .then(self.draw_tileset_utils(ui)) // Draw tileset utils
            .then(self.draw_tileset_image(ui)) // Draw tileset image
    }

    fn draw_tileset_list(&self, ui: &mut egui::Ui) -> Option<UiAction> {
        let mut action = None;

        ui.group(|ui| {
            for (tileset_name, _tileset) in self.map_resource.map.tilesets.iter() {
                let is_selected = self
                    .selected_tile
                    .as_ref()
                    .map(|s| &s.tileset == tileset_name)
                    .unwrap_or(false);

                if ui.selectable_label(is_selected, tileset_name).clicked() {
                    action.then_do_some(UiAction::SelectTileset(tileset_name.clone()));
                }
            }
        });

        action
    }

    fn draw_tileset_utils(&self, ui: &mut egui::Ui) -> Option<UiAction> {
        let mut action = None;

        ui.horizontal(|ui| {
            if ui.button("+").clicked() {
                action.then_do_some(UiAction::OpenCreateTilesetWindow);
            }
            ui.add_enabled_ui(self.selected_tile.is_some(), |ui| {
                if ui.button("-").clicked() {
                    action.then_do_some(UiAction::DeleteTileset(
                        self.selected_tile.as_ref().unwrap().tileset.clone(),
                    ));
                }
                ui.button("Edit").on_hover_text("Does not do anything yet");
            })
        });

        action
    }

    fn draw_tileset_image(&self, ui: &mut egui::Ui) -> Option<UiAction> {
        let mut action = None;

        if let Some(selection) = &self.selected_tile {
            if let Some(tileset) = self.map_resource.map.tilesets.get(&selection.tileset) {
                let tileset_texture: &TextureResource =
                    &storage::get::<Resources>().textures[&tileset.texture_id];
                let texture_id = tileset_texture.texture.egui_id();
                let texture_size = tileset_texture.meta.size;
                let tile_size = egui::Vec2 {
                    x: tileset.tile_size.x,
                    y: tileset.tile_size.y,
                };
                let tileset_size = egui::Vec2 {
                    x: tileset.grid_size.x as f32,
                    y: tileset.grid_size.y as f32,
                };

                let image =
                    egui::Image::new(texture_id, egui::Vec2::new(texture_size.x, texture_size.y))
                        .sense(egui::Sense::click());
                let image_response = ui.add(image);
                let image_bounds = image_response.rect;

                if image_response.clicked() {
                    let mouse_pos = ui.input().pointer.interact_pos().unwrap() - image_bounds.min;
                    let tile_pos = (mouse_pos / tile_size).floor();
                    dbg!(&tile_pos);
                    if tile_pos.x < tileset_size.x
                        && tile_pos.y < tileset_size.y
                        && tile_pos.x >= 0.
                        && tile_pos.y >= 0.
                    {
                        let tile_id = tile_pos.x as u32 + tile_pos.y as u32 * tileset_size.x as u32;

                        action.then_do_some(UiAction::SelectTile {
                            id: tile_id,
                            tileset_id: selection.tileset.clone(),
                        });
                    }
                }

                if let EditorTool::TilePlacer = self.selected_tool {
                    let painter = ui.painter_at(image_bounds);
                    let tile_rect = egui::Rect::from_min_size(
                        image_bounds.min
                            + tile_size
                                * egui::Vec2::new(
                                    (selection.tile_id % tileset_size.x as u32) as f32,
                                    (selection.tile_id / tileset_size.x as u32) as f32,
                                ),
                        tile_size,
                    );
                    painter.rect_filled(
                        tile_rect,
                        egui::Rounding::none(),
                        egui::Color32::BLUE.linear_multiply(0.3),
                    );
                }
            }
        }

        action
    }
}
