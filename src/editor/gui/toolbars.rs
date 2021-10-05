use std::{
    any::{
        TypeId,
    },
    collections::HashMap,
};

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
};

use super::{
    ButtonParams,
    EditorAction,
    EditorDrawParams,
    ELEMENT_MARGIN,
};

mod tool_selector;

pub use tool_selector::ToolSelectorElement;

mod layer_list;

pub use layer_list::LayerListElement;

mod tileset_list;

pub use tileset_list::TilesetListElement;

mod tileset_details;

pub use tileset_details::TilesetDetailsElement;

#[derive(Debug, Default, Clone)]
pub struct ToolbarElementParams {
    header: Option<String>,
    has_buttons: bool,
    has_margins: bool,
}

pub trait ToolbarElement {
    fn get_params(&self) -> &ToolbarElementParams;

    fn draw(&mut self, ui: &mut Ui, size: Vec2, map: &Map, draw_params: &EditorDrawParams) -> Option<EditorAction>;

    fn get_buttons(&self, _map: &Map, _draw_params: &EditorDrawParams) -> Vec<ButtonParams> {
        Vec::new()
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
    draw_order: Vec<(TypeId, f32)>,
    elements: HashMap<TypeId, Box<dyn ToolbarElement>>,
}

impl Toolbar {
    pub const LIST_ENTRY_HEIGHT: f32 = 25.0;

    pub const BUTTON_HEIGHT: f32 = 25.0;

    pub fn new(position: ToolbarPosition, width: f32) -> Self {
        Toolbar {
            position,
            width,
            draw_order: Vec::new(),
            elements: HashMap::new(),
        }
    }

    pub fn with_element<E: ToolbarElement + 'static>(self, height_factor: f32, element: E) -> Self {
        let mut res = self;
        res.add_element(height_factor, element);
        res
    }

    pub fn add_element<E: ToolbarElement + 'static>(&mut self, height_factor: f32, element: E) {
        let key = TypeId::of::<E>();

        let entry = (key, height_factor);
        self.draw_order.push(entry);

        let value = Box::new(element);
        self.elements.insert(key, value);
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
            for (_, height_factor) in &self.draw_order {
                total_height_factor += *height_factor;
            }

            assert!(total_height_factor <= 1.0, "Total height factor of all toolbar elements exceed 1.0");
        }

        let toolbar_id = hash!(self.position);

        widgets::Group::new(toolbar_id, toolbar_size).position(position).ui(ui, |ui| {
            let mut position = Vec2::ZERO;

            {
                let gui_resources = storage::get::<GuiResources>();
                ui.push_skin(&gui_resources.editor_skins.toolbar_bg);
                widgets::Button::new("").position(position).size(toolbar_size).ui(ui);
                ui.pop_skin();
            }

            for (element_id, height_factor) in self.draw_order.clone() {
                let element = self.elements.get_mut(&element_id).unwrap();

                let params = element.get_params().clone();

                let element_id = hash!(toolbar_id, element_id);

                let element_size = {
                    let height = screen_height() * height_factor;
                    vec2(self.width, height)
                };

                let element_position = position;
                let mut content_position = element_position;
                let mut content_size = element_size;

                if let Some(header) = &params.header {
                    let gui_resources = storage::get::<GuiResources>();
                    ui.push_skin(&gui_resources.editor_skins.toolbar_header_bg);

                    let header_height = ui.calc_size(header).y;

                    {
                        let size = vec2(toolbar_size.x, header_height);

                        widgets::Button::new("").position(element_position).size(size).ui(ui);
                        ui.label(element_position, header);
                    }

                    content_size.y -= header_height;
                    content_position.y += header_height;

                    ui.pop_skin();
                }

                if params.has_buttons {
                    content_size.y -= Toolbar::BUTTON_HEIGHT + (ELEMENT_MARGIN * 2.0);
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

                if params.has_buttons {
                    let mut menubar_position = vec2(element_position.x, content_position.y);
                    menubar_position.y += content_size.y + ELEMENT_MARGIN;
                    menubar_position.x += ELEMENT_MARGIN;

                    let mut menubar_size = vec2(element_size.x, Toolbar::BUTTON_HEIGHT);
                    menubar_size.x -= ELEMENT_MARGIN * 2.0;

                    widgets::Group::new(hash!(element_id, "menubar"), menubar_size).position(menubar_position).ui(ui, |ui| {
                        let buttons = element.get_buttons(map, draw_params);

                        let button_cnt = buttons
                            .len()
                            .clamp(0, 4);

                        if button_cnt > 0 {
                            let mut button_position = Vec2::ZERO;

                            let total_width = menubar_size.x - (ELEMENT_MARGIN * 3.0);

                            let mut overrides = 0;
                            let mut total_override_factor = 0.0;
                            for i in 0..button_cnt {
                                let button = buttons.get(i).unwrap();
                                if let Some(width_factor) = button.width_override {
                                    total_override_factor += to_corrected_button_width_factor(width_factor);
                                    overrides += 1;
                                }
                            }

                            let auto_width = {
                                let remaining_buttons = button_cnt - overrides;
                                let remaining_width_factor = 1.0 - total_override_factor;
                                let width_factor = remaining_width_factor / remaining_buttons as f32;
                                total_width * to_corrected_button_width_factor(width_factor)
                            };

                            let mut i = 0;
                            for button in buttons {
                                let mut button_size = vec2(auto_width, Self::BUTTON_HEIGHT);
                                if let Some(width_factor) = button.width_override {
                                    button_size.x = total_width * to_corrected_button_width_factor(width_factor);
                                }

                                if button.action.is_none() {
                                    let gui_resources = storage::get::<GuiResources>();
                                    ui.push_skin(&gui_resources.editor_skins.toolbar_button_disabled);
                                }

                                let was_clicked = widgets::Button::new(button.label)
                                    .size(button_size)
                                    .position(button_position)
                                    .ui(ui);

                                if button.action.is_some() {
                                    if was_clicked {
                                        res = button.action;
                                    }
                                } else {
                                    ui.pop_skin();
                                }

                                button_position.x += button_size.x + ELEMENT_MARGIN;

                                i += 1;

                                if i >= button_cnt {
                                    break;
                                }
                            }
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

fn to_corrected_button_width_factor(width_factor: f32) -> f32 {
    if width_factor <= 0.25 {
        0.25
    } else if width_factor <= 0.5 {
        0.5
    } else if width_factor <= 0.75 {
        0.75
    } else {
        1.0
    }
}