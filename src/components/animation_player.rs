use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation as MQAnimation},
        collections::storage,
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

use crate::{json, Resources};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub id: String,
    pub row: u32,
    pub frames: u32,
    pub fps: u32,
}

impl From<Animation> for MQAnimation {
    fn from(a: Animation) -> Self {
        MQAnimation {
            name: a.id,
            row: a.row,
            frames: a.frames,
            fps: a.fps,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationParams {
    /// The id of the spritesheet texture that will be used
    #[serde(rename = "texture")]
    pub texture_id: String,
    /// The offset of the drawn frame, relative to the position provided as an argument to the
    /// `AnimationPlayer` draw method.
    /// Note that this offset will not be inverted if the frame is flipped.
    #[serde(default, with = "json::vec2_def")]
    pub offset: Vec2,
    /// The pivot of the frame, relative to the position provided as an argument to the
    /// `AnimationPlayer` draw method, plus any offset.
    /// Note that this offset will not be inverted if the frame is flipped.
    #[serde(default, with = "json::vec2_def")]
    pub pivot: Vec2,
    /// The size of the drawn sprite. If no size is specified, the texture entry's `sprite_size`
    /// will be used.
    #[serde(
        default,
        with = "json::uvec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub frame_size: Option<UVec2>,
    /// An optional color to blend with the texture color
    #[serde(
        default,
        with = "json::color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub tint: Option<Color>,
    /// A list of animations that will be available in the `AnimationPlayer`
    pub animations: Vec<Animation>,
    /// If this is true, the `AnimationPlayer` will automatically start playing its first animation.
    #[serde(default)]
    pub should_autoplay: bool,
    /// If this is true, the `AnimationPlayer` will not be updated or drawn.
    #[serde(default)]
    pub is_deactivated: bool,
}

impl Default for AnimationParams {
    fn default() -> Self {
        AnimationParams {
            texture_id: "".to_string(),
            offset: Vec2::ZERO,
            pivot: Vec2::ZERO,
            frame_size: None,
            tint: None,
            animations: vec![],
            should_autoplay: false,
            is_deactivated: false,
        }
    }
}

pub struct AnimationPlayer {
    texture: Texture2D,
    offset: Vec2,
    pivot: Vec2,
    tint: Color,
    sprite: AnimatedSprite,
    animations: Vec<Animation>,
    pub is_deactivated: bool,
}

impl AnimationPlayer {
    pub fn new(params: AnimationParams) -> Self {
        let resources = storage::get::<Resources>();
        let texture_resource = resources
            .textures
            .get(&params.texture_id)
            .unwrap_or_else(|| {
                panic!(
                    "AnimationPlayer: Invalid texture id '{}'",
                    &params.texture_id,
                )
            });

        let texture = texture_resource.texture;

        let frame_size = params.frame_size.unwrap_or_else(|| {
            texture_resource
                .meta
                .sprite_size
                .unwrap_or_else(|| vec2(texture.width(), texture.height()).as_u32())
        });

        let tint = params.tint.unwrap_or(color::WHITE);

        assert!(
            !params.animations.is_empty(),
            "AnimationPlayer: One or more animations are required"
        );

        let animations: Vec<MQAnimation> = {
            let mut ids = Vec::new();
            params
                .animations
                .clone()
                .into_iter()
                .map(|a| {
                    assert!(
                        !ids.contains(&a.id),
                        "AnimationPlayer: Invalid animation id '{}' (duplicate)",
                        &a.id
                    );
                    ids.push(a.id.clone());

                    let res: MQAnimation = a.into();
                    res
                })
                .collect()
        };

        let mut sprite = AnimatedSprite::new(
            frame_size.x,
            frame_size.y,
            &animations,
            !params.is_deactivated,
        );

        sprite.playing = params.should_autoplay;

        let animations = params.animations.to_vec();

        AnimationPlayer {
            texture,
            offset: params.offset,
            pivot: params.pivot,
            tint,
            sprite,
            animations,
            is_deactivated: params.is_deactivated,
        }
    }

    pub fn update(&mut self) {
        if !self.is_deactivated {
            self.sprite.update();
        }
    }

    pub fn draw(&self, position: Vec2, rotation: f32, flip_x: bool, flip_y: bool) {
        if !self.is_deactivated {
            let source_rect = self.sprite.frame().source_rect;
            let size = self.get_size();

            draw_texture_ex(
                self.texture,
                position.x + self.offset.x,
                position.y + self.offset.y,
                self.tint,
                DrawTextureParams {
                    flip_x,
                    flip_y,
                    rotation,
                    source: Some(source_rect),
                    dest_size: Some(size),
                    pivot: Some(self.pivot),
                },
            )
        }
    }

    #[cfg(debug_assertions)]
    pub fn debug_draw(&self, position: Vec2) {
        if crate::debug::is_debug_draw_enabled() && !self.is_deactivated {
            let size = self.get_size();

            draw_rectangle_lines(
                position.x + self.offset.x,
                position.y + self.offset.y,
                size.x,
                size.y,
                2.0,
                color::BLUE,
            )
        }
    }

    pub fn get_size(&self) -> Vec2 {
        self.sprite.frame().dest_size
    }

    pub fn get_animation(&self, id: &str) -> Option<&Animation> {
        self.animations.iter().find(|a| a.id == id)
    }

    // Set the current animation, using the animations id.
    // Will return a reference to the animation or `None`, if it doesn't exist
    pub fn set_animation(&mut self, id: &str) -> Option<&Animation> {
        let res = self.animations.iter().enumerate().find(|(_, a)| a.id == id);

        if let Some((i, animation)) = res {
            self.sprite.set_animation(i);
            return Some(animation);
        }

        None
    }

    // Set the frame of the current animation
    pub fn set_frame(&mut self, frame: usize) {
        self.sprite.set_frame(frame as u32);
    }

    pub fn play(&mut self) {
        self.sprite.playing = true;
    }

    pub fn stop(&mut self) {
        self.sprite.playing = false;
    }

    pub fn is_playing(&self) -> bool {
        !self.is_deactivated && self.sprite.playing
    }
}
