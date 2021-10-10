use std::{any::TypeId, collections::HashMap};

mod eraser;
mod placement;

pub use eraser::EraserTool;
pub use placement::PlacementTool;

use macroquad::prelude::*;

use super::{EditorAction, EditorContext, Map};

static mut TOOL_INSTANCES: Option<HashMap<TypeId, Box<dyn EditorTool>>> = None;

unsafe fn get_tool_instance_directory() -> &'static mut HashMap<TypeId, Box<dyn EditorTool>> {
    if TOOL_INSTANCES.is_none() {
        TOOL_INSTANCES = Some(HashMap::new());
    }

    TOOL_INSTANCES.as_mut().unwrap()
}

pub fn add_tool_instance<T: EditorTool + 'static>(tool: T) -> TypeId {
    let id = TypeId::of::<T>();
    unsafe { get_tool_instance_directory() }.insert(id, Box::new(tool));
    id
}

pub fn get_tool_instance_of_id(id: &TypeId) -> &'static mut dyn EditorTool {
    unsafe { get_tool_instance_directory() }
        .get_mut(id)
        .unwrap()
        .as_mut()
}

// TODO: Cast to T
pub fn get_tool_instance<T: EditorTool + 'static>() -> &'static mut dyn EditorTool {
    let id = TypeId::of::<T>();
    get_tool_instance_of_id(&id)
}

#[allow(dead_code)]
pub fn get_tool_params_of_id(id: &TypeId) -> &'static EditorToolParams {
    get_tool_instance_of_id(id).get_params()
}

#[allow(dead_code)]
pub fn get_tool_params<T: EditorTool + 'static>() -> &'static EditorToolParams {
    let id = TypeId::of::<T>();
    get_tool_params_of_id(&id)
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
