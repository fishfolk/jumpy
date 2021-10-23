use macroquad::{color, experimental::collections::storage, prelude::*};

use serde::{Deserialize, Serialize};

use crate::{json, Resources};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteParams {
    pub texture_id: String,
    #[serde(default)]
    pub index: usize,
    #[serde(
        default,
        with = "json::color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub tint: Option<Color>,
    #[serde(
        default,
        with = "json::vec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub offset: Option<Vec2>,
    #[serde(
        default,
        with = "json::vec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub pivot: Option<Vec2>,
    #[serde(
        default,
        with = "json::uvec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub size: Option<UVec2>,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    texture: Texture2D,
    source_rect: Rect,
    tint: Color,
    offset: Vec2,
    pivot: Vec2,
}

impl Sprite {
    pub fn new(params: SpriteParams) -> Self {
        let texture_res = {
            let resources = storage::get::<Resources>();
            resources
                .textures
                .get(&params.texture_id)
                .cloned()
                .unwrap_or_else(|| panic!("Sprite: Invalid texture ID '{}'", &params.texture_id))
        };

        let source_rect = {
            let sprite_size = params.size.map(|uvec| uvec.as_f32()).unwrap_or_else(|| {
                texture_res
                    .meta
                    .sprite_size
                    .map(|val| val.as_f32())
                    .unwrap_or_else(|| {
                        vec2(texture_res.texture.width(), texture_res.texture.height())
                    })
            });

            let grid_size = uvec2(
                (texture_res.texture.width() / sprite_size.x) as u32,
                (texture_res.texture.height() / sprite_size.y) as u32,
            );

            {
                let frame_cnt = (grid_size.x * grid_size.y) as usize;
                assert!(
                    params.index < frame_cnt,
                    "Sprite: index '{}' exceeds total frame count '{}'",
                    params.index,
                    frame_cnt
                );
            }

            let position = vec2(
                (params.index as u32 % grid_size.x) as f32 * sprite_size.x,
                (params.index as u32 / grid_size.x) as f32 * sprite_size.y,
            );

            Rect::new(position.x, position.y, sprite_size.x, sprite_size.y)
        };

        let tint = params.tint.unwrap_or(color::WHITE);
        let offset = params.offset.unwrap_or_default();
        let pivot = params.pivot.unwrap_or_default();

        Sprite {
            texture: texture_res.texture,
            source_rect,
            tint,
            offset,
            pivot,
        }
    }

    pub fn draw(
        &self,
        position: Vec2,
        rotation: f32,
        scale: Option<Vec2>,
        flip_x: bool,
        flip_y: bool,
    ) {
        let rect = self.get_rect(scale);

        let pivot = {
            let size = self.get_size(scale);
            let mut pivot = self.pivot;
            if flip_x {
                pivot.x = size.x - self.pivot.x;
            }
            if flip_y {
                pivot.y = size.y - self.pivot.y;
            }

            pivot
        };

        draw_texture_ex(
            self.texture,
            position.x + rect.x,
            position.y + rect.y,
            self.tint,
            DrawTextureParams {
                flip_x,
                flip_y,
                rotation,
                source: Some(self.source_rect),
                dest_size: Some(rect.size()),
                pivot: Some(pivot),
            },
        );

        // draw_rectangle_lines(
        //     position.x + rect.x,
        //     position.y + rect.y,
        //     rect.w,
        //     rect.h,
        //     2.0,
        //     color::BLUE,
        // )
    }

    pub fn get_size(&self, scale: Option<Vec2>) -> Vec2 {
        if let Some(scale) = scale {
            let size = self.source_rect.size();
            vec2(size.x * scale.x, size.y * scale.y)
        } else {
            self.source_rect.size()
        }
    }

    pub fn get_rect(&self, scale: Option<Vec2>) -> Rect {
        let position = if let Some(scale) = scale {
            vec2(self.offset.x * scale.x, self.offset.y * scale.y)
        } else {
            self.offset
        };

        let size = self.get_size(scale);

        Rect::new(position.x, position.y, size.x, size.y)
    }
}
