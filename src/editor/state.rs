use macroquad::prelude::{collections::storage, RenderTarget};

use crate::{
    map::MapLayerKind,
    resources::{MapResource, TextureResource},
    Resources,
};

use super::{
    actions::{UiAction, UiActionExt},
    util::{EguiCompatibleVec, EguiTextureHandler, Resizable},
};

// TODO: Factor in level view scale
fn screen_to_world_pos(
    p: egui::Pos2,
    level_top_left: egui::Vec2,
    level_view: &LevelView,
) -> egui::Pos2 {
    p - level_top_left + level_view.position.to_egui()
}

fn world_to_screen_pos(
    p: egui::Pos2,
    level_top_left: egui::Vec2,
    level_view: &LevelView,
) -> egui::Pos2 {
    p + level_top_left - level_view.position.to_egui()
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
    /// The view offset in pixels.
    pub position: macroquad::prelude::Vec2,
    /// The scale the level is viewed with. 1.0 == 1:1, bigger numbers mean bigger tiles.
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
        let mut action = None;

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(egui_ctx, |ui| {
                let texture_id = level_render_target.texture.egui_id();
                let level_img = egui::Image::new(texture_id, ui.available_size())
                    .sense(egui::Sense::click_and_drag());

                let response = ui.add(level_img);

                action.then_do(self.draw_level_overlays(egui_ctx, ui, &response, level_view));

                let (width, height) = (response.rect.width() as u32, response.rect.height() as u32);
                level_render_target.resize_if_appropiate(width, height);
            });

        action
    }

    fn draw_level_overlays(
        &self,
        egui_ctx: &egui::Context,
        ui: &mut egui::Ui,
        level_response: &egui::Response,
        level_view: &LevelView,
    ) -> Option<UiAction> {
        let action;

        if level_response.hovered() {
            let map = &self.map_resource.map;
            let tile_size = map.tile_size.to_egui();

            let cursor_screen_pos = ui.input().pointer.interact_pos().unwrap();
            let cursor_px_pos = screen_to_world_pos(
                cursor_screen_pos,
                level_response.rect.min.to_vec2(),
                level_view,
            );
            let cursor_tile_pos = (cursor_px_pos.to_vec2() / tile_size).floor().to_pos2();

            action = self.draw_level_placement_overlay(
                egui_ctx,
                level_response,
                cursor_tile_pos,
                level_view,
            );

            let level_top_left = level_response.rect.min;
            self.draw_level_pointer_pos_overlay(
                egui_ctx,
                ui,
                level_top_left,
                cursor_px_pos,
                cursor_tile_pos,
            );
        } else {
            action = None;
        }

        action
    }

    fn draw_level_placement_overlay(
        &self,
        egui_ctx: &egui::Context,
        level_response: &egui::Response,
        cursor_tile_pos: egui::Pos2,
        level_view: &LevelView,
    ) -> Option<UiAction> {
        let mut action = None;
        let map = &self.map_resource.map;
        let tile_size = map.tile_size.to_egui();
        let level_top_left = level_response.rect.min.to_vec2();

        if self.selected_tool == EditorTool::TilePlacer
            && cursor_tile_pos.x >= 0.
            && cursor_tile_pos.y >= 0.
        {
            match (&self.selected_tile, &self.selected_layer) {
                (Some(selected_tile), Some(selected_layer)) => {
                    egui::containers::Area::new("selected tile overlay")
                        .order(egui::Order::Background)
                        .interactable(false)
                        .fixed_pos(world_to_screen_pos(
                            (cursor_tile_pos.to_vec2() * tile_size).to_pos2(),
                            level_top_left,
                            level_view,
                        ))
                        .show(egui_ctx, |ui| {
                            let tileset = &map.tilesets[&selected_tile.tileset];
                            let texture_id = storage::get::<Resources>().textures
                                [&tileset.texture_id]
                                .texture
                                .egui_id();
                            let tileset_uv_tile_size = egui::Vec2::splat(1.)
                                / egui::vec2(
                                    tileset.grid_size.x as f32,
                                    tileset.grid_size.y as f32,
                                );
                            let tileset_x = (selected_tile.tile_id % tileset.grid_size.x) as f32
                                * tileset_uv_tile_size.x;
                            let tileset_y = (selected_tile.tile_id / tileset.grid_size.x) as f32
                                * tileset_uv_tile_size.y;
                            let uv = egui::Rect::from_min_size(
                                egui::Pos2 {
                                    x: tileset_x,
                                    y: tileset_y,
                                },
                                tileset_uv_tile_size,
                            );
                            let (response, painter) =
                                ui.allocate_painter(tile_size, egui::Sense::hover());
                            let mut tile_mesh = egui::Mesh::with_texture(texture_id);
                            tile_mesh.add_rect_with_uv(
                                response.rect,
                                uv,
                                egui::Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 200),
                            );

                            painter.add(egui::Shape::mesh(tile_mesh));
                        });

                    if level_response.clicked() || level_response.dragged() {
                        let input = egui_ctx.input();
                        if input.pointer.button_down(egui::PointerButton::Primary) {
                            action.then_do_some(UiAction::PlaceTile {
                                id: selected_tile.tile_id,
                                layer_id: selected_layer.clone(),
                                tileset_id: selected_tile.tileset.clone(),
                                coords: macroquad::math::UVec2::new(
                                    cursor_tile_pos.x as u32,
                                    cursor_tile_pos.y as u32,
                                ),
                            });
                        } else if input.pointer.button_down(egui::PointerButton::Secondary) {
                            action.then_do_some(UiAction::RemoveTile {
                                layer_id: selected_layer.clone(),
                                coords: macroquad::math::UVec2::new(
                                    cursor_tile_pos.x as u32,
                                    cursor_tile_pos.y as u32,
                                ),
                            });
                        }
                    }
                }

                _ => (),
            }
        }
        action
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
