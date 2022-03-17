use ff_core::prelude::*;

use ff_core::gui::get_gui_theme;
use ff_core::gui::combobox::{ComboBoxVec, ComboBoxBuilder, ComboBoxValue};
use ff_core::macroquad::hash;
use ff_core::macroquad::ui::{Ui, widgets};

use super::{ButtonParams, EditorAction, EditorContext, Map, Window, WindowParams};
use ff_core::map::MapTileset;
use ff_core::resources::TextureKind;
use crate::GuiTheme;

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

        let texture_ids = iter_textures()
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
            let subgrid_size = tileset.grid_size * tileset.tile_subdivisions.into();
            let subtile_cnt = (subgrid_size.width * subgrid_size.height) as usize;

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
        let texture_res = get_texture(&tileset.texture_id);

        let texture_size = texture_res.texture.size();
        let tileset_texture_size = vec2(
            texture_size.width,
            texture_size.height,
        );

        let mut scaled_width = size.x;
        let mut scaled_height = (scaled_width / tileset_texture_size.x) * tileset_texture_size.y;

        if scaled_height > size.y {
            scaled_height = size.y;
            scaled_width = (scaled_height / tileset_texture_size.y) * tileset_texture_size.x;
        }

        let subgrid_size = tileset.grid_size * tileset.tile_subdivisions.into();

        let scaled_subtile_size = Size::new(
            scaled_width / subgrid_size.width as f32,
            scaled_height / subgrid_size.height as f32,
        );

        widgets::Texture::new(texture_res.texture.into())
            .size(scaled_width, scaled_height)
            .position(position)
            .ui(ui);

        {
            let gui_theme = get_gui_theme();
            ui.push_skin(&gui_theme.tileset_subtile_grid);
        }

        for y in 0..subgrid_size.height {
            for x in 0..subgrid_size.width {
                let i = (y * subgrid_size.width + x) as usize;

                let is_selected = self.autotile_mask[i];

                if is_selected {
                    let gui_theme = get_gui_theme();
                    ui.push_skin(&gui_theme.tileset_subtile_grid_selected);
                }

                let subtile_position = position + vec2(x as f32, y as f32) * Vec2::from(scaled_subtile_size);

                let was_clicked = widgets::Button::new("")
                    .size(scaled_subtile_size.into())
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
