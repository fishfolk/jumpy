use macroquad::prelude::{collections::storage, render_target, RenderTarget};

use crate::{
    map::MapLayerKind,
    resources::{MapResource, TextureKind, TextureResource},
    Resources,
};

use super::actions::{UiAction, UiActionExt};

trait RectMap {
    fn map(self, from: egui::Rect, to: egui::Rect) -> egui::Rect;
}

impl RectMap for egui::Rect {
    fn map(self, from: egui::Rect, to: egui::Rect) -> egui::Rect {
        let min_origin_offset = from.min.to_vec2();
        let max_origin_offset = from.max.to_vec2();
        let min_target_offset = to.min.to_vec2();
        let max_target_offset = to.max.to_vec2();

        let min =
            (self.min - min_origin_offset).to_vec2() / from.size() * to.size() + min_target_offset;
        let max =
            (self.max - max_origin_offset).to_vec2() / from.size() * to.size() + max_target_offset;

        egui::Rect::from_min_max(min.to_pos2(), max.to_pos2())
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EditorTool {
    Cursor,
    TilePlacer,
    ObjectPlacer,
    SpawnPointPlacer,
    Eraser,
}

pub struct TileSelection {
    pub tileset: String,
    pub tile_id: u32,
}

/// Contains the editor state, i.e. the data whose change is tracked by the [`ActionHistory`] of the
/// editor.
pub struct EditorState {
    pub selected_tool: EditorTool,
    pub map_resource: MapResource,
    pub selected_layer: Option<String>,
    pub selected_tile: Option<TileSelection>,
    pub is_parallax_enabled: bool,
    pub should_draw_grid: bool,
}

impl EditorState {
    pub fn new(map_resource: MapResource) -> Self {
        Self {
            map_resource,
            selected_tool: EditorTool::Cursor,
            selected_layer: None,
            selected_tile: None,
            is_parallax_enabled: true,
            should_draw_grid: true,
        }
    }

    pub fn selected_layer_type(&self) -> Option<MapLayerKind> {
        self.selected_layer
            .as_ref()
            .and_then(|id| self.map_resource.map.layers.get(id))
            .map(|layer| layer.kind)
    }
}

// FIXME: This is very ugly, and shouldn't be passed into the editor state as parameter. Is there some
// better way to do this?
pub struct LevelView {
    pub position: macroquad::prelude::Vec2,
    pub scale: f32,
}

/// UI-related functions
impl EditorState {
    pub fn ui(
        &self,
        egui_ctx: &egui::Context,
        level_render_target: &mut RenderTarget,
        level_view: &LevelView,
    ) -> Option<UiAction> {
        self.draw_toolbar(egui_ctx)
            .then(self.draw_side_panel(egui_ctx))
            .then(self.draw_level(egui_ctx, level_render_target, level_view))
    }

    fn draw_level(
        &self,
        egui_ctx: &egui::Context,
        level_render_target: &mut RenderTarget,
        level_view: &LevelView,
    ) -> Option<UiAction> {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(egui_ctx, |ui| {
                let texture_id = egui::TextureId::User(
                    level_render_target
                        .texture
                        .raw_miniquad_texture_handle()
                        .gl_internal_id() as u64,
                );
                let image =
                    egui::Image::new(texture_id, ui.available_size()).sense(egui::Sense::click());
                let response = ui.add(image);
                if response.clicked() {
                    let cursor_pos = ui.input().pointer.interact_pos().unwrap();
                    let tile_size = egui::vec2(
                        self.map_resource.map.tile_size.x,
                        self.map_resource.map.tile_size.y,
                    );
                    let tile_pos =
                        (cursor_pos - response.rect.min) / tile_size + level_view.position;
                }

                let (width, height) = (ui.available_width() as u32, ui.available_height() as u32);
                if width != level_render_target.texture.width() as u32
                    || height != level_render_target.texture.height() as u32
                {
                    level_render_target.delete();
                    *level_render_target = render_target(width, height);
                    dbg!("Remade level render target");
                }
            });
        None
    }

    fn draw_toolbar(&self, egui_ctx: &egui::Context) -> Option<UiAction> {
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

    fn draw_side_panel(&self, egui_ctx: &egui::Context) -> Option<UiAction> {
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

    fn draw_layer_info(&self, ui: &mut egui::Ui) -> Option<UiAction> {
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

    fn draw_tileset_info(&self, ui: &mut egui::Ui) -> Option<UiAction> {
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
                let texture_id = egui::TextureId::User(
                    tileset_texture
                        .texture
                        .raw_miniquad_texture_handle()
                        .gl_internal_id() as u64,
                );
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
