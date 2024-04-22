//! System for rendering player image in UI

use crate::prelude::*;

pub fn player_image(
    mut params: In<(&mut egui::Ui, &PlayerMeta, Option<&HatMeta>)>,
    egui_textures: Res<EguiTextures>,
    asset_server: Res<AssetServer>,
) {
    let (ui, player_meta, hat_meta) = &mut *params;
    let time = ui.ctx().input(|i| i.time as f32);
    let width = ui.available_width();
    let available_height = ui.available_width();

    let body_rect;
    let body_scale;
    let body_offset;
    let y_offset;
    // Render the body sprite
    {
        let atlas_handle = &player_meta.layers.body.atlas;
        let atlas = asset_server.get(*atlas_handle);
        let anim_clip = player_meta
            .layers
            .body
            .animations
            .frames
            .get(&ustr("idle"))
            .unwrap();
        let fps = anim_clip.fps;
        let frame_in_time_idx = (time * fps).round() as usize;
        let frame_in_clip_idx = frame_in_time_idx % anim_clip.frames.len();
        let frame_in_sheet_idx = anim_clip.frames[frame_in_clip_idx];
        let sprite_pos = atlas.tile_pos(frame_in_sheet_idx);
        body_offset =
            player_meta.layers.body.animations.offsets[&ustr("idle")][frame_in_clip_idx].body;

        let sprite_aspect = atlas.tile_size.y / atlas.tile_size.x;
        let height = sprite_aspect * width;
        y_offset = -(available_height - height) / 2.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

        let uv_min = sprite_pos / atlas.size();
        let uv_max = (sprite_pos + atlas.tile_size) / atlas.size();
        let uv = egui::Rect {
            min: egui::pos2(uv_min.x, uv_min.y),
            max: egui::pos2(uv_max.x, uv_max.y),
        };

        let mut mesh = egui::Mesh {
            texture_id: *egui_textures.0.get(&atlas.image).unwrap(),
            ..default()
        };

        mesh.add_rect_with_uv(rect, uv, egui::Color32::WHITE);
        mesh.translate(egui::vec2(0.0, y_offset));
        ui.painter().add(mesh);

        body_rect = rect;
        body_scale = width / atlas.tile_size.x;
    }

    // Render the fin & face animation
    for layer in [&player_meta.layers.fin, &player_meta.layers.face] {
        let atlas_handle = &layer.atlas;
        let atlas = asset_server.get(*atlas_handle);
        let anim_clip = layer.animations.get(&ustr("idle")).unwrap();
        let fps = anim_clip.fps;
        let frame_in_time_idx = (time * fps).round() as usize;
        let frame_in_clip_idx = frame_in_time_idx % anim_clip.frames.len();
        let frame_in_sheet_idx = anim_clip.frames[frame_in_clip_idx];
        let sprite_pos = atlas.tile_pos(frame_in_sheet_idx);

        let uv_min = sprite_pos / atlas.size();
        let uv_max = (sprite_pos + atlas.tile_size) / atlas.size();
        let uv = egui::Rect {
            min: egui::pos2(uv_min.x, uv_min.y),
            max: egui::pos2(uv_max.x, uv_max.y),
        };

        let mut mesh = egui::Mesh {
            texture_id: *egui_textures.0.get(&atlas.image).unwrap(),
            ..default()
        };

        let sprite_size = atlas.tile_size * body_scale;
        let offset = (layer.offset + body_offset) * body_scale;
        let rect = egui::Rect::from_center_size(
            body_rect.center() + egui::vec2(offset.x, -offset.y + y_offset),
            egui::vec2(sprite_size.x, sprite_size.y),
        );

        mesh.add_rect_with_uv(rect, uv, egui::Color32::WHITE);
        ui.painter().add(mesh);
    }

    // Render the player hat
    if let Some(hat_meta) = hat_meta {
        let atlas_handle = &hat_meta.atlas;
        let atlas = asset_server.get(*atlas_handle);
        let sprite_pos = Vec2::ZERO;

        let uv_min = sprite_pos / atlas.size();
        let uv_max = (sprite_pos + atlas.tile_size) / atlas.size();
        let uv = egui::Rect {
            min: egui::pos2(uv_min.x, uv_min.y),
            max: egui::pos2(uv_max.x, uv_max.y),
        };

        let mut mesh = egui::Mesh {
            texture_id: *egui_textures.0.get(&atlas.image).unwrap(),
            ..default()
        };

        let sprite_size = atlas.tile_size * body_scale;
        let offset = (hat_meta.offset + body_offset) * body_scale;
        let rect = egui::Rect::from_center_size(
            body_rect.center() + egui::vec2(offset.x, -offset.y + y_offset),
            egui::vec2(sprite_size.x, sprite_size.y),
        );

        mesh.add_rect_with_uv(rect, uv, egui::Color32::WHITE);
        ui.painter().add(mesh);
    }
}
