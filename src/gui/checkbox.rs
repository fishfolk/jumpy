use std::ops::Deref;

use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{widgets, Id, Ui},
};

use crate::gui::{GuiResources, ELEMENT_MARGIN};

pub struct Checkbox {
    id: Id,
    position: Option<Vec2>,
    label: String,
    allow_click_on_label: bool,
}

impl Checkbox {
    const ALLOW_CLICK_ON_LABEL: bool = true;

    pub fn new<P: Into<Option<Vec2>>>(id: Id, position: P, label: &str) -> Self {
        Checkbox {
            id,
            position: position.into(),
            label: label.to_string(),
            allow_click_on_label: Self::ALLOW_CLICK_ON_LABEL,
        }
    }

    #[allow(dead_code)]
    pub fn with_inactive_label(self) -> Self {
        Checkbox {
            id: self.id,
            position: self.position,
            label: self.label,
            allow_click_on_label: false,
        }
    }

    pub fn ui(&self, ui: &mut Ui, value: &mut bool) {
        let gui_resources = storage::get::<GuiResources>();

        if *value {
            ui.push_skin(&gui_resources.skins.checkbox_selected);
        } else {
            ui.push_skin(&gui_resources.skins.checkbox);
        }

        ui.separator();

        let label_size = ui.calc_size(&self.label);
        let label_height = label_size.y - (ELEMENT_MARGIN * 2.0);
        let checkbox_size = vec2(label_height, label_height);
        let total_size = vec2(checkbox_size.x * 1.5, 0.0) + label_size;

        let mut group = widgets::Group::new(self.id, total_size);

        if let Some(position) = &self.position {
            group = group.position(*position);
        }

        group.ui(ui, |ui| {
            let checkbox = widgets::Button::new("")
                .position(vec2(0.0, ELEMENT_MARGIN))
                .size(checkbox_size)
                .ui(ui);

            if checkbox {
                *value = !*value;
            }

            ui.push_skin(&gui_resources.skins.label_button);
            let label_btn = widgets::Button::new(self.label.deref())
                .position(vec2(checkbox_size.x * 1.5, 0.0))
                .ui(ui);
            ui.pop_skin();

            if label_btn && self.allow_click_on_label {
                *value = !*value;
            }
        });

        ui.pop_skin();
    }
}
