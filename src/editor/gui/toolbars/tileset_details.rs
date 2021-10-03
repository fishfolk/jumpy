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
    Resources,
    editor::{
        gui::{
            ELEMENT_MARGIN,
        },
    },
};

use super::{
    Map,
    GuiResources,
    Toolbar,
    ToolbarElement,
    ToolbarElementBuilder,
    EditorAction,
    EditorDrawParams,
};

pub fn create_tileset_details_element(width: f32, height_factor: f32) -> ToolbarElement {
    ToolbarElementBuilder::new(width, height_factor)
        .with_menubar(draw_tileset_details_menubar)
        .with_margins()
        .build(hash!("tileset_details"), draw_tileset_details_element)
}

fn draw_tileset_details_element(ui: &mut Ui, id: Id, size: Vec2, map: &Map, params: &EditorDrawParams) -> Option<EditorAction> {
    let mut res = None;

    let mut position = Vec2::ZERO;

    let gui_resources = storage::get::<GuiResources>();
    ui.push_skin(&gui_resources.editor_skins.menu);

    widgets::Group::new(hash!(id, "main_group"), size).position(position).ui(ui, |ui| {
        let mut position = Vec2::ZERO;

        if let Some(id) = &params.selected_tileset {
            let tileset = map.tilesets.get(id).unwrap();

            let texture = {
                let resources = storage::get::<Resources>();
                resources.textures.get(&tileset.texture_id).cloned().unwrap()
            };

            let width = size.x - (ELEMENT_MARGIN * 2.0);
            let height = (width / texture.width()) * texture.height();

            let cell_size = vec2(width / tileset.grid_size.x as f32, height / tileset.grid_size.y as f32);

            widgets::Texture::new(texture)
                .position(position)
                .size(width, height)
                .ui(ui);

            ui.push_skin(&gui_resources.editor_skins.toolbar_tileset_grid);

            for y in 0..tileset.grid_size.y {
                for x in 0..tileset.grid_size.x {
                    let tile_id = y * tileset.grid_size.x + x;

                    let is_selected = if let Some(selected) = params.selected_tile {
                        selected == tile_id
                    } else {
                        false
                    };

                    if is_selected {
                        ui.push_skin(&gui_resources.editor_skins.toolbar_tileset_grid_selected);
                    }

                    let position = vec2(x as f32, y as f32) * cell_size;

                    let button = widgets::Button::new("")
                        .size(cell_size)
                        .position(position)
                        .ui(ui);

                    if is_selected {
                        ui.pop_skin();
                    }

                    if button {
                        res = Some(EditorAction::SelectTile {
                            id: tile_id,
                            tileset_id: tileset.id.clone(),
                        });
                    }
                }
            }

            ui.pop_skin();

            position.y += height + Toolbar::MARGIN;
        }
    });

    ui.pop_skin();

    res
}

fn draw_tileset_details_menubar(ui: &mut Ui, id: Id, size: Vec2, _map: &Map, params: &EditorDrawParams) -> Option<EditorAction> {
    let mut res = None;

    let mut position = Vec2::ZERO;

    let button_size = vec2(size.x * 0.5, Toolbar::MENUBAR_HEIGHT);

    let attributes_btn = widgets::Button::new("attributes")
        .size(button_size)
        .position(position)
        .ui(ui);

    if attributes_btn {
        if let Some(tileset_id) = params.selected_tileset.clone() {
            //res = Some(EditorAction::OpenTileAttributesWindow(tileset_id));
        }
    }

    position.x += button_size.x;

    let advanced_btn = widgets::Button::new("advanced")
        .size(button_size)
        .position(position)
        .ui(ui);

    if advanced_btn {
        if let Some(tileset_id) = params.selected_tileset.clone() {
            //res = Some(EditorAction::OpenTilesetPropertiesWindow(tileset_id));
        }
    }

    res
}