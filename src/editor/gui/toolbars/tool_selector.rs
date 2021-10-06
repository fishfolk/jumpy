use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{widgets, Ui},
};

use super::{
    EditorAction, EditorContext, Map, ToolbarElement, ToolbarElementParams, ELEMENT_MARGIN,
};

use crate::{gui::GuiResources, Resources};

pub struct ToolSelectorElement {
    params: ToolbarElementParams,
}

impl ToolSelectorElement {
    pub fn new() -> Self {
        let params = ToolbarElementParams {
            header: None,
            has_margins: true,
            has_buttons: false,
        };

        ToolSelectorElement { params }
    }
}

impl ToolbarElement for ToolSelectorElement {
    fn get_params(&self) -> &ToolbarElementParams {
        &self.params
    }

    fn draw(
        &mut self,
        ui: &mut Ui,
        size: Vec2,
        _map: &Map,
        ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let mut res = None;

        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.editor_skins.tool_selector);
        }

        let size = vec2(size.x, size.x);
        let mut position = Vec2::ZERO;

        let resources = storage::get::<Resources>();
        // for (index, params) in ctx.available_tools.iter().enumerate() {
        //     let mut is_selected = false;
        //     if let Some(selected_index) = ctx.selected_tool {
        //         is_selected = index == selected_index;
        //     }
        //
        //     if is_selected {
        //         let gui_resources = storage::get::<GuiResources>();
        //         ui.push_skin(&gui_resources.editor_skins.tool_selector_selected);
        //     }
        //
        //     let texture = *resources.textures.get(&params.icon_texture_id).unwrap();
        //
        //     widgets::Texture::new(texture)
        //         .position(position)
        //         .size(size.x, size.y)
        //         .ui(ui);
        //
        //     let was_clicked = widgets::Button::new("")
        //         .position(position)
        //         .size(size)
        //         .ui(ui);
        //
        //     if was_clicked {
        //         res = Some(EditorAction::SelectTool(index));
        //     }
        //
        //     position.y += size.y + ELEMENT_MARGIN;
        //
        //     if is_selected {
        //         ui.pop_skin();
        //     }
        // }

        ui.pop_skin();

        res
    }
}

impl Default for ToolSelectorElement {
    fn default() -> Self {
        Self::new()
    }
}
