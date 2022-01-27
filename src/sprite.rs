use std::collections::HashMap;
use std::iter::FromIterator;
use std::ops::Div;

use macroquad::color;
use macroquad::experimental::animation::Animation as MQAnimation;
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use hecs::World;

use serde::{Deserialize, Serialize};

use crate::json;
use crate::{Resources, Transform};

/// This is used to specify draw order on a sprite
/// This will be used, primarily, by `Player` to draw equipped items in the right order, relative
/// to its own sprite. This is done by multiplying the player id by ten and adding whatever offset
/// is required to this number, to order it relative to other sprites controlled by this specific
/// `Player` component.
pub struct DrawOrder(pub u32);

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
    #[serde(default, with = "json::vec2_def")]
    pub offset: Vec2,
    /// The pivot of the sprite, relative to the position provided as an argument to the `Sprite`
    /// draw method, plus any offset.
    /// Note that this offset will not be inverted if the sprite is flipped.
    #[serde(
        default,
        with = "json::vec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub pivot: Option<Vec2>,
    /// The size of the drawn sprite. If no size is specified, the texture entry's `sprite_size`
    /// will be used, if specified, or the raw texture size, if not.
    #[serde(
        default,
        with = "json::vec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub size: Option<Vec2>,
    /// An optional color to blend with the texture color
    #[serde(
        default,
        with = "json::color_opt",
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
    pub fn size(&self) -> Vec2 {
        self.source_rect.size() * self.scale
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }
}

impl From<SpriteMetadata> for Sprite {
    fn from(params: SpriteMetadata) -> Self {
        let texture_res = {
            let resources = storage::get::<Resources>();
            resources
                .textures
                .get(&params.texture_id)
                .cloned()
                .unwrap_or_else(|| panic!("Sprite: Invalid texture ID '{}'", &params.texture_id))
        };

        let source_rect = {
            let sprite_size = params.size.unwrap_or_else(|| texture_res.frame_size());

            let grid_size = texture_res.meta.size.div(sprite_size).as_u32();

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

        Sprite {
            texture: texture_res.texture,
            source_rect,
            tint,
            scale: params.scale.unwrap_or(1.0),
            offset: params.offset,
            pivot: params.pivot,
            is_flipped_x: false,
            is_flipped_y: false,
            is_deactivated: params.is_deactivated,
        }
    }
}

pub fn draw_one_sprite(transform: &Transform, sprite: &Sprite) {
    if !sprite.is_deactivated {
        let size = sprite.size();

        draw_texture_ex(
            sprite.texture,
            transform.position.x + sprite.offset.x,
            transform.position.y + sprite.offset.y,
            sprite.tint,
            DrawTextureParams {
                flip_x: sprite.is_flipped_x,
                flip_y: sprite.is_flipped_y,
                rotation: transform.rotation,
                source: Some(sprite.source_rect),
                dest_size: Some(size),
                pivot: sprite.pivot,
            },
        );
    }
}

pub fn debug_draw_one_sprite(position: Vec2, sprite: &Sprite) {
    if !sprite.is_deactivated {
        let size = sprite.size();

        draw_rectangle_lines(
            position.x + sprite.offset.x,
            position.y + sprite.offset.y,
            size.x,
            size.y,
            2.0,
            color::BLUE,
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
                .cloned()
                .map(|(id, sprite)| (id.to_string(), sprite)),
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

pub fn draw_sprites(world: &mut World) {
    let mut ordered = world
        .query::<&DrawOrder>()
        .iter()
        .map(|(e, order)| (e, order.0))
        .collect::<Vec<_>>();

    ordered.sort_by(|&(_, a), &(_, b)| a.cmp(&b));

    // This needs to be performance tested. If it causes any issues, we can work around having to
    // do all these queries by wrapping all drawables in a `Drawable` enum type and match on that
    // in stead.
    for e in ordered.into_iter().map(|(e, _)| e) {
        if let Ok(transform) = world.get::<Transform>(e) {
            if let Ok(sprite) = world.get::<Sprite>(e) {
                draw_one_sprite(&transform, &sprite);
            } else if let Ok(sprite_set) = world.get::<SpriteSet>(e) {
                for id in sprite_set.draw_order.iter() {
                    let sprite = sprite_set.map.get(id).unwrap();
                    draw_one_sprite(&transform, sprite);
                }
            } else if let Ok(sprite) = world.get::<AnimatedSprite>(e) {
                draw_one_animated_sprite(&transform, &sprite);
            } else if let Ok(sprite_set) = world.get::<AnimatedSpriteSet>(e) {
                for id in sprite_set.draw_order.iter() {
                    let sprite = sprite_set.map.get(id).unwrap();
                    draw_one_animated_sprite(&transform, sprite);
                }
            }
        }
    }
}

pub fn debug_draw_sprites(world: &mut World) {
    let mut ordered = world
        .query::<&DrawOrder>()
        .iter()
        .map(|(e, order)| (e, order.0))
        .collect::<Vec<_>>();

    ordered.sort_by(|&(_, a), &(_, b)| a.cmp(&b));

    // This needs to be performance tested. If it causes any issues, we can work around having to
    // do all these queries by wrapping all drawables in a `Drawable` enum type and match on that
    // in stead.
    for e in ordered.into_iter().map(|(e, _)| e) {
        if let Ok(transform) = world.get::<Transform>(e) {
            if let Ok(sprite) = world.get::<Sprite>(e) {
                debug_draw_one_sprite(transform.position, &sprite);
            } else if let Ok(sprite_set) = world.get::<SpriteSet>(e) {
                for id in sprite_set.draw_order.iter() {
                    let sprite = sprite_set.map.get(id).unwrap();
                    debug_draw_one_sprite(transform.position, sprite);
                }
            } else if let Ok(sprite) = world.get::<AnimatedSprite>(e) {
                debug_draw_one_animated_sprite(transform.position, &sprite);
            } else if let Ok(sprite_set) = world.get::<AnimatedSpriteSet>(e) {
                for id in sprite_set.draw_order.iter() {
                    let sprite = sprite_set.map.get(id).unwrap();
                    debug_draw_one_animated_sprite(transform.position, sprite);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub id: String,
    pub row: u32,
    pub frames: u32,
    pub fps: u32,
    pub is_looping: bool,
}

impl From<AnimationMetadata> for Animation {
    fn from(meta: AnimationMetadata) -> Self {
        Animation {
            id: meta.id,
            row: meta.row,
            frames: meta.frames,
            fps: meta.fps,
            is_looping: meta.is_looping,
        }
    }
}

pub struct AnimatedSpriteParams {
    pub scale: f32,
    pub offset: Vec2,
    pub pivot: Option<Vec2>,
    pub tint: Color,
    pub should_autoplay: bool,
}

impl Default for AnimatedSpriteParams {
    fn default() -> Self {
        AnimatedSpriteParams {
            scale: 1.0,
            offset: Vec2::ZERO,
            pivot: None,
            tint: color::WHITE,
            should_autoplay: true,
        }
    }
}

impl From<AnimatedSpriteMetadata> for AnimatedSpriteParams {
    fn from(meta: AnimatedSpriteMetadata) -> Self {
        AnimatedSpriteParams {
            scale: meta.scale.unwrap_or(1.0),
            offset: meta.offset,
            pivot: meta.pivot,
            tint: meta.tint.unwrap_or(color::WHITE),
            should_autoplay: meta.should_autoplay,
        }
    }
}

#[derive(Clone)]
pub enum QueuedAnimationAction {
    Play(String),
    PlayIndex(usize),
    Deactivate,
}

pub struct AnimatedSprite {
    pub texture: Texture2D,
    pub frame_size: Vec2,
    pub scale: f32,
    pub offset: Vec2,
    pub pivot: Option<Vec2>,
    pub tint: Color,
    pub animations: Vec<Animation>,
    pub current_index: usize,
    pub queued_action: Option<QueuedAnimationAction>,
    pub current_frame: u32,
    pub frame_timer: f32,
    pub is_playing: bool,
    pub is_flipped_x: bool,
    pub is_flipped_y: bool,
    pub is_deactivated: bool,
}

impl AnimatedSprite {
    pub fn new(
        texture: Texture2D,
        frame_size: Vec2,
        animations: &[Animation],
        params: AnimatedSpriteParams,
    ) -> Self {
        let animations = animations.to_vec();
        let is_playing = params.should_autoplay;

        AnimatedSprite {
            texture,
            frame_size,
            animations,
            scale: params.scale,
            offset: params.offset,
            pivot: params.pivot,
            tint: params.tint,
            frame_timer: 0.0,
            current_index: 0,
            queued_action: None,
            current_frame: 0,
            is_playing,
            is_flipped_x: false,
            is_flipped_y: false,
            is_deactivated: false,
        }
    }

    pub fn get_animation(&self, id: &str) -> Option<&Animation> {
        self.animations.iter().find(|&a| a.id == *id)
    }

    pub fn current_animation(&self) -> &Animation {
        self.animations.get(self.current_index).unwrap()
    }

    pub fn size(&self) -> Vec2 {
        self.frame_size * self.scale
    }

    pub fn source_rect(&self) -> Rect {
        let animation = self.animations.get(self.current_index).unwrap();

        Rect::new(
            self.current_frame as f32 * self.frame_size.x,
            animation.row as f32 * self.frame_size.y,
            self.frame_size.x,
            self.frame_size.y,
        )
    }

    pub fn as_index(&self, id: &str) -> Option<usize> {
        self.animations
            .iter()
            .enumerate()
            .find(|&(_, a)| a.id == *id)
            .map(|(i, _)| i)
    }

    pub fn set_animation_index(&mut self, index: usize, should_restart: bool) {
        if should_restart || self.current_index != index {
            self.current_index = index;
            self.current_frame = 0;
            self.frame_timer = 0.0;
            self.is_playing = true;
        }
    }

    pub fn set_animation(&mut self, id: &str, should_restart: bool) {
        if let Some(index) = self.as_index(id) {
            self.set_animation_index(index, should_restart);
        }
    }

    pub fn queue_action(&mut self, action: QueuedAnimationAction) {
        self.queued_action = Some(action);
    }

    pub fn restart(&mut self) {
        self.current_frame = 0;
        self.frame_timer = 0.0;
        self.is_playing = true;
    }
}

pub fn update_animated_sprites(world: &mut World) {
    for (_, sprite) in world.query_mut::<&mut AnimatedSprite>() {
        update_one_animated_sprite(sprite);
    }
}

pub fn update_one_animated_sprite(sprite: &mut AnimatedSprite) {
    if !sprite.is_deactivated && sprite.is_playing {
        let (is_last_frame, is_looping) = {
            let animation = sprite.animations.get(sprite.current_index).unwrap();
            (
                sprite.current_frame == animation.frames - 1,
                animation.is_looping,
            )
        };

        if is_last_frame {
            if let Some(action) = sprite.queued_action.take() {
                match &action {
                    QueuedAnimationAction::Play(id) => {
                        sprite.set_animation(id, false);
                    }
                    QueuedAnimationAction::PlayIndex(index) => {
                        sprite.set_animation_index(*index, false);
                    }
                    QueuedAnimationAction::Deactivate => {
                        sprite.is_deactivated = true;
                    }
                }
            } else {
                sprite.is_playing = is_looping;
            }
        }

        let (fps, frame_cnt) = {
            let animation = sprite.animations.get(sprite.current_index).unwrap();
            (animation.fps, animation.frames)
        };

        if sprite.is_playing {
            sprite.frame_timer += get_frame_time();

            if sprite.frame_timer > 1.0 / fps as f32 {
                sprite.current_frame += 1;
                sprite.frame_timer = 0.0;
            }
        }

        sprite.current_frame %= frame_cnt;
    }
}

pub fn draw_one_animated_sprite(transform: &Transform, sprite: &AnimatedSprite) {
    if !sprite.is_deactivated {
        let position = transform.position + sprite.offset;

        draw_texture_ex(
            sprite.texture,
            position.x,
            position.y,
            sprite.tint,
            DrawTextureParams {
                flip_x: sprite.is_flipped_x,
                flip_y: sprite.is_flipped_y,
                rotation: transform.rotation,
                source: Some(sprite.source_rect()),
                dest_size: Some(sprite.size()),
                pivot: sprite.pivot,
            },
        )
    }
}

pub fn debug_draw_one_animated_sprite(position: Vec2, sprite: &AnimatedSprite) {
    if !sprite.is_deactivated {
        let position = position + sprite.offset;
        let size = sprite.size();

        draw_rectangle_lines(position.x, position.y, size.x, size.y, 2.0, color::BLUE)
    }
}

#[derive(Default)]
pub struct AnimatedSpriteSet {
    pub draw_order: Vec<String>,
    pub map: HashMap<String, AnimatedSprite>,
}

impl AnimatedSpriteSet {
    pub fn is_empty(&self) -> bool {
        self.draw_order.is_empty()
    }

    pub fn size(&self) -> Vec2 {
        let mut size = Vec2::ZERO;

        for sprite in self.map.values() {
            let sprite_size = sprite.size();

            if sprite_size.x > size.x {
                size.x = sprite_size.x;
            }

            if sprite_size.y > size.y {
                size.y = sprite_size.y;
            }
        }

        size
    }

    pub fn set_animation(&mut self, sprite_id: &str, id: &str, should_restart: bool) {
        if let Some(sprite) = self.map.get_mut(sprite_id) {
            sprite.set_animation(id, should_restart);
        }
    }

    pub fn set_animation_index(&mut self, sprite_id: &str, index: usize, should_restart: bool) {
        if let Some(sprite) = self.map.get_mut(sprite_id) {
            sprite.set_animation_index(index, should_restart);
        }
    }

    pub fn set_queued_action(&mut self, sprite_id: &str, action: QueuedAnimationAction) {
        if let Some(sprite) = self.map.get_mut(sprite_id) {
            sprite.queue_action(action);
        }
    }

    pub fn set_all(&mut self, id: &str, should_restart: bool) {
        for sprite in self.map.values_mut() {
            sprite.set_animation(id, should_restart);
        }
    }

    pub fn set_all_to_index(&mut self, index: usize, should_restart: bool) {
        for sprite in self.map.values_mut() {
            sprite.set_animation_index(index, should_restart);
        }
    }

    pub fn queue_action_on_all(&mut self, action: QueuedAnimationAction) {
        for sprite in self.map.values_mut() {
            sprite.queue_action(action.clone());
        }
    }

    pub fn restart_all(&mut self) {
        for sprite in self.map.values_mut() {
            sprite.restart();
        }
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

    pub fn play_all(&mut self) {
        for sprite in self.map.values_mut() {
            sprite.is_playing = true;
        }
    }

    pub fn stop_all(&mut self) {
        for sprite in self.map.values_mut() {
            sprite.is_playing = false;
        }
    }
}

impl From<Vec<(&str, AnimatedSprite)>> for AnimatedSpriteSet {
    fn from(vec: Vec<(&str, AnimatedSprite)>) -> Self {
        let draw_order = vec.iter().map(|&(k, _)| k.to_string()).collect();

        let map = vec.into_iter().map(|(k, v)| (k.to_string(), v)).collect();

        AnimatedSpriteSet { draw_order, map }
    }
}

pub fn update_animated_sprite_sets(world: &mut World) {
    for (_, sprite_set) in world.query_mut::<&mut AnimatedSpriteSet>() {
        for key in &sprite_set.draw_order {
            let sprite = sprite_set.map.get_mut(key).unwrap();
            update_one_animated_sprite(sprite);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationMetadata {
    pub id: String,
    pub row: u32,
    pub frames: u32,
    pub fps: u32,
    #[serde(default)]
    pub is_looping: bool,
}

impl From<AnimationMetadata> for MQAnimation {
    fn from(a: AnimationMetadata) -> Self {
        MQAnimation {
            name: a.id,
            row: a.row,
            frames: a.frames,
            fps: a.fps,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedSpriteMetadata {
    #[serde(rename = "texture")]
    pub texture_id: String,
    #[serde(default)]
    pub scale: Option<f32>,
    #[serde(default, with = "json::vec2_def")]
    pub offset: Vec2,
    #[serde(
        default,
        with = "json::vec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub pivot: Option<Vec2>,
    #[serde(
        default,
        with = "json::color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub tint: Option<Color>,
    pub animations: Vec<AnimationMetadata>,
    #[serde(default, alias = "autoplay")]
    pub should_autoplay: bool,
    #[serde(default)]
    pub is_deactivated: bool,
}
