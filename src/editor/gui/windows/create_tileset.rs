use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{hash, widgets, Ui},
};

use crate::editor::gui::combobox::ComboBoxVec;
use crate::editor::gui::{ComboBoxBuilder, ComboBoxValue};
use crate::{map::Map, Resources};

use super::{ButtonParams, EditorAction, EditorContext, Window, WindowParams};
use crate::resources::TextureKind;

pub struct CreateTilesetWindow {
    params: WindowParams,
    tileset_id: String,
    texture: ComboBoxVec,
}

impl CreateTilesetWindow {
    pub fn new() -> Self {
        let params = WindowParams {
            title: Some("Create Tileset".to_string()),
            size: vec2(320.0, 250.0),
            ..Default::default()
        };

        let resources = storage::get::<Resources>();
        let mut textures = resources
            .textures
            .iter()
            .filter_map(|(k, v)| {
                if let Some(kind) = v.meta.kind {
                    if kind == TextureKind::Tileset {
                        return Some(k.clone());
                    }
                }

                None
            })
            .collect::<Vec<String>>();

        textures.sort_unstable();

        CreateTilesetWindow {
            params,
            tileset_id: "Unnamed Tileset".to_string(),
            texture: textures.as_slice().into(),
        }
    }
}

impl Window for CreateTilesetWindow {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn get_buttons(&self, map: &Map, _ctx: &EditorContext) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let is_existing_id = map.tilesets.iter().any(|(id, _)| id == &self.tileset_id);

        let mut action = None;
        if !is_existing_id {
            let batch = self.get_close_action().then(EditorAction::CreateTileset {
                id: self.tileset_id.clone(),
                texture_id: self.texture.get_value(),
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

    fn draw(
        &mut self,
        ui: &mut Ui,
        _size: Vec2,
        _map: &Map,
        _ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let id = hash!("create_tileset_window");

        widgets::InputText::new(hash!(id, "name_input"))
            .ratio(0.8)
            .label("Name")
            .ui(ui, &mut self.tileset_id);

        ui.separator();

        ComboBoxBuilder::new(hash!(id, "texture_input"))
            .with_ratio(0.8)
            .with_label("Texture")
            .build(ui, &mut self.texture);

        None
    }
}

impl Default for CreateTilesetWindow {
    fn default() -> Self {
        Self::new()
    }
}
