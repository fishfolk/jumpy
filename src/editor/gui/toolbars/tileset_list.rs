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

pub fn create_tileset_list(width: f32, height_factor: f32) -> ToolbarElement {
    ToolbarElementBuilder::new(width, height_factor)
        .with_header("Tilesets")
        .build(hash!("tileset_list"), draw_tileset_list)
}

fn draw_tileset_list(ui: &mut Ui, id: Id, size: Vec2, map: &Map, params: &EditorDrawParams) -> Option<EditorAction> {
    let mut res = None;

    let size = vec2(size.x, size.y - Toolbar::BUTTON_BAR_TOTAL_HEIGHT);

    let entry_size = vec2(size.x, Toolbar::LIST_ENTRY_HEIGHT);
    let mut position = Vec2::ZERO;

    let gui_resources = storage::get::<GuiResources>();
    ui.push_skin(&gui_resources.editor_skins.menu);

    widgets::Group::new(hash!(id, "tileset_list_group"), size).position(position).ui(ui, |ui| {
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

    position.y += size.y;

    let size = vec2(size.x, Toolbar::BUTTON_BAR_TOTAL_HEIGHT);

    widgets::Group::new(hash!(id, "tileset_button_bar"), size).position(position).ui(ui, |ui| {
        let mut position = vec2(0.0, Toolbar::SEPARATOR_HEIGHT);

        let button_size = vec2(size.x * 0.25, Toolbar::BUTTON_BAR_BUTTON_HEIGHT);

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
    });

    res
}