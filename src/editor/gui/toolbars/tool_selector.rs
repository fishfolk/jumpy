use super::{ToolbarElement, ToolbarElementParams};
use crate::editor::actions::EditorAction;
use crate::editor::gui::EditorDrawParams;
use crate::map::Map;
use macroquad::math::Vec2;
use macroquad::ui::Ui;

pub struct ToolSelectorElement {
    params: ToolbarElementParams,
}

impl ToolSelectorElement {
    pub fn new() -> Self {
        let params = ToolbarElementParams {
            header: None,
            has_buttons: false,
            has_margins: true,
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
        _ui: &mut Ui,
        _size: Vec2,
        _map: &Map,
        _draw_params: &EditorDrawParams,
    ) -> Option<EditorAction> {
        None
    }
}

impl Default for ToolSelectorElement {
    fn default() -> Self {
        Self::new()
    }
}
