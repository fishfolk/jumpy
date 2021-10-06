mod eraser;
mod placement;

pub use eraser::EraserTool;
pub use placement::PlacementTool;

use macroquad::prelude::*;

use super::{EditorAction, EditorContext, Map};


static mut TOOL_INSTANCES: Option<Vec<Box<dyn EditorTool>>> = None;

unsafe fn get_all_tool_instances() -> &'static mut Vec<Box<dyn EditorTool>> {
    if TOOL_INSTANCES.is_none() {
        TOOL_INSTANCES = Some(Vec::new());
    }

    TOOL_INSTANCES.as_mut().unwrap()
}

pub fn add_tool_instance<T: EditorTool + 'static>(tool: T) {
    unsafe { get_all_tool_instances() }.push(Box::new(tool));
}

pub fn get_tool_instance(index: usize) -> &'static mut dyn EditorTool {
    unsafe { get_all_tool_instances() }.get_mut(index).unwrap().as_mut()
}

pub const DEFAULT_TOOL_ICON_TEXTURE_ID: &str = "default_tool_icon";

#[derive(Debug, Clone)]
pub struct EditorToolParams {
    pub name: String,
    pub icon_texture_id: String,
}

impl Default for EditorToolParams {
    fn default() -> Self {
        EditorToolParams {
            name: "Unnamed Tool".to_string(),
            icon_texture_id: DEFAULT_TOOL_ICON_TEXTURE_ID.to_string(),
        }
    }
}

pub trait EditorTool {
    fn get_params(&self) -> &EditorToolParams;

    fn get_action(&mut self, map: &Map, ctx: &EditorContext) -> Option<EditorAction>;

    fn is_available(&self, _map: &Map, _ctx: &EditorContext) -> bool {
        true
    }
}
