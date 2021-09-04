//! Spritesheets for guns in the fishgame made like this:
//! ┌──────────────────────────────────┐
//! │                                  │         
//! │GUN      lots of emtpy space      │ H  
//! │                                  │         
//! └──────────────────────────────────┘
//!                  W
//! yeah, its a WxH box where gun is aligned to the left
//!
//! and each gun have a collider, something GUN sized, way less than W
//! so, when fish turns left and we flip this picture by X axis
//! GUN goes all the way to the right
//! and collider, being smaller, do not really aligns with a GUN animore
//! Solution - we believe, that collider size is pretty much the same as a
//! GUN width. So we move flipped texture the way G in GUN is always near the
//! gun mountpoint, being rotated either left or right.
//! (I do not know if this explanations makes any sense whatsoever :/)

use macroquad::{
    color,
    experimental::animation::AnimatedSprite,
    math::Vec2,
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
};

pub struct GunlikeAnimation {
    sprite: AnimatedSprite,
    spritesheet: Texture2D,
    collider_width: f32,
}

impl GunlikeAnimation {
    pub fn new(
        sprite: AnimatedSprite,
        spritesheet: Texture2D,
        collider_width: f32,
    ) -> GunlikeAnimation {
        GunlikeAnimation {
            sprite,
            spritesheet,
            collider_width,
        }
    }

    pub fn update(&mut self) {
        self.sprite.update();
    }

    pub fn draw(&self, pos: Vec2, facing: bool, angle: f32) {
        let w = self.sprite.frame().source_rect.w;

        draw_texture_ex(
            self.spritesheet,
            pos.x + if !facing { self.collider_width - w } else { 0. },
            pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(self.sprite.frame().source_rect),
                dest_size: Some(self.sprite.frame().dest_size),
                flip_x: !facing,
                rotation: angle,
                ..Default::default()
            },
        );
    }

    pub fn set_animation(&mut self, animation: usize) {
        self.sprite.set_animation(animation);
    }

    pub fn set_frame(&mut self, frame: u32) {
        self.sprite.set_frame(frame);
    }
}
