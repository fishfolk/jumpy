use std::collections::HashMap;
use std::iter::FromIterator;
use std::ops::Div;

use serde::{Deserialize, Serialize};

use crate::color::{Color, colors};
use crate::math::{Size, UVec2, Vec2, Rect, vec2, AsVec2, AsUVec2};
use crate::rendering::{draw_rectangle_outline, draw_texture, DrawTextureParams};
use crate::storage;
use crate::texture::Texture2D;
use crate::transform::Transform;

/// Parameters for `Sprite` component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteMetadata {
    /// The id of the texture that will be used
    #[serde(rename = "texture")]
    pub texture_id: String,
    /// The sprites index in the sprite sheet
    #[serde(default)]
    pub index: usize,
    /// This is a scale factor that the sprite size will be multiplied by before draw
    #[serde(default)]
    pub scale: Option<f32>,
    /// The offset of the drawn sprite, relative to the position provided as an argument to the
    /// `Sprite` draw method.
    /// Note that this offset will not be inverted if the sprite is flipped.
    #[serde(default, with = "crate::json::vec2_def")]
    pub offset: Vec2,
    /// The pivot of the sprite, relative to the position provided as an argument to the `Sprite`
    /// draw method, plus any offset.
    /// Note that this offset will not be inverted if the sprite is flipped.
    #[serde(
        default,
        with = "crate::json::vec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub pivot: Option<Vec2>,
    /// The size of the drawn sprite. If no size is specified, the texture entry's `sprite_size`
    /// will be used, if specified, or the raw texture size, if not.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub size: Option<Size<f32>>,
    /// An optional color to blend with the texture color
    #[serde(
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tint: Option<Color>,
    /// If this is true, the sprite will not be drawn.
    #[serde(default)]
    pub is_deactivated: bool,
}

impl Default for SpriteMetadata {
    fn default() -> Self {
        SpriteMetadata {
            texture_id: "".to_string(),
            index: 0,
            scale: None,
            offset: Vec2::ZERO,
            pivot: None,
            size: None,
            tint: None,
            is_deactivated: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub texture: Texture2D,
    pub source_rect: Rect,
    pub tint: Color,
    pub scale: f32,
    pub offset: Vec2,
    pub pivot: Option<Vec2>,
    pub is_flipped_x: bool,
    pub is_flipped_y: bool,
    pub is_deactivated: bool,
}

impl Sprite {
    pub fn new(texture: Texture2D, params: SpriteParams) -> Self {
        let source_rect = {
            let sprite_size = params.sprite_size.unwrap_or_else(|| texture.size());

            let grid_size = Size::from(texture.size().as_vec2().as_uvec2() / sprite_size.as_vec2().as_uvec2());

            {
                let frame_cnt = (grid_size.width * grid_size.height) as usize;
                assert!(
                    params.index < frame_cnt,
                    "Sprite: index '{}' exceeds total frame count '{}'",
                    params.index,
                    frame_cnt
                );
            }

            let position = vec2(
                (params.index as u32 % grid_size.width) as f32 * sprite_size.width,
                (params.index as u32 / grid_size.width) as f32 * sprite_size.height,
            );

            Rect::new(position.x, position.y, sprite_size.width, sprite_size.height)
        };

        let tint = params.tint.unwrap_or(colors::WHITE);

        Sprite {
            texture,
            source_rect,
            tint,
            scale: params.scale,
            offset: params.offset,
            pivot: params.pivot,
            is_flipped_x: params.is_flipped_x,
            is_flipped_y: params.is_flipped_y,
            is_deactivated: params.is_deactivated,
        }
    }

    pub fn size(&self) -> Size<f32> {
        (self.source_rect.size() * self.scale).into()
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }
}

#[derive(Clone)]
pub struct SpriteParams {
    pub sprite_size: Option<Size<f32>>,
    pub index: usize,
    pub scale: f32,
    pub offset: Vec2,
    pub pivot: Option<Vec2>,
    pub size: Option<Size<f32>>,
    pub tint: Option<Color>,
    pub is_flipped_x: bool,
    pub is_flipped_y: bool,
    pub is_deactivated: bool,
}

impl Default for SpriteParams {
    fn default() -> Self {
        SpriteParams {
            sprite_size: None,
            index: 0,
            scale: 1.0,
            offset: Vec2::ZERO,
            pivot: None,
            size: None,
            tint: None,
            is_flipped_x: false,
            is_flipped_y: false,
            is_deactivated: false,
        }
    }
}

impl From<SpriteMetadata> for SpriteParams {
    fn from(meta: SpriteMetadata) -> Self {
        SpriteParams {
            index: meta.index,
            scale: meta.scale.unwrap_or(1.0),
            offset: meta.offset,
            pivot: meta.pivot,
            size: meta.size,
            tint: meta.tint,
            is_deactivated: meta.is_deactivated,
            ..Default::default()
        }
    }
}

pub fn draw_one_sprite(transform: &Transform, sprite: &Sprite) {
    if !sprite.is_deactivated {
        let size = sprite.size();

        draw_texture(
            transform.position.x + sprite.offset.x,
            transform.position.y + sprite.offset.y,
            sprite.texture,
            DrawTextureParams {
                flip_x: sprite.is_flipped_x,
                flip_y: sprite.is_flipped_y,
                rotation: transform.rotation,
                source: Some(sprite.source_rect),
                dest_size: Some(size),
                pivot: sprite.pivot,
                tint: Some(sprite.tint),
            },
        );
    }
}

pub fn debug_draw_one_sprite(position: Vec2, sprite: &Sprite) {
    if !sprite.is_deactivated {
        let size = sprite.size();

        draw_rectangle_outline(
            position.x + sprite.offset.x,
            position.y + sprite.offset.y,
            size.width,
            size.height,
            2.0,
            colors::BLUE,
        )
    }
}

#[derive(Debug)]
pub struct SpriteSet {
    pub draw_order: Vec<String>,
    pub map: HashMap<String, Sprite>,
}

impl From<&[(&str, Sprite)]> for SpriteSet {
    fn from(sprites: &[(&str, Sprite)]) -> Self {
        let draw_order = sprites.iter().map(|(id, _)| id.to_string()).collect();

        let map = HashMap::from_iter(
            sprites
                .iter()
                .map(|(id, sprite)| (id.to_string(), sprite.clone())),
        );

        SpriteSet { draw_order, map }
    }
}

impl SpriteSet {
    pub fn is_empty(&self) -> bool {
        self.draw_order.is_empty()
    }

    pub fn flip_all_x(&mut self, state: bool) {
        for sprite in self.map.values_mut() {
            sprite.is_flipped_x = state;
        }
    }

    pub fn flip_all_y(&mut self, state: bool) {
        for sprite in self.map.values_mut() {
            sprite.is_flipped_y = state;
        }
    }

    pub fn activate_all(&mut self) {
        for sprite in self.map.values_mut() {
            sprite.is_deactivated = false;
        }
    }

    pub fn deactivate_all(&mut self) {
        for sprite in self.map.values_mut() {
            sprite.is_deactivated = true;
        }
    }
}
