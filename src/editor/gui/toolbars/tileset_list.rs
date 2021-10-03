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

use super::{
    GuiResources,
    Toolbar,
    ToolbarElement,
    ToolbarElementBuilder,
    EditorDrawParams,
    EditorAction,
    Map,
};

pub fn create_tileset_list_element(width: f32, height_factor: f32) -> ToolbarElement {
    ToolbarElementBuilder::new(width, height_factor)
        .with_header("Tilesets")
        .with_menubar(draw_tileset_list_menubar)
        .build(hash!("tileset_list"), draw_tileset_list_element)
}

fn draw_tileset_list_element(ui: &mut Ui, id: Id, size: Vec2, map: &Map, params: &EditorDrawParams) -> Option<EditorAction> {
    let mut res = None;

    let entry_size = vec2(size.x, Toolbar::LIST_ENTRY_HEIGHT);
    let mut position = Vec2::ZERO;

    let gui_resources = storage::get::<GuiResources>();
    ui.push_skin(&gui_resources.editor_skins.menu);

    widgets::Group::new(hash!(id, "main_group"), size).position(position).ui(ui, |ui| {
        let mut position = Vec2::ZERO;

        for (id, _) in &map.tilesets {
            let is_selected = if let Some(selected_id) = &params.selected_tileset {
                id == selected_id
            } else {
                false
            };

            if is_selected {
                ui.push_skin(&gui_resources.editor_skins.menu_selected);
            }

            let button = widgets::Button::new("")
                .size(entry_size)
                .position(position)
                .ui(ui);

            ui.label(position, id);

            if button {
                res = Some(EditorAction::SelectTileset(id.clone()));
            }

            if is_selected {
                ui.pop_skin();
            }

            position.y += entry_size.y;
        }
    });

    ui.pop_skin();

    res
}

fn draw_tileset_list_menubar(ui: &mut Ui, _id: Id, size: Vec2, _map: &Map, params: &EditorDrawParams) -> Option<EditorAction> {
    let mut res = None;

    let mut position = Vec2::ZERO;

    let button_size = vec2(size.x * 0.25, Toolbar::MENUBAR_HEIGHT);

    let create_btn = widgets::Button::new("+")
        .size(button_size)
        .position(position)
        .ui(ui);

    if create_btn {
        res = Some(EditorAction::OpenCreateTilesetWindow);
    }

    position.x += button_size.x;

    let delete_btn = widgets::Button::new("-")
        .size(button_size)
        .position(position)
        .ui(ui);

    if delete_btn {
        if let Some(id) = params.selected_tileset.clone() {
            res = Some(EditorAction::DeleteTileset(id));
        }
    }

    res
}