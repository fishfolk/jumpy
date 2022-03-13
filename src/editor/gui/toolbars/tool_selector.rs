use std::any::TypeId;

use core::prelude::*;

use super::{
    EditorAction, EditorContext, Map, ToolbarElement, ToolbarElementParams, ELEMENT_MARGIN,
};

use crate::editor::tools::EditorTool;
use crate::{editor::tools::{get_tool_instance_of_id, EditorToolParams}, GuiTheme};
use crate::macroquad::ui::{Ui, widgets};

pub struct ToolSelectorElement {
    params: ToolbarElementParams,
    tools: Vec<TypeId>,
}

impl ToolSelectorElement {
    pub fn new() -> Self {
        let params = ToolbarElementParams {
            header: None,
            has_margins: true,
            has_buttons: false,
        };

        ToolSelectorElement {
            params,
            tools: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_tool<T: EditorTool + 'static>(self) -> Self {
        let id = TypeId::of::<T>();
        let mut tools = self.tools;
        tools.push(id);

        ToolSelectorElement { tools, ..self }
    }

    pub fn add_tool<T: EditorTool + 'static>(&mut self) {
        let id = TypeId::of::<T>();
        self.tools.push(id);
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
        map: &Map,
        ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let mut res = None;

        {
            let gui_theme = storage::get::<GuiTheme>();
            ui.push_skin(&gui_theme.tool_selector);
        }

        let size = vec2(size.x, size.x);
        let mut position = Vec2::ZERO;

        // TODO: Grey out inactive tools, in stead of removing them altogether
        let mut available_tools = self
            .tools
            .iter()
            .filter_map(|id| {
                let tool = get_tool_instance_of_id(id);
                if tool.is_available(map, ctx) {
                    return Some((Some(*id), tool.get_params().clone()));
                }

                None
            })
            .collect::<Vec<(Option<TypeId>, EditorToolParams)>>();

        available_tools.insert(
            0,
            (
                None,
                EditorToolParams {
                    name: "Cursor".to_string(),
                    icon_texture_id: "cursor_tool_icon".to_string(),
                    ..Default::default()
                },
            ),
        );

        for (id, params) in available_tools {
            let mut is_selected = false;
            if let Some(id) = id {
                if let Some(selected_id) = ctx.selected_tool {
                    is_selected = id == selected_id;
                }
            } else {
                is_selected = ctx.selected_tool.is_none();
            }

            if is_selected {
                let gui_theme = storage::get::<GuiTheme>();
                ui.push_skin(&gui_theme.tool_selector_selected);
            }

            let was_clicked = widgets::Button::new("")
                .position(position)
                .size(size)
                .ui(ui);

            let texture_entry = get_texture(&params.icon_texture_id);

            widgets::Texture::new(texture_entry.texture.into())
                .position(position)
                .size(size.x, size.y)
                .ui(ui);

            /*{
                let label_size = ui.calc_size(&params.name);
                if label_size.x + (ELEMENT_MARGIN * 2.0) > size.x {
                    let words: Vec<_> = params.name.split(' ').collect();

                    let x_center = position.x + size.x / 2.0;
                    let mut y_offset = ELEMENT_MARGIN;

                    for word in words {
                        let label_size = ui.calc_size(word);

                        if y_offset + label_size.y > size.y {
                            break;
                        }

                        let position = vec2(x_center - label_size.x / 2.0, position.y + y_offset);
                        widgets::Label::new(word).position(position).ui(ui);

                        y_offset += label_size.y;
                    }
                }
            }*/

            if was_clicked {
                res = Some(EditorAction::SelectTool(id));
            }

            position.y += size.y + ELEMENT_MARGIN;

            if is_selected {
                ui.pop_skin();
            }
        }

        ui.pop_skin();

        res
    }
}

impl Default for ToolSelectorElement {
    fn default() -> Self {
        Self::new()
    }
}
