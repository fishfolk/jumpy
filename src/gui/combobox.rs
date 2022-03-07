use core::prelude::*;

use crate::macroquad::ui::widgets;

pub trait ComboBoxValue {
    fn get_index(&self) -> usize;

    fn get_options(&self) -> Vec<String>;

    fn set_index(&mut self, index: usize);

    fn get_value(&self) -> String {
        self.get_options()
            .get(self.get_index())
            .unwrap()
            .to_string()
    }
}

pub struct ComboBoxVec {
    index: usize,
    options: Vec<String>,
}

impl ComboBoxVec {
    pub fn new(index: usize, options: &[&str]) -> Self {
        let options = options.iter().map(|s| s.to_string()).collect();

        ComboBoxVec { index, options }
    }

    pub fn set_value(&mut self, value: &str) {
        for (i, v) in self.options.iter().enumerate() {
            if *v == *value {
                self.index = i;
            }
        }
    }
}

impl ComboBoxValue for ComboBoxVec {
    fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn get_options(&self) -> Vec<String> {
        self.options.clone()
    }
}

impl From<&[&str]> for ComboBoxVec {
    fn from(slice: &[&str]) -> Self {
        ComboBoxVec::new(0, slice)
    }
}

impl From<&[String]> for ComboBoxVec {
    fn from(slice: &[String]) -> Self {
        let slice = slice.iter().map(|s| s.as_str()).collect::<Vec<_>>();

        ComboBoxVec::new(0, &slice)
    }
}

impl From<&ComboBoxVec> for usize {
    fn from(v: &ComboBoxVec) -> Self {
        v.get_index()
    }
}

cfg_if! {
    if #[cfg(not(feature = "ultimate"))] {
        use core::macroquad::ui::{Ui, Id};

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

            #[must_use]
            pub fn with_label(self, label: &str) -> Self {
                ComboBoxBuilder {
                    label: Some(label.to_string()),
                    ..self
                }
            }

            #[must_use]
            pub fn with_ratio(self, ratio: f32) -> Self {
                ComboBoxBuilder {
                    ratio: Some(ratio),
                    ..self
                }
            }

            pub fn build<V: ComboBoxValue>(&self, ui: &mut Ui, value: &mut V) {
                let mut index = value.get_index();

                let owned = value.get_options();
                let options = owned.iter().map(String::as_str).collect::<Vec<_>>();

                let mut combobox = widgets::ComboBox::new(self.id, &options);

                if let Some(ratio) = self.ratio {
                    combobox = combobox.ratio(ratio);
                }

                if let Some(label) = &self.label {
                    combobox = combobox.label(label);
                }

                combobox.ui(ui, &mut index);

                value.set_index(index);
            }
        }
    }
}