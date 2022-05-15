use std::ops::Deref;

use crate::gui::{widgets, Id, Ui};

use crate::math::{vec2, Vec2};

use crate::gui::theme::get_gui_theme;
use crate::gui::ELEMENT_MARGIN;

pub struct Checkbox {
    id: Id,
    margin: f32,
    position: Option<Vec2>,
    label: String,
    allow_click_on_label: bool,
}

impl Checkbox {
    const ALLOW_CLICK_ON_LABEL: bool = true;

    pub fn new<P: Into<Option<Vec2>>>(id: Id, position: P, label: &str) -> Self {
        Checkbox {
            id,
            margin: 0.0,
            position: position.into(),
            label: label.to_string(),
            allow_click_on_label: Self::ALLOW_CLICK_ON_LABEL,
        }
    }

    #[allow(dead_code)]
    pub fn with_inactive_label(self) -> Self {
        Checkbox {
            allow_click_on_label: false,
            ..self
        }
    }

    pub fn with_margin(self, margin: f32) -> Self {
        Checkbox { margin, ..self }
    }

    pub fn ui(&self, ui: &mut Ui, value: &mut bool) {
        let gui_theme = get_gui_theme();

        if *value {
            ui.push_skin(&gui_theme.checkbox_selected);
        } else {
            ui.push_skin(&gui_theme.checkbox);
        }

        ui.separator();

        let label_size = ui.calc_size(&self.label);
        let element_height = label_size.y * 0.75;
        let checkbox_size = vec2(element_height, element_height);
        let mut total_size = vec2(checkbox_size.x + ELEMENT_MARGIN, 0.0) + label_size;

        let mut position = None;
        if let Some(mut pos) = self.position {
            pos.x += self.margin;
            position = Some(pos);
        } else {
            total_size.x += self.margin;
        }

        let mut group = widgets::Group::new(self.id, total_size);

        if let Some(position) = position {
            group = group.position(position);
        }

        group.ui(ui, |ui| {
            let mut checkbox_position = vec2(0.0, (label_size.y - element_height) / 2.0);
            let mut label_position = vec2(checkbox_size.x + ELEMENT_MARGIN, 0.0);

            if position.is_none() {
                checkbox_position.x += self.margin;
                label_position.x += self.margin;
            }

            let checkbox = widgets::Button::new("")
                .position(checkbox_position)
                .size(checkbox_size)
                .ui(ui);

            if checkbox {
                *value = !*value;
            }

            ui.push_skin(&gui_theme.label_button);
            let label_btn = widgets::Button::new(self.label.deref())
                .position(label_position)
                .ui(ui);
            ui.pop_skin();

            if label_btn && self.allow_click_on_label {
                *value = !*value;
            }
        });

        ui.pop_skin();
    }
}
