use macroquad::{experimental::collections::storage, prelude::*};

use serde::{Deserialize, Serialize};

use crate::{json, Resources};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteParams {
    pub texture_id: String,
    #[serde(default, rename = "sprite_tint", with = "json::color_opt")]
    pub tint: Option<Color>,
    #[serde(default, rename = "sprite_offset", with = "json::vec2_opt")]
    pub offset: Option<Vec2>,
    #[serde(default, rename = "sprite_pivot", with = "json::vec2_opt")]
    pub pivot: Option<Vec2>,
    #[serde(default, rename = "sprite_source_rect", with = "json::rect_opt")]
    pub source_rect: Option<Rect>,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    texture: Texture2D,
    source_rect: Rect,
    pub tint: Color,
    pub offset: Vec2,
    pub pivot: Vec2,
    pub flip_x: bool,
    pub flip_y: bool,
    pub is_disabled: bool,
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

        let source_rect = params.source_rect.unwrap_or_else(|| {
            let sprite_size = texture_res
                .meta
                .sprite_size
                .map(|val| val.as_f32())
                .unwrap_or_else(|| vec2(texture_res.texture.width(), texture_res.texture.height()));

            Rect::new(0.0, 0.0, sprite_size.x as f32, sprite_size.y as f32)
        });

        let tint = params.tint.unwrap_or_default();
        let offset = params.offset.unwrap_or_default();
        let pivot = params.pivot.unwrap_or_default();

        Sprite {
            texture: texture_res.texture,
            source_rect,
            tint,
            offset,
            pivot,
            flip_x: false,
            flip_y: false,
            is_disabled: false,
        }
    }

    pub fn draw(&self, position: Vec2, rotation: f32, scale: Option<Vec2>) {
        let dest_size = if let Some(scale) = scale {
            let size = self.source_rect.size();
            vec2(size.x * scale.x, size.y * scale.y)
        } else {
            self.source_rect.size()
        };

        let params = DrawTextureParams {
            flip_x: self.flip_x,
            flip_y: self.flip_y,
            rotation,
            source: Some(self.source_rect),
            dest_size: Some(dest_size),
            pivot: Some(self.pivot),
        };

        draw_texture_ex(
            self.texture,
            position.x + self.offset.x,
            position.y + self.offset.y,
            self.tint,
            params,
        )
    }
}
