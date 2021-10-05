use macroquad::{
    prelude::*,
};

use super::{Map, EditorContext, EditorAction};


pub const DEFAULT_TOOL_ICON_TEXTURE_ID: &'static str = "default_tool_icon";

#[derive(Debug, Clone)]
pub struct EditorToolParams {
    pub name: String,
    pub icon_texture_id: String,
    pub icon_texture_rect: Option<Rect>,
}

impl Default for EditorToolParams {
    fn default() -> Self {
        EditorToolParams {
            name: "Unnamed Tool".to_string(),
            icon_texture_id: DEFAULT_TOOL_ICON_TEXTURE_ID.to_string(),
            icon_texture_rect: None,
        }
    }
}

pub trait EditorTool {
    fn get_params(&self) -> &EditorToolParams;

    fn apply(&mut self, map: &Map, ctx: &EditorContext) -> Option<EditorAction>;
}
