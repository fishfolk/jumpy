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
    editor::{
        EditorAction,
        EditorDrawParams,
    }
};

use super::{
    Window,
    WindowParams,
    WindowResult,
};

pub struct CreateTilesetWindow {
    params: WindowParams,
    tileset_id: String,
    texture_id: String,
}

impl CreateTilesetWindow {
    pub fn new() -> Box<Self> {
        let params = WindowParams {
            title: Some("Create Tileset".to_string()),
            size: vec2(350.0, 350.0),
            ..Default::default()
        };

        Box::new(CreateTilesetWindow {
            params,
            tileset_id: "Unnamed Tileset".to_string(),
            texture_id: "tileset".to_string(),
        })
    }
}

impl Window for CreateTilesetWindow {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn draw(&mut self, ui: &mut Ui, _size: Vec2, map: &Map, _draw_params: &EditorDrawParams) -> Option<WindowResult> {
        let id = hash!("create_tileset_element");

        let resources = storage::get::<Resources>();
        let mut textures = resources.textures
            .iter()
            .map(|(key, _)| key.deref())
            .collect::<Vec<&str>>();

        textures.sort();

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

        let is_existing_id = map.tilesets
            .iter()
            .find(|(id, _)| *id == &self.tileset_id)
            .is_some();

        if is_existing_id {
            ui.label(None, "A tileset with this name already exist!");
        } else {
            ui.label(None, "")
        }

        if ui.button(None, "Create") && is_existing_id == false {
            let action = EditorAction::CreateTileset {
                id: self.tileset_id.clone(),
                texture_id: self.texture_id.clone(),
            };

            return Some(WindowResult::Action(action));
        }

        ui.same_line(0.0);

        if ui.button(None, "Cancel") {
            return Some(WindowResult::Cancel);
        }

        None
    }
}
