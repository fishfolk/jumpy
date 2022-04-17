use crate::macroquad::ui::{root_ui, widgets, Ui};
use crate::math::{Size, Vec2};
use crate::prelude::{draw_texture, viewport};
use crate::rendering::DrawTextureParams;
use crate::resources::get_texture;

use crate::resources::TextureResource;
use crate::storage;
use crate::window::window_size;

pub struct Background {
    textures: Vec<TextureResource>,
    size: Size<f32>,
    position: Vec2,
}

impl Background {
    pub fn new(size: Size<f32>, position: Vec2, textures: &[TextureResource]) -> Self {
        let textures = textures.to_vec();

        Background {
            textures,
            size,
            position,
        }
    }

    #[cfg(feature = "macroquad-backend")]
    pub fn ui(&self, ui: &mut Ui) {
        for res in &self.textures {
            widgets::Texture::new(res.texture.into())
                .size(self.size.width, self.size.height)
                .position(self.position)
                .ui(ui);
        }
    }

    pub fn draw(&self) {
        for texture_res in &self.textures {
            draw_texture(
                self.position.x,
                self.position.y,
                texture_res.texture,
                DrawTextureParams {
                    dest_size: Some(self.size),
                    ..Default::default()
                },
            )
        }
    }
}

pub fn draw_main_menu_background(is_ui: bool) {
    let backgrounds = [
        get_texture("background_01"),
        get_texture("background_02"),
        get_texture("background_03"),
        get_texture("background_04"),
    ];

    let size = backgrounds[0].texture.size();

    let height = viewport().height as f32;
    let width = (height / size.height) * size.width;

    let bg = Background::new(
        Size::new(width, height),
        Vec2::ZERO,
        &[
            backgrounds[3].clone(),
            backgrounds[2].clone(),
            backgrounds[1].clone(),
            backgrounds[0].clone(),
        ],
    );

    if is_ui {
        #[cfg(feature = "macroquad-backend")]
        bg.ui(&mut *root_ui());
    } else {
        bg.draw();
    }
}
