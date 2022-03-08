use core::prelude::*;
use crate::macroquad::ui::{root_ui, Ui, widgets};

use crate::resources::{Resources, TextureResource};

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

    #[cfg(not(feature = "ultimate"))]
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
    let resources = storage::get::<Resources>();

    let background_01 = resources.textures.get("background_01").cloned().unwrap();
    let background_02 = resources.textures.get("background_02").cloned().unwrap();
    let background_03 = resources.textures.get("background_03").cloned().unwrap();
    let background_04 = resources.textures.get("background_04").cloned().unwrap();

    let size = background_01.texture.size();

    let height = get_viewport().height;
    let width = (height / size.height) * size.width;

    let bg = Background::new(
        Size::new(width, height),
        Vec2::ZERO,
        &[background_04, background_03, background_02, background_01],
    );

    if is_ui {
        #[cfg(not(feature = "ultimate"))]
        bg.ui(&mut *root_ui());
    } else {
        bg.draw();
    }
}
