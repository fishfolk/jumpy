use macroquad::{
    ui::{
        self,
        widgets,
        Id,
        Ui,
        hash,
    },
    experimental::{
        collections::storage,
    },
    prelude::*,
};

use crate::{
    gui::GuiResources,
    map::Map,
    editor::{
        EditorAction,
        EditorDrawParams,
        ContextMenuEntry,
    },
};

mod builder;
pub use builder::{
    ToolbarDrawFunc,
    ToolbarElementBuilder
};

mod layer_list;
pub use layer_list::create_layer_list_element;

mod tileset_list;
pub use tileset_list::create_tileset_list_element;

mod tileset_details;
pub use tileset_details::create_tileset_details_element;

#[derive(Clone)]
pub struct  ToolbarElement {
    id: Id,
    header: Option<String>,
    width: f32,
    height_factor: f32,
    draw_func: ToolbarDrawFunc,
}

impl ToolbarElement {
    pub fn get_height(&self) -> f32 {
        screen_height() * self.height_factor
    }

    pub fn get_size(&self) -> Vec2 {
        let x = self.width;
        let y = self.get_height();
        vec2(x, y)
    }

    pub fn draw(&self, ui: &mut Ui, map: &Map, params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let mut position = Vec2::ZERO;
        let mut size = self.get_size();

        if let Some(header) = &self.header {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.editor_skins.toolbar_header_bg);

            let header_height= ui.calc_size(header).y;

            {
                let size = vec2(size.x, header_height);

                widgets::Button::new("").position(position).size(size).ui(ui);
                ui.label(position, header);
            }

            size.y -= header_height;
            position.y += header_height;

            ui.pop_skin();
        }

        widgets::Group::new(hash!(self.id, "element"), size).position(position).ui(ui, |ui| {
            res = (self.draw_func)(ui, self.id, size, map, params);
        });

        res
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ToolbarPosition {
    Left,
    Right,
}

pub struct Toolbar {
    pub width: f32,
    position: ToolbarPosition,
    elements: Vec<ToolbarElement>,
}

impl Toolbar {
    pub const LIST_ENTRY_HEIGHT: f32 = 25.0;
    pub const SEPARATOR_HEIGHT: f32 = 4.0;
    pub const BUTTON_BAR_BUTTON_HEIGHT: f32 = 25.0;
    pub const BUTTON_BAR_TOTAL_HEIGHT: f32 = Self::BUTTON_BAR_BUTTON_HEIGHT + (Self::SEPARATOR_HEIGHT * 2.0);

    pub fn new(position: ToolbarPosition, width: f32, elements: &[ToolbarElement]) -> Self {
        let elements = elements.to_vec();

        Toolbar {
            position,
            width,
            elements,
        }
    }

    pub fn get_rect(&self) -> Rect {
        let mut offset = 0.0;
        if self.position == ToolbarPosition::Right {
            offset += screen_width() - self.width;
        }

        Rect::new(offset, 0.0, self.width, screen_height())
    }

    pub fn contains(&self, point: Vec2) -> bool {
        let rect = self.get_rect();
        rect.contains(point)
    }

    pub fn draw(&mut self, ui: &mut Ui, map: &Map, params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let gui_resources = storage::get::<GuiResources>();
        ui.push_skin(&gui_resources.editor_skins.toolbar);

        let mut position = Vec2::ZERO;
        if self.position == ToolbarPosition::Right {
            position.x += screen_width() - self.width;
        }

        let mut size = vec2(self.width, screen_height());

        {
            let mut total_height_factor = 0.0;
            for element in &self.elements {
                total_height_factor += element.height_factor;
            }

            assert!(total_height_factor <= 1.0, "Total height factor of all toolbar elements exceed 1.0");
        }

        widgets::Group::new(hash!(self.position, "toolbar_group"), size).position(position).ui(ui, |ui| {
            let mut position = Vec2::ZERO;

            ui.push_skin(&gui_resources.editor_skins.toolbar_bg);
            widgets::Button::new("").position(position).size(size).ui(ui);
            ui.pop_skin();

            for element in &mut self.elements {
                let size = element.get_size();

                widgets::Group::new(hash!(self.position, element.id, "outer_group"), size).position(position).ui(ui, |ui| {
                    if let Some(action) = element.draw(ui, map, params) {
                        // If this is not handled in this way, the result doesn't register, for some reason...
                        res = Some(action);
                    }
                });

                position.y += size.y;
            }
        });

        ui.pop_skin();

        res
    }
}

pub fn create_left_toolbar(width: f32) -> Toolbar {
    Toolbar::new(ToolbarPosition::Left, width, &[
    ])
}

const LAYER_LIST_HEIGHT_FACTOR: f32 = 0.2;
const TILESET_LIST_HEIGHT_FACTOR: f32 = 0.3;
const TILESET_DETAILS_HEIGHT_FACTOR: f32 = 0.5;

pub fn create_right_toolbar(width: f32) -> Toolbar {
    Toolbar::new(ToolbarPosition::Right, width, &[
        create_layer_list_element(width, LAYER_LIST_HEIGHT_FACTOR),
        create_tileset_list_element(width, TILESET_LIST_HEIGHT_FACTOR),
        create_tileset_details_element(width, TILESET_DETAILS_HEIGHT_FACTOR),
    ])
}
