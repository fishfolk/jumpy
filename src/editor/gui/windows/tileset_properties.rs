use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{hash, widgets, Ui},
};

use crate::editor::gui::combobox::ComboBoxVec;
use crate::editor::gui::{ComboBoxBuilder, ComboBoxValue};
use crate::{gui::GuiResources, Resources};

use super::{ButtonParams, EditorAction, EditorContext, Map, Window, WindowParams};
use crate::map::MapTileset;
use crate::resources::TextureKind;

pub struct TilesetPropertiesWindow {
    params: WindowParams,
    tileset_id: String,
    autotile_mask: Vec<bool>,
    texture: ComboBoxVec,
    has_data: bool,
}

impl TilesetPropertiesWindow {
    pub fn new(tileset_id: &str) -> Self {
        let params = WindowParams {
            size: vec2(600.0, 500.0),
            is_static: true,
            ..Default::default()
        };

        let resources = storage::get::<Resources>();
        let texture_ids = resources
            .textures
            .iter()
            .filter_map(|(k, v)| {
                if let Some(kind) = v.meta.kind {
                    if kind == TextureKind::Tileset {
                        return Some(k.as_str());
                    }
                }

                None
            })
            .collect::<Vec<_>>();

        let texture = ComboBoxVec::new(0, &texture_ids);

        TilesetPropertiesWindow {
            params,
            tileset_id: tileset_id.to_string(),
            autotile_mask: Vec::new(),
            texture,
            has_data: false,
        }
    }

    pub fn read_from_tileset(&mut self, map: &Map) {
        if let Some(tileset) = map.tilesets.get(&self.tileset_id) {
            let subgrid_size = tileset.grid_size * tileset.tile_subdivisions;
            let subtile_cnt = (subgrid_size.x * subgrid_size.y) as usize;

            self.texture.set_value(&tileset.texture_id);

            self.autotile_mask = Vec::with_capacity(subtile_cnt);
            for i in 0..subtile_cnt {
                if let Some(subtile) = tileset.autotile_mask.get(i) {
                    self.autotile_mask.push(*subtile);
                } else {
                    self.autotile_mask.push(false);
                }
            }

            self.has_data = true;
        }
    }

    #[allow(dead_code)]
    fn draw_autotile_settings(
        &mut self,
        ui: &mut Ui,
        position: Vec2,
        size: Vec2,
        tileset: &MapTileset,
    ) -> Option<EditorAction> {
        let texture_entry = {
            let resources = storage::get::<Resources>();
            resources
                .textures
                .get(&tileset.texture_id)
                .cloned()
                .unwrap()
        };

        let tileset_texture_size = vec2(
            texture_entry.texture.width(),
            texture_entry.texture.height(),
        );

        let mut scaled_width = size.x;
        let mut scaled_height = (scaled_width / tileset_texture_size.x) * tileset_texture_size.y;

        if scaled_height > size.y {
            scaled_height = size.y;
            scaled_width = (scaled_height / tileset_texture_size.y) * tileset_texture_size.x;
        }

        let subgrid_size = tileset.grid_size * tileset.tile_subdivisions;

        let scaled_subtile_size = vec2(
            scaled_width / subgrid_size.x as f32,
            scaled_height / subgrid_size.y as f32,
        );

        widgets::Texture::new(texture_entry.texture)
            .size(scaled_width, scaled_height)
            .position(position)
            .ui(ui);

        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.skins.tileset_subtile_grid);
        }

        for y in 0..subgrid_size.y {
            for x in 0..subgrid_size.x {
                let i = (y * subgrid_size.x + x) as usize;

                let is_selected = self.autotile_mask[i];

                if is_selected {
                    let gui_resources = storage::get::<GuiResources>();
                    ui.push_skin(&gui_resources.skins.tileset_subtile_grid_selected);
                }

                let subtile_position = position + vec2(x as f32, y as f32) * scaled_subtile_size;

                let was_clicked = widgets::Button::new("")
                    .size(scaled_subtile_size)
                    .position(subtile_position)
                    .ui(ui);

                if is_selected {
                    ui.pop_skin();
                }

                if was_clicked {
                    self.autotile_mask[i] = !is_selected;
                }
            }
        }

        ui.pop_skin();

        None
    }
}

impl Window for TilesetPropertiesWindow {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn draw(
        &mut self,
        ui: &mut Ui,
        _size: Vec2,
        map: &Map,
        _ctx: &EditorContext,
    ) -> Option<EditorAction> {
        if let Some(_tileset) = map.tilesets.get(&self.tileset_id) {
            let id = hash!("tileset_properties_window");

            if !self.has_data {
                self.read_from_tileset(map);
            }

            if self.has_data {
                /*
                {
                    let size = size;
                    let position = Vec2::ZERO;

                    if let Some(action) = self.draw_autotile_settings(ui, position, size, tileset) {
                        return Some(action);
                    }
                }
                */

                widgets::Label::new(&self.tileset_id).ui(ui);

                ui.separator();

                ComboBoxBuilder::new(hash!(id, "texture_input"))
                    .with_ratio(0.8)
                    .with_label("Texture")
                    .build(ui, &mut self.texture);
            }
        }

        None
    }

    fn get_buttons(&self, _map: &Map, _ctx: &EditorContext) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let autotile_mask = self.autotile_mask.clone();

        let action = self.get_close_action().then(EditorAction::UpdateTileset {
            id: self.tileset_id.clone(),
            texture_id: self.texture.get_value(),
            autotile_mask,
        });

        res.push(ButtonParams {
            label: "Save",
            action: Some(action),
            ..Default::default()
        });

        res.push(ButtonParams {
            label: "Cancel",
            action: Some(self.get_close_action()),
            ..Default::default()
        });

        res
    }
}
