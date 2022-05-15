use ff_core::prelude::*;
use std::ops::Deref;

use super::{EditorAction, EditorContext, Map, ToolbarElement, ToolbarElementParams};

use ff_core::gui::get_gui_theme;
use ff_core::gui::ELEMENT_MARGIN;
use ff_core::macroquad::ui::{widgets, Ui};

pub struct TilesetDetailsElement {
    params: ToolbarElementParams,
}

impl TilesetDetailsElement {
    pub fn new() -> Self {
        let params = ToolbarElementParams {
            header: None,
            has_margins: true,
            ..Default::default()
        };

        TilesetDetailsElement { params }
    }
}

impl ToolbarElement for TilesetDetailsElement {
    fn get_params(&self) -> &ToolbarElementParams {
        &self.params
    }

    fn draw(
        &mut self,
        ui: &mut Ui,
        size: Vec2,
        map: &Map,
        ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let mut res = None;

        let mut position = Vec2::ZERO;

        if let Some(tileset_id) = &ctx.selected_tileset {
            let tileset = map.tilesets.get(tileset_id).unwrap();

            let texture = get_texture(&tileset.texture_id);

            let grid_size = Size::new(
                tileset.grid_size.width as f32,
                tileset.grid_size.height as f32,
            );

            let scaled_width = size.x;
            let texture_size = texture.size();

            let scaled_height = (scaled_width / texture_size.width) * texture_size.height;

            let scaled_tile_size = Size::new(
                scaled_width / grid_size.width,
                scaled_height / grid_size.height,
            );

            widgets::Texture::new(texture.deref().into())
                .position(position)
                .size(scaled_width, scaled_height)
                .ui(ui);

            {
                let gui_theme = get_gui_theme();
                ui.push_skin(&gui_theme.tileset_grid);
            }

            for y in 0..tileset.grid_size.height {
                for x in 0..tileset.grid_size.width {
                    let tile_id = y * tileset.grid_size.width + x;

                    let is_selected = if let Some(selected) = ctx.selected_tile {
                        selected == tile_id
                    } else {
                        false
                    };

                    if is_selected {
                        let gui_theme = get_gui_theme();
                        ui.push_skin(&gui_theme.tileset_grid_selected);
                    }

                    let position: Vec2 = vec2(x as f32, y as f32) * Vec2::from(scaled_tile_size);

                    let button = widgets::Button::new("")
                        .size(scaled_tile_size.into())
                        .position(position)
                        .ui(ui);

                    if button {
                        res = Some(EditorAction::SelectTile {
                            id: tile_id,
                            tileset_id: tileset.id.clone(),
                        });
                    }

                    if is_selected {
                        ui.pop_skin();
                    }
                }
            }

            ui.pop_skin();

            position.y += scaled_height + ELEMENT_MARGIN;
        }

        res
    }

    fn is_drawn(&self, _map: &Map, ctx: &EditorContext) -> bool {
        ctx.selected_tileset.is_some()
    }
}

impl Default for TilesetDetailsElement {
    fn default() -> Self {
        Self::new()
    }
}
