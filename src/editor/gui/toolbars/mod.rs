use std::{any::TypeId, collections::HashMap};

use ff_core::prelude::*;

use ff_core::gui::{get_gui_theme, ELEMENT_MARGIN};
use ff_core::map::Map;

use super::{ButtonParams, EditorAction, EditorContext};

mod tool_selector;

pub use tool_selector::ToolSelectorElement;

mod layer_list;

pub use layer_list::LayerListElement;

mod tileset_list;

pub use tileset_list::TilesetListElement;

mod tileset_details;

pub use tileset_details::TilesetDetailsElement;

mod object_list;

use ff_core::macroquad::hash;
use ff_core::macroquad::ui::{widgets, Ui};
pub use object_list::ObjectListElement;

#[derive(Debug, Default, Clone)]
pub struct ToolbarElementParams {
    header: Option<String>,
    has_margins: bool,
    has_buttons: bool,
}

pub trait ToolbarElement {
    fn get_params(&self) -> &ToolbarElementParams;

    // The `size` parameter will be the size of the group containing the element
    fn draw(
        &mut self,
        ui: &mut Ui,
        size: Vec2,
        map: &Map,
        ctx: &EditorContext,
    ) -> Option<EditorAction>;

    // Implement this and set `has_buttons` to true in the `ToolbarElementParams` returned by
    // `get_params` to add a button bar to the bottom of the element.
    // There can be no more than four buttons, each represented by a `ButtonParam` struct, in
    // the `Vec` returned by this method.
    fn get_buttons(&self, _map: &Map, _ctx: &EditorContext) -> Vec<ButtonParams> {
        Vec::new()
    }

    // This controls whether the element is drawn or not
    fn is_drawn(&self, _map: &Map, _ctx: &EditorContext) -> bool {
        true
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ToolbarPosition {
    Left,
    Right,
}

pub struct Toolbar {
    pub width: f32,
    pub position: ToolbarPosition,
    draw_order: Vec<TypeId>,
    elements: HashMap<TypeId, (f32, Box<dyn ToolbarElement>)>,
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

    #[must_use]
    pub fn with_element<E: ToolbarElement + 'static>(self, height_factor: f32, element: E) -> Self {
        let mut res = self;
        res.add_element(height_factor, element);
        res
    }

    pub fn add_element<E: ToolbarElement + 'static>(&mut self, height_factor: f32, element: E) {
        let id = TypeId::of::<E>();
        self.elements.insert(id, (height_factor, Box::new(element)));
        self.draw_order.push(id);
    }

    pub fn remove_element<E: ToolbarElement + 'static>(
        &mut self,
    ) -> Option<Box<dyn ToolbarElement>> {
        let id = TypeId::of::<E>();
        self.draw_order.retain(|other_id| *other_id != id);
        self.elements.remove(&id).map(|(_, id)| id)
    }

    pub fn get_rect(&self) -> Rect {
        let mut offset = 0.0;

        let viewport_size = viewport_size();

        if self.position == ToolbarPosition::Right {
            offset += viewport_size.width - self.width;
        }

        Rect::new(offset, 0.0, self.width, viewport_size.height)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        let rect = self.get_rect();
        rect.contains(point)
    }

    pub fn draw(&mut self, ui: &mut Ui, map: &Map, ctx: &EditorContext) -> Option<EditorAction> {
        let mut res = None;

        {
            let gui_theme = get_gui_theme();
            ui.push_skin(&gui_theme.toolbar);
        }

        let viewport_size = viewport_size();

        let mut position = Vec2::ZERO;
        if self.position == ToolbarPosition::Right {
            position.x += viewport_size.width - self.width;
        }

        let toolbar_id = hash!(self.position);
        let toolbar_size = vec2(self.width, viewport_size.height);

        widgets::Group::new(toolbar_id, toolbar_size)
            .position(position)
            .ui(ui, |ui| {
                let mut position = Vec2::ZERO;

                {
                    let gui_theme = get_gui_theme();
                    ui.push_skin(&gui_theme.toolbar_bg);
                    widgets::Button::new("")
                        .position(position)
                        .size(toolbar_size)
                        .ui(ui);
                    ui.pop_skin();
                }

                for element_id in &self.draw_order {
                    let (height_factor, element) = self
                        .elements
                        .get_mut(element_id)
                        .map(|(height_factor, element)| (*height_factor, element))
                        .unwrap();

                    if element.is_drawn(map, ctx) {
                        let params = element.get_params().clone();

                        let element_id = hash!(toolbar_id, element_id);

                        let element_size = {
                            let height = viewport_size.height * height_factor;
                            vec2(self.width, height)
                        };

                        let element_position = position;
                        let mut content_position = element_position;
                        let mut content_size = element_size;

                        if let Some(header) = &params.header {
                            let gui_theme = get_gui_theme();
                            ui.push_skin(&gui_theme.toolbar_header_bg);

                            let header_height = ui.calc_size(header).y;

                            {
                                let size = vec2(toolbar_size.x, header_height);

                                widgets::Button::new("")
                                    .position(element_position)
                                    .size(size)
                                    .ui(ui);
                                ui.label(element_position, header);
                            }

                            content_size.y -= header_height;
                            content_position.y += header_height;

                            ui.pop_skin();
                        }

                        if params.has_buttons {
                            content_size.y -= Toolbar::BUTTON_HEIGHT + (ELEMENT_MARGIN * 2.0);
                        }

                        let margins = vec2(ELEMENT_MARGIN, ELEMENT_MARGIN);
                        if params.has_margins {
                            content_size -= vec2(margins.x, margins.y * 2.0);
                            content_position += margins;
                        }

                        widgets::Group::new(hash!(element_id, "content"), content_size)
                            .position(content_position)
                            .ui(ui, |ui| {
                                content_size.x -= margins.x;
                                if let Some(action) = element.draw(ui, content_size, map, ctx) {
                                    res = Some(action);
                                }
                            });

                        if params.has_buttons {
                            let mut menubar_position =
                                vec2(element_position.x, content_position.y) + margins;
                            menubar_position.y += content_size.y;

                            let mut menubar_size = vec2(element_size.x, Toolbar::BUTTON_HEIGHT);
                            menubar_size.x -= margins.x * 2.0;

                            widgets::Group::new(hash!(element_id, "menubar"), menubar_size)
                                .position(menubar_position)
                                .ui(ui, |ui| {
                                    {
                                        let gui_theme = get_gui_theme();
                                        ui.push_skin(&gui_theme.toolbar_button);
                                    }

                                    let buttons = element.get_buttons(map, ctx);

                                    let button_cnt = buttons.len().clamp(0, 4);

                                    if button_cnt > 0 {
                                        let mut button_position = Vec2::ZERO;

                                        let total_width =
                                            menubar_size.x - (margins.x * (button_cnt - 1) as f32);

                                        let mut overrides = 0;
                                        let mut total_override_factor = 0.0;
                                        for i in 0..button_cnt {
                                            let button = buttons.get(i).unwrap();
                                            if let Some(width_factor) = button.width_override {
                                                total_override_factor +=
                                                    to_corrected_button_width_factor(width_factor);
                                                overrides += 1;
                                            }
                                        }

                                        let auto_width = {
                                            let remaining_buttons = button_cnt - overrides;
                                            let remaining_width_factor =
                                                1.0 - total_override_factor;
                                            let width_factor =
                                                remaining_width_factor / remaining_buttons as f32;
                                            total_width
                                                * to_corrected_button_width_factor(width_factor)
                                        };

                                        let mut i = 0;
                                        for button in buttons {
                                            let mut button_size =
                                                vec2(auto_width, Self::BUTTON_HEIGHT);
                                            if let Some(width_factor) = button.width_override {
                                                button_size.x = total_width
                                                    * to_corrected_button_width_factor(
                                                        width_factor,
                                                    );
                                            }

                                            if button.action.is_none() {
                                                let gui_theme = get_gui_theme();
                                                ui.push_skin(&gui_theme.toolbar_button_disabled);
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

                                            button_position.x += button_size.x + margins.x;

                                            i += 1;

                                            if i >= button_cnt {
                                                break;
                                            }
                                        }
                                    }

                                    ui.pop_skin();
                                });
                        }

                        position.y += element_size.y;
                    }
                }
            });

        ui.pop_skin();

        res
    }
}

fn to_corrected_button_width_factor(width_factor: f32) -> f32 {
    if width_factor <= 0.25 {
        0.25
    } else {
        0.5
    }
}
