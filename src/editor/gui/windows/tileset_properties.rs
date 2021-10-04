use macroquad::{
    ui::{
        Ui,
        widgets,
    },
    experimental::{
        collections::storage,
    },
    prelude::*,
};

use crate::{
    Resources,
    gui::GuiResources,
};

use super::{
    Map,
    ButtonParams,
    Window,
    WindowParams,
    EditorAction,
    EditorDrawParams,
};
use crate::map::MapTileset;

pub struct TilesetPropertiesWindow {
    params: WindowParams,
    tileset_id: String,
    autotile_mask: Vec<bool>,
    has_data: bool,
}

impl TilesetPropertiesWindow {
    pub fn new(tileset_id: &str) -> Self {
        let params = WindowParams {
            size: vec2(600.0, 500.0),
            is_static: true,
            ..Default::default()
        };

        TilesetPropertiesWindow {
            params,
            tileset_id: tileset_id.to_string(),
            autotile_mask: Vec::new(),
            has_data: false,
        }
    }

    pub fn read_from_tileset(&mut self, map: &Map) {
        if let Some(tileset) = map.tilesets.get(&self.tileset_id) {
            let subgrid_size = tileset.grid_size * tileset.tile_subdivisions;
            let subtile_cnt = (subgrid_size.x * subgrid_size.y) as usize;

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

    fn draw_autotile_settings(&mut self, ui: &mut Ui, position: Vec2, size: Vec2, tileset: &MapTileset) -> Option<EditorAction> {
        let tileset_texture = {
            let resources = storage::get::<Resources>();
            resources.textures.get(&tileset.texture_id).unwrap().clone()
        };

        let tileset_texture_size = vec2(tileset_texture.width(), tileset_texture.height());

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

        widgets::Texture::new(tileset_texture)
            .size(scaled_width, scaled_height)
            .position(position)
            .ui(ui);

        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.editor_skins.tileset_subtile_grid);
        }

        for y in 0..subgrid_size.y {
            for x in 0..subgrid_size.x {
                let i = (y * subgrid_size.x + x) as usize;

                let is_selected = self.autotile_mask[i];

                if is_selected {
                    let gui_resources = storage::get::<GuiResources>();
                    ui.push_skin(&gui_resources.editor_skins.tileset_subtile_grid_selected);
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

    fn draw(&mut self, ui: &mut Ui, size: Vec2, map: &Map, _draw_params: &EditorDrawParams) -> Option<EditorAction> {
        if let Some(tileset) = map.tilesets.get(&self.tileset_id) {
            if self.has_data == false {
                self.read_from_tileset(map);
            }

            if self.has_data {
                {
                    let size = size;
                    let position = Vec2::ZERO;

                    if let Some(action) = self.draw_autotile_settings(ui, position, size, tileset) {
                        return Some(action);
                    }
                }
            }
        }

        None
    }

    fn get_buttons(&self, _map: &Map, _draw_params: &EditorDrawParams) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let id = self.tileset_id.clone();
        let autotile_mask = self.autotile_mask.clone();

        let action = self.get_close_action()
            .then(EditorAction::UpdateTilesetAutotileMask {
                id,
                autotile_mask
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