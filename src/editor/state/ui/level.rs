use macroquad::prelude::{collections::storage, RenderTarget};

use crate::{
    editor::{
        actions::{UiAction, UiActionExt},
        state::EditorTool,
        util::{EguiCompatibleVec, EguiTextureHandler, Resizable},
        view::LevelView,
    },
    map::{MapObject, MapObjectKind},
    AnimatedSpriteMetadata, Resources, Sprite,
};

use super::super::State;

impl State {
    pub(super) fn draw_level(
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

                let (response, painter) =
                    ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
                let mut level_mesh = egui::Mesh::with_texture(texture_id);
                level_mesh.add_rect_with_uv(
                    response.rect,
                    egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)),
                    egui::Color32::WHITE,
                );
                painter.add(egui::Shape::mesh(level_mesh));

                action.then_do(self.draw_objects(egui_ctx, ui, &response, &painter, level_view));

                action.then_do(
                    self.draw_level_overlays(egui_ctx, ui, &response, &painter, level_view),
                );

                let (width, height) = (response.rect.width() as u32, response.rect.height() as u32);
                level_render_target.resize_if_appropiate(width, height);
            });

        action
    }

    fn draw_objects(
        &self,
        egui_ctx: &egui::Context,
        ui: &mut egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
        level_view: &LevelView,
    ) -> Option<UiAction> {
        const FULL_UV: egui::Rect =
            egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.));
        let mut action = None;

        let resources = storage::get::<Resources>();
        for layer_idx in self.map_resource.map.draw_order.iter() {
            let layer = if let Some(layer) = self.map_resource.map.layers.get(layer_idx) {
                layer
            } else {
                continue;
            };
            for (object_idx, object) in layer.objects.iter().enumerate() {
                let draw_object = |texture_id: egui::TextureId,
                                   offset: macroquad::math::Vec2,
                                   dest_size: egui::Vec2,
                                   uv: egui::Rect,
                                   tint: egui::Color32|
                 -> egui::Rect {
                    let position_in_lvl = (object.position + offset).into_egui().to_pos2();

                    let dest = egui::Rect::from_min_size(
                        world_to_screen_pos(position_in_lvl, response.rect.min, level_view),
                        dest_size,
                    );

                    let mut mesh = egui::Mesh::with_texture(texture_id);
                    mesh.add_rect_with_uv(dest, uv, tint);
                    painter.add(egui::Shape::mesh(mesh));
                    dest
                };

                let draw_invalid_object = || -> egui::Rect {
                    let texture_id = resources
                        .textures
                        .get("object_error_icon")
                        .unwrap()
                        .texture
                        .egui_id();
                    let dest_size = egui::vec2(32., 32.);
                    let uv = egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.));

                    draw_object(
                        texture_id,
                        (0., 0.).into(),
                        dest_size,
                        uv,
                        egui::Color32::WHITE,
                    )
                };

                let draw_animated_sprite = |sprite: &AnimatedSpriteMetadata,
                                            row: Option<u32>|
                 -> egui::Rect {
                    if let Some(texture_res) = resources.textures.get(&sprite.texture_id) {
                        let tint = sprite
                            .tint
                            .map(|color| {
                                let [r, g, b, a]: [u8; 4] = color.into();
                                egui::Color32::from_rgba_unmultiplied(r, g, b, a)
                            })
                            .unwrap_or(egui::Color32::WHITE);

                        let texture_id = texture_res.texture.egui_id();
                        let texture_size =
                            egui::vec2(texture_res.texture.width(), texture_res.texture.height());
                        let frame_size = texture_res.frame_size().into_egui();

                        let dest_size = sprite.scale.map(|s| s * frame_size).unwrap_or(frame_size);

                        let uv = row
                            .map(|row| {
                                egui::Rect::from_min_size(
                                    (egui::vec2(0.0, row as f32 * frame_size.y) / texture_size)
                                        .to_pos2(),
                                    frame_size / texture_size,
                                )
                            })
                            .unwrap_or(FULL_UV);

                        draw_object(texture_id, sprite.offset, dest_size, uv, tint)
                    } else {
                        // Invalid texture ID
                        draw_invalid_object()
                    }
                };

                let dest;
                let is_valid;
                match object.kind {
                    MapObjectKind::Decoration => {
                        if let Some(meta) = resources.decoration.get(&object.id) {
                            dest = draw_animated_sprite(
                                &meta.sprite,
                                meta.sprite.animations.first().map(|a| a.row),
                            );
                            is_valid = true;
                        } else {
                            // Invalid object ID
                            dest = draw_invalid_object();
                            is_valid = false;
                        }
                    }

                    MapObjectKind::Item => {
                        if let Some(meta) = resources.items.get(&object.id) {
                            dest = draw_animated_sprite(
                                &meta.sprite,
                                Some(
                                    meta.sprite
                                        .animations
                                        .iter()
                                        .find(|&a| a.id == *crate::player::IDLE_ANIMATION_ID)
                                        .map(|a| a.row)
                                        .unwrap_or_default(),
                                ),
                            );
                            is_valid = true;
                        } else {
                            // Invalid object ID
                            dest = draw_invalid_object();
                            is_valid = false;
                        }
                    }
                    MapObjectKind::Environment => {
                        if &object.id == "sproinger" {
                            let texture_res = resources.textures.get("sproinger").unwrap();

                            let texture_id = texture_res.texture.egui_id();
                            let texture_size = egui::vec2(
                                texture_res.texture.width(),
                                texture_res.texture.height(),
                            );
                            let dest_size = texture_res
                                .meta
                                .frame_size
                                .map(macroquad::math::Vec2::into_egui)
                                .unwrap_or_else(|| texture_size);
                            let uv = egui::Rect::from_min_size(
                                egui::Vec2::ZERO.to_pos2(),
                                texture_res.frame_size().into_egui() / texture_size,
                            );

                            dest = draw_object(
                                texture_id,
                                (0., 0.).into(),
                                dest_size,
                                uv,
                                egui::Color32::WHITE,
                            );
                            is_valid = true;
                        } else {
                            // Invalid object ID
                            dest = draw_invalid_object();
                            is_valid = false;
                        }
                    }
                };

                if self
                    .selected_entity
                    .as_ref()
                    .map(|selection| selection.kind.is_object(&layer.id, object_idx))
                    .unwrap_or(false)
                {
                    if response.dragged() {
                        painter.add(egui::Shape::rect_stroke(
                            dest,
                            egui::Rounding::none(),
                            egui::Stroke::new(2., egui::Color32::WHITE),
                        ));
                        let cursor_offset = self.selected_entity.as_ref().unwrap().click_offset;
                        let cursor_level_pos = screen_to_world_pos(
                            ui.input().pointer.interact_pos().unwrap() + cursor_offset,
                            response.rect.min.to_vec2(),
                            level_view,
                        );
                        action.then_do_some(UiAction::UpdateObject {
                            id: object.id.clone(),
                            index: object_idx,
                            kind: object.kind,
                            layer_id: layer.id.clone(),
                            position: macroquad::math::Vec2::new(
                                cursor_level_pos.x,
                                cursor_level_pos.y,
                            ),
                        });
                    } else if response.drag_released() {
                        action.then_do_some(UiAction::DeselectObject);
                    }
                } else if self.selected_tool == EditorTool::Cursor
                    && ui
                        .input()
                        .pointer
                        .hover_pos()
                        .map(|hover_pos| dest.contains(hover_pos))
                        .unwrap_or(false)
                {
                    if response.hovered() {
                        painter.add(egui::Shape::rect_stroke(
                            dest,
                            egui::Rounding::none(),
                            egui::Stroke::new(2., egui::Color32::GRAY),
                        ));
                        self.show_object_info_tooltip(egui_ctx, object, is_valid);
                    }
                    if response.drag_started() {
                        let click_pos = ui.input().pointer.interact_pos().unwrap();
                        action.then_do_some(UiAction::SelectObject {
                            index: object_idx,
                            layer_id: layer.id.clone(),
                            cursor_offset: dest.min - click_pos,
                        });
                    }
                }
            }
        }

        action
    }

    fn show_object_info_tooltip(
        &self,
        egui_ctx: &egui::Context,
        object: &MapObject,
        is_valid: bool,
    ) {
        egui::containers::show_tooltip_at_pointer(egui_ctx, egui::Id::new("object info"), |ui| {
            ui.set_max_width(200.);
            ui.vertical_centered(|ui| {
                ui.heading(&object.id);
                ui.label(egui::RichText::new(format!("{}", object.kind)).small());
            });
            ui.separator();
            ui.horizontal_top(|ui| {
                ui.label(egui::RichText::new("Position: ").weak());
                ui.label(
                    egui::RichText::new(format!("({}, {})", object.position.x, object.position.y))
                        .monospace(),
                );
            });
            if !is_valid {
                ui.label(
                    egui::RichText::new(
                        "Object is not valid (i.e. has no valid object or texture ID)",
                    )
                    .color(egui::Color32::RED),
                );
            }
        });
    }

    fn draw_level_overlays(
        &self,
        egui_ctx: &egui::Context,
        ui: &mut egui::Ui,
        level_response: &egui::Response,
        painter: &egui::Painter,
        level_view: &LevelView,
    ) -> Option<UiAction> {
        let action;

        if level_response.hovered() {
            let map = &self.map_resource.map;
            let tile_size = map.tile_size.into_egui();

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
                painter,
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
        painter: &egui::Painter,
        cursor_tile_pos: egui::Pos2,
        level_view: &LevelView,
    ) -> Option<UiAction> {
        let action;

        action = match self.selected_tool {
            EditorTool::TilePlacer => self.draw_level_tile_placement_overlay(
                egui_ctx,
                level_response,
                painter,
                cursor_tile_pos,
                level_view,
            ),
            // TODO: Spawnpoint placement overlay
            // TODO: Object placement overlay
            _ => None,
        };

        action
    }

    fn draw_level_tile_placement_overlay(
        &self,
        egui_ctx: &egui::Context,
        level_response: &egui::Response,
        painter: &egui::Painter,
        cursor_tile_pos: egui::Pos2,
        level_view: &LevelView,
    ) -> Option<UiAction> {
        let mut action = None;
        let map = &self.map_resource.map;
        let tile_size = map.tile_size.into_egui();
        let level_top_left = level_response.rect.min;

        if cursor_tile_pos.x >= 0. && cursor_tile_pos.y >= 0. {
            if let (Some(selected_tile), Some(selected_layer)) =
                (&self.selected_tile, &self.selected_layer)
            {
                let tileset = &map.tilesets[&selected_tile.tileset];
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
                            level_view,
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
