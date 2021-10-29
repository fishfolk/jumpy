use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{root_ui, widgets, Ui},
};

use crate::resources::{Resources, TextureResource};

pub struct Background {
    textures: Vec<TextureResource>,
    size: Vec2,
    position: Vec2,
}

impl Background {
    pub fn new(size: Vec2, position: Vec2, textures: &[TextureResource]) -> Self {
        let textures = textures.to_vec();

        Background {
            textures,
            size,
            position,
        }
    }

    pub fn ui(&self, ui: &mut Ui) {
        for res in &self.textures {
            widgets::Texture::new(res.texture)
                .size(self.size.x, self.size.y)
                .position(self.position)
                .ui(ui);
        }
    }
}

pub fn draw_main_menu_background() {
    let resources = storage::get::<Resources>();

    let background_01 = resources.textures.get("background_01").cloned().unwrap();
    let background_02 = resources.textures.get("background_02").cloned().unwrap();
    let background_03 = resources.textures.get("background_03").cloned().unwrap();
    let background_04 = resources.textures.get("background_04").cloned().unwrap();

    let height = screen_height();
    let width = (height / background_01.texture.height()) * background_01.texture.width();

    let bg = Background::new(
        vec2(width, height),
        Vec2::ZERO,
        &[background_04, background_03, background_02, background_01],
    );

    bg.ui(&mut *root_ui());
}
