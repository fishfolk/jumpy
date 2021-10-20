use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

use crate::{json, Resources};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationParams {
    pub texture_id: String,
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
    pub frame_size: Option<UVec2>,
    #[serde(
        default,
        with = "json::color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub tint: Option<Color>,
    #[serde(with = "json::animation_vec")]
    pub animations: Vec<Animation>,
    #[serde(default)]
    pub should_autoplay: bool,
}

pub struct AnimationPlayer {
    texture: Texture2D,
    offset: Vec2,
    pivot: Vec2,
    tint: Color,
    sprite: AnimatedSprite,
    animations: Vec<Animation>,
}

impl AnimationPlayer {
    pub fn new(params: AnimationParams) -> Self {
        let resources = storage::get::<Resources>();
        let texture_resource = resources
            .textures
            .get(&params.texture_id)
            .unwrap_or_else(|| {
                panic!(
                    "AnimationPlayer: Invalid texture ID '{}'",
                    &params.texture_id
                )
            });
        let texture = texture_resource.texture;

        let offset = params.offset.unwrap_or(Vec2::ZERO);

        let pivot = params.pivot.unwrap_or(Vec2::ZERO);

        let frame_size = params.frame_size.unwrap_or_else(|| {
            texture_resource
                .meta
                .sprite_size
                .unwrap_or_else(|| vec2(texture.width(), texture.height()).as_u32())
        });

        let tint = params.tint.unwrap_or(color::WHITE);

        let sprite = AnimatedSprite::new(
            frame_size.x,
            frame_size.y,
            &params.animations,
            params.should_autoplay,
        );

        let animations = params.animations.to_vec();

        AnimationPlayer {
            texture,
            offset,
            pivot,
            tint,
            sprite,
            animations,
        }
    }

    pub fn update(&mut self) {
        self.sprite.update();
    }

    pub fn draw(
        &self,
        position: Vec2,
        rotation: f32,
        scale: Option<Vec2>,
        flip_x: bool,
        flip_y: bool,
    ) {
        let source_rect = self.sprite.frame().source_rect;
        let rect = self.get_rect(scale);

        let pivot = {
            let size= self.get_size(scale);
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
                source: Some(source_rect),
                dest_size: Some(rect.size()),
                pivot: Some(pivot),
            },
        );
    }

    pub fn get_size(&self, scale: Option<Vec2>) -> Vec2 {
        let size = self.sprite.frame().dest_size;
        if let Some(scale) = scale {
            vec2(size.x * scale.x, size.y * scale.y)
        } else {
            size
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

    pub fn get_animation(&self, animation: usize) -> Option<&Animation> {
        self.animations.get(animation)
    }

    pub fn set_animation(&mut self, animation: usize) {
        self.sprite.set_animation(animation);
    }

    pub fn set_frame(&mut self, frame: usize) {
        self.sprite.set_frame(frame as u32);
    }
}
