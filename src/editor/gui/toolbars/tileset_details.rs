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
        .build(hash!("tileset_details"), draw_tileset_details)
}

fn draw_tileset_details(ui: &mut Ui, id: Id, size: Vec2, map: &Map, params: &EditorDrawParams) -> Option<EditorAction> {
    let mut res = None;

    let size = vec2(size.x, size.y - Toolbar::BUTTON_BAR_TOTAL_HEIGHT);

    let mut position = Vec2::ZERO;

    let gui_resources = storage::get::<GuiResources>();
    ui.push_skin(&gui_resources.editor_skins.menu);

    widgets::Group::new(hash!(id, "tileset_properties_group"), size).position(position).ui(ui, |ui| {
        let mut position = Vec2::ZERO;

        if let Some(id) = &params.selected_tileset {
            let tileset = map.tilesets.get(id).unwrap();

            let texture = {
                let resources = storage::get::<Resources>();
                resources.textures.get(&tileset.texture_id).cloned().unwrap()
            };

            let height = (size.x / texture.width()) * texture.height();

            widgets::Texture::new(texture)
                .position(position)
                .size(size.x, height)
                .ui(ui);

            let cell_size = vec2(size.x / tileset.grid_size.x as f32, height / tileset.grid_size.y as f32);

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

            position.y += height + Toolbar::SEPARATOR_HEIGHT;
        }
    });

    ui.pop_skin();

    // position.y += size.y;
    //
    // let size = vec2(size.x, Toolbar::BUTTON_BAR_TOTAL_HEIGHT);
    //
    // widgets::Group::new(hash!(id, "layer_list_button_bar"), size).position(position).ui(ui, |ui| {
    //     let mut position = vec2(0.0, Toolbar::SEPARATOR_HEIGHT);
    //
    //     let button_size = vec2(size.x * 0.25, Toolbar::BUTTON_BAR_BUTTON_HEIGHT);
    //
    //     let create_btn = widgets::Button::new("+")
    //         .size(button_size)
    //         .position(position)
    //         .ui(ui);
    //
    //     if create_btn {
    //         res = Some(EditorAction::OpenCreateLayerWindow);
    //     }
    //
    //     position.x += button_size.x;
    //
    //     let delete_btn = widgets::Button::new("-")
    //         .size(button_size)
    //         .position(position)
    //         .ui(ui);
    //
    //     if delete_btn {
    //         if let Some(layer_id) = params.selected_layer.clone() {
    //             res = Some(EditorAction::DeleteLayer(layer_id));
    //         }
    //     }
    //
    //     position.x += button_size.x;
    //
    //     let up_btn = widgets::Button::new("Up")
    //         .size(button_size)
    //         .position(position)
    //         .ui(ui);
    //
    //     if up_btn {
    //         if let Some(layer_id) = &params.selected_layer {
    //             let mut i = 0;
    //             for id in &map.draw_order {
    //                 if id == layer_id && i > 0 {
    //                     res = Some(EditorAction::SetLayerDrawOrderIndex {
    //                         id: layer_id.clone(),
    //                         index: i - 1,
    //                     });
    //
    //                     break;
    //                 }
    //
    //                 i += 1;
    //             }
    //         }
    //     }
    //
    //     position.x += button_size.x;
    //
    //     let down_btn = widgets::Button::new("Down")
    //         .size(button_size)
    //         .position(position)
    //         .ui(ui);
    //
    //     if down_btn {
    //         if let Some(layer_id) = &params.selected_layer {
    //             let mut i = 0;
    //             for id in &map.draw_order {
    //                 if id == layer_id && i + 1 < map.draw_order.len() {
    //                     res = Some(EditorAction::SetLayerDrawOrderIndex {
    //                         id: layer_id.clone(),
    //                         index: i + 1,
    //                     });
    //
    //                     break;
    //                 }
    //
    //                 i += 1;
    //             }
    //         }
    //     }
    //});

    res
}