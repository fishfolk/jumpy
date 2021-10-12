use macroquad::{
    ui::{
        Ui,
        Id,
        widgets,
    },
    prelude::*,
};

pub trait ComboBoxValue {
    fn from_index(index: usize) -> Self;

    fn to_index(&self) -> usize;

    fn options() -> &'static [&'static str];
}

pub struct ComboBoxBuilder {
    id: Id,
    label: Option<String>,
    ratio: Option<f32>,
}

impl ComboBoxBuilder {
    pub fn new(id: Id) -> Self {
        ComboBoxBuilder {
            id,
            label: None,
            ratio: None,
        }
    }

    pub fn with_label(self, label: &str) -> Self {
        ComboBoxBuilder {
            label: Some(label.to_string()),
            ..self
        }
    }

    pub fn with_ratio(self, ratio: f32) -> Self {
        ComboBoxBuilder {
            ratio: Some(ratio),
            ..self
        }
    }

    pub fn build<V: ComboBoxValue>(&self, ui: &mut Ui, value: &mut V) {
        let mut value_index = value.to_index();

        let mut combobox = widgets::ComboBox::new(self.id, V::options());

        if let Some(ratio) = self.ratio {
            combobox = combobox.ratio(ratio);
        }

        if let Some(label) = &self.label {
            combobox = combobox.label(label);
        }

        combobox.ui(ui, &mut value_index);

        *value = V::from_index(value_index);
    }
}