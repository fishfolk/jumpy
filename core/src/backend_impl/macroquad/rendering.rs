use macroquad::shapes::{draw_circle_lines, draw_rectangle_lines};
use macroquad::texture::draw_texture_ex;

use crate::color::{Color, colors};
use crate::math::{Vec2, Rect, Circle};
use crate::rendering::DrawTextureParams;
use crate::texture::Texture2D;

pub fn draw_texture(x: f32, y: f32, texture: Texture2D, params: DrawTextureParams) {
    let color = params.tint.unwrap_or(colors::WHITE).into();

    draw_texture_ex(texture.into(), x, y, color, params.into());
}

impl From<DrawTextureParams> for macroquad::texture::DrawTextureParams {
    fn from(params: DrawTextureParams) -> Self {
        macroquad::texture::DrawTextureParams {
            dest_size: params.dest_size,
            source: params.source,
            rotation: params.rotation,
            flip_x: params.flip_x,
            flip_y: params.flip_y,
            pivot: params.pivot,
        }
    }
}

pub fn draw_rectangle(x: f32, y: f32, width: f32, height: f32, color: Color) {
    macroquad::shapes::draw_rectangle(x, y, width, height, color.into());
}

pub fn draw_rectangle_outline(x: f32, y: f32, width: f32, height: f32, weight: f32, color: Color) {
    draw_rectangle_lines(x, y, width, height, weight, color.into());
}

pub fn draw_circle(x: f32, y: f32, radius: f32, color: Color) {
    macroquad::shapes::draw_circle(x, y, radius, color.into());
}

pub fn draw_circle_outline(x: f32, y: f32, radius: f32, weight: f32, color: Color) {
    draw_circle_lines(x, y, radius, weight, color.into());
}

pub fn draw_line(x: f32, y: f32, end_x: f32, end_y: f32, weight: f32, color: Color) {
    macroquad::shapes::draw_line(x, y, end_x, end_y, weight, color.into())
}