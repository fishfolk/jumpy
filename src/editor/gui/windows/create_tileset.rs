use std::ops::Deref;

use macroquad::{
    ui::{
        Ui,
        hash,
        widgets,
    },
    experimental::{
        collections::storage,
    },
    prelude::*,
};

use crate::{
    map::Map,
    Resources,
};

use super::{
    ButtonParams,
    Window,
    WindowParams,
    EditorAction,
    EditorDrawParams,
};

pub struct CreateTilesetWindow {
    params: WindowParams,
    tileset_id: String,
    texture_id: String,
}

impl CreateTilesetWindow {
    pub fn new() -> Self {
        let params = WindowParams {
            title: Some("Create Tileset".to_string()),
            size: vec2(350.0, 350.0),
            ..Default::default()
        };

        CreateTilesetWindow {
            params,
            tileset_id: "Unnamed Tileset".to_string(),
            texture_id: "tileset".to_string(),
        }
    }
}

impl Window for CreateTilesetWindow {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn get_buttons(&self, map: &Map, _draw_params: &EditorDrawParams) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let is_existing_id = map.tilesets
            .iter()
            .any(|(id, _)| id == &self.tileset_id);

        let mut action = None;
        if !is_existing_id {
            let batch = self.get_close_action()
                .then(EditorAction::CreateTileset {
                    id: self.tileset_id.clone(),
                    texture_id: self.texture_id.clone(),
                });

            action = Some(batch);
        }

        res.push(ButtonParams {
            label: "Create",
            action,
            ..Default::default()
        });

        res.push(ButtonParams {
            label: "Cancel",
            action: Some(self.get_close_action()),
            ..Default::default()
        });

        res
    }

    fn draw(&mut self, ui: &mut Ui, _size: Vec2, _map: &Map, _draw_params: &EditorDrawParams) -> Option<EditorAction> {
        let id = hash!("create_tileset_element");

        let resources = storage::get::<Resources>();
        let mut textures = resources.textures
            .iter()
            .map(|(key, _)| key.deref())
            .collect::<Vec<&str>>();

        textures.sort_unstable();

        {
            let size = vec2(173.0, 25.0);

            widgets::InputText::new(hash!(id, "name_input"))
                .size(size)
                .ratio(1.0)
                .label("Name")
                .ui(ui, &mut self.tileset_id);
        }

        ui.separator();
        ui.separator();
        ui.separator();
        ui.separator();

        let mut texture_index = 0;
        for id in &textures {
            if id == &self.texture_id {
                break;
            }

            texture_index += 1;
        }

        widgets::ComboBox::new(hash!(id, "texture_input"), textures.as_slice())
            .ratio(0.4)
            .label("Texture")
            .ui(ui, &mut texture_index);

        self.texture_id = textures
            .get(texture_index)
            .unwrap()
            .to_string();

        None
    }
}

impl Default for CreateTilesetWindow {
    fn default() -> Self {
        Self::new()
    }
}