use macroquad::{
    ui::{
        Id,
        Ui,
    },
    prelude::*,
};

use crate::{
    editor::{
        EditorAction,
        EditorDrawParams,
    },
    map::Map,
};

use super::ToolbarElement;

pub type ToolbarDrawFunc = fn (&mut Ui, Id, Vec2, &Map, &EditorDrawParams) -> Option<EditorAction>;

pub struct ToolbarElementBuilder {
    header: Option<String>,
    width: f32,
    height_factor: f32,
}

impl ToolbarElementBuilder {
    pub fn new(width: f32, height_factor: f32) -> Self {
        ToolbarElementBuilder {
            header: None,
            width,
            height_factor,
        }
    }

    pub fn with_header(self, header: &str) -> Self {
        ToolbarElementBuilder {
            header: Some(header.to_string()),
            ..self
        }
    }

    pub fn build(self, id: Id, draw_func: ToolbarDrawFunc) -> ToolbarElement {
        ToolbarElement {
            id,
            header: self.header,
            width: self.width,
            height_factor: self.height_factor,
            draw_func,
        }
    }
}
