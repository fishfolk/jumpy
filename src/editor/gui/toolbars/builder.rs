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

use super::{
    ToolbarPosition,
    ToolbarElement,
};

pub type ToolbarDrawFunc = fn (&mut Ui, Id, Vec2, &Map, &EditorDrawParams) -> Option<EditorAction>;

pub struct ToolbarElementBuilder {
    header: Option<String>,
    width: f32,
    height_factor: f32,
    menubar_draw_func: Option<ToolbarDrawFunc>,
    has_margins: bool,
}

impl ToolbarElementBuilder {
    pub fn new(width: f32, height_factor: f32) -> Self {
        ToolbarElementBuilder {
            header: None,
            width,
            height_factor,
            menubar_draw_func: None,
            has_margins: false,
        }
    }

    pub fn with_header(self, header: &str) -> Self {
        ToolbarElementBuilder {
            header: Some(header.to_string()),
            ..self
        }
    }

    pub fn with_margins(self) -> Self {
        ToolbarElementBuilder {
            has_margins: true,
            ..self
        }
    }

    pub fn with_menubar(self, draw_func: ToolbarDrawFunc) -> Self {
        ToolbarElementBuilder {
            menubar_draw_func: Some(draw_func),
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
            menubar_draw_func: self.menubar_draw_func,
            has_margins: self.has_margins,
        }
    }
}
