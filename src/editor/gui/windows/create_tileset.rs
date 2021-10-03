use std::ops::Deref;

use macroquad::{
    ui::{
        Id,
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
    WindowPosition,
    WindowBuilder,
};

pub struct CreateTilesetWindow {
    position: WindowPosition,
    size: Vec2,
    tileset_id: String,
    texture_id: String,
}

impl CreateTilesetWindow {
    pub fn new() -> Self {
        CreateTilesetWindow {
            position: WindowPosition::Centered,
            size: vec2(350.0, 350.0),
            tileset_id: "Unnamed Tileset".to_string(),
            texture_id: "tileset".to_string(),
        }
    }

    pub fn get_rect(&self) -> Rect {
        let position = self.position.to_absolute(self.size);
        Rect::new(position.x, position.y, self.size.x, self.size.y)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        let rect = self.get_rect();
        rect.contains(point)
    }

    pub fn draw(&mut self, ui: &mut Ui, map: &Map, _params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let resources = storage::get::<Resources>();
        let mut textures = resources.textures
            .iter()
            .map(|(key, _)| key.deref())
            .collect::<Vec<&str>>();

        textures.sort();

        WindowBuilder::new(self.size)
            .with_position(self.position, true)
            .with_title("Create Tileset")
            .build(ui, |ui| {
                {
                    let size = vec2(173.0, 25.0);

                    widgets::InputText::new(hash!())
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

                widgets::ComboBox::new(hash!(), textures.as_slice())
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
                    let batch = EditorAction::batch(&[
                        EditorAction::CloseCreateTilesetWindow,
                        EditorAction::CreateTileset {
                            id: self.tileset_id.clone(),
                            texture_id: self.texture_id.clone(),
                        }
                    ]);

                    res = Some(batch);
                }

                ui.same_line(0.0);

                if ui.button(None, "Cancel") {
                    res = Some(EditorAction::CloseCreateTilesetWindow);
                }
            });

        res
    }
}
