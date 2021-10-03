use macroquad::{
    ui::{
        widgets,
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
    },
};

use super::ELEMENT_MARGIN;

mod layer_list;
pub use layer_list::LayerListElement;

mod tileset_list;
pub use tileset_list::TilesetListElement;

mod tileset_details;
pub use tileset_details::TilesetDetailsElement;

#[derive(Debug, Clone)]
pub struct ToolbarElementParams {
    header: Option<String>,
    has_menubar: bool,
    has_margins: bool,
}

impl Default for ToolbarElementParams {
    fn default() -> Self {
        ToolbarElementParams {
            header: None,
            has_menubar: false,
            has_margins: false,
        }
    }
}

pub trait ToolbarElement {
    fn get_params(&self) -> ToolbarElementParams;

    fn draw(&mut self, ui: &mut Ui, size: Vec2, map: &Map, draw_params: &EditorDrawParams) -> Option<EditorAction>;

    fn draw_menubar(&mut self, _ui: &mut Ui, _size: Vec2, _map: &Map, _draw_params: &EditorDrawParams) -> Option<EditorAction> {
        None
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
    elements: Vec<(f32, Box<dyn ToolbarElement>)>,
}

impl Toolbar {
    pub const MARGIN: f32 = 8.0;

    pub const LIST_ENTRY_HEIGHT: f32 = 25.0;
    pub const MENUBAR_HEIGHT: f32 = 25.0;
    pub const MENUBAR_TOTAL_HEIGHT: f32 = Self::MENUBAR_HEIGHT + (Self::MARGIN * 2.0);

    pub fn new(position: ToolbarPosition, width: f32) -> Self {
        Toolbar {
            position,
            width,
            elements: Vec::new(),
        }
    }

    pub fn with_element(self, height_factor: f32, element: Box<dyn ToolbarElement>) -> Self {
        let mut elements = self.elements;
        elements.push((height_factor, element));

        Toolbar {
            elements,
            ..self
        }
    }

    pub fn add_element(&mut self, height_factor: f32, element: Box<dyn ToolbarElement>) {
        self.elements.push((height_factor, element));
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

    pub fn draw(&mut self, ui: &mut Ui, map: &Map, draw_params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.editor_skins.toolbar);
        }

        let mut position = Vec2::ZERO;
        if self.position == ToolbarPosition::Right {
            position.x += screen_width() - self.width;
        }

        let toolbar_size = vec2(self.width, screen_height());

        {
            let mut total_height_factor = 0.0;
            for (height_factor, _) in &self.elements {
                total_height_factor += *height_factor;
            }

            assert!(total_height_factor <= 1.0, "Total height factor of all toolbar elements exceed 1.0");
        }

        let id = hash!(self.position);

        widgets::Group::new(id, toolbar_size).position(position).ui(ui, |ui| {
            let mut position = Vec2::ZERO;

            {
                let gui_resources = storage::get::<GuiResources>();
                ui.push_skin(&gui_resources.editor_skins.toolbar_bg);
                widgets::Button::new("").position(position).size(toolbar_size).ui(ui);
                ui.pop_skin();
            }

            let mut i = 0;
            for (height_factor, element) in &mut self.elements {
                let params = element.get_params();

                let element_id = hash!(id, "element", i);
                i += 1;

                let element_size = {
                    let height = screen_height() * *height_factor;
                    vec2(self.width, height)
                };

                let element_position = position;
                let mut content_position = element_position;
                let mut content_size = element_size;

                if let Some(header) = params.header {
                    let gui_resources = storage::get::<GuiResources>();
                    ui.push_skin(&gui_resources.editor_skins.toolbar_header_bg);

                    let header_height= ui.calc_size(&header).y;

                    {
                        let size = vec2(toolbar_size.x, header_height);

                        widgets::Button::new("").position(element_position).size(size).ui(ui);
                        ui.label(element_position, &header);
                    }

                    content_size.y -= header_height;
                    content_position.y += header_height;

                    ui.pop_skin();
                }

                if params.has_menubar {
                    content_size.y -= Toolbar::MENUBAR_TOTAL_HEIGHT;
                }

                if params.has_margins {
                    content_size.x -= ELEMENT_MARGIN;
                    content_position.x += ELEMENT_MARGIN;
                }

                {
                    let has_margins = params.has_margins;

                    widgets::Group::new(element_id, content_size).position(content_position).ui(ui, |ui| {
                        if has_margins {
                            // This is done here so that scrollbar is pushed to edge of screen, even when the element has margins
                            content_size.x -= ELEMENT_MARGIN;
                        }

                        if let Some(action) = element.draw(ui, content_size, map, draw_params) {
                            res = Some(action);
                        }
                    });
                }

                if params.has_menubar {
                    let mut menubar_position = vec2(element_position.x, content_position.y);
                    menubar_position.y += content_size.y + ELEMENT_MARGIN;
                    menubar_position.x += ELEMENT_MARGIN;

                    let mut menubar_size = vec2(element_size.x, Toolbar::MENUBAR_HEIGHT);
                    menubar_size.x -= ELEMENT_MARGIN * 2.0;

                    widgets::Group::new(hash!(element_id, "menubar"), menubar_size).position(menubar_position).ui(ui, |ui| {
                        if let Some(action) = element.draw_menubar(ui, menubar_size, map, draw_params) {
                            res = Some(action);
                        }
                    });
                }

                position.y += element_size.y;
            }
        });

        ui.pop_skin();

        res
    }
}

