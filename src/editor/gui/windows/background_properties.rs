use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{hash, widgets, Ui},
};

use crate::map::{MapBackgroundLayer, MapObject};
use crate::{
    editor::gui::{ComboBoxBuilder, ComboBoxValue},
    map::{Map, MapObjectKind},
    Resources,
};
use crate::editor::gui::ELEMENT_MARGIN;
use crate::gui::GuiResources;
use crate::resources::TextureKind;

use super::{ButtonParams, EditorAction, EditorContext, Window, WindowParams};

pub struct BackgroundPropertiesWindow {
    params: WindowParams,
    color: Color,
    layers: Vec<MapBackgroundLayer>,
    layer_texture_id: Option<String>,
    layer_depth: Option<f32>,
}

impl BackgroundPropertiesWindow {
    const LAYER_LIST_ENTRY_HEIGHT: f32 = 24.0;

    pub fn new(color: Color, layers: Vec<MapBackgroundLayer>) -> Self {
        let params = WindowParams {
            title: Some("Background Properties".to_string()),
            size: vec2(350.0, 500.0),
            ..Default::default()
        };

        BackgroundPropertiesWindow {
            params,
            color,
            layers,
            layer_texture_id: None,
            layer_depth: None,
        }
    }
}

impl Window for BackgroundPropertiesWindow {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn get_buttons(&self, _map: &Map, _ctx: &EditorContext) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let action = self.get_close_action().then(EditorAction::UpdateBackground {
            color: self.color,
            layers: self.layers.clone(),
        });

        res.push(ButtonParams {
            label: "Save",
            action: Some(action),
            ..Default::default()
        });

        res.push(ButtonParams {
            label: "Cancel",
            action: Some(self.get_close_action()),
            ..Default::default()
        });

        res
    }

    fn draw(
        &mut self,
        ui: &mut Ui,
        size: Vec2,
        _map: &Map,
        _ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let id = hash!("background_properties_window");

        widgets::Group::new(hash!(id, "color_group"), vec2((size.x * 0.35), size.y * 0.5))
            .position(vec2(0.0, 0.0))
            .ui(ui, |ui| {
                let mut r_str = self.color.r.to_string();
                let mut g_str = self.color.g.to_string();
                let mut b_str = self.color.b.to_string();
                let mut a_str = self.color.a.to_string();

                widgets::InputText::new(hash!(id, "color_r_input"))
                    .ratio(1.0)
                    .label("r")
                    .ui(ui, &mut r_str);

                widgets::InputText::new(hash!(id, "color_g_input"))
                    .ratio(1.0)
                    .label("g")
                    .ui(ui, &mut g_str);

                widgets::InputText::new(hash!(id, "color_b_input"))
                    .ratio(1.0)
                    .label("b")
                    .ui(ui, &mut b_str);

                widgets::InputText::new(hash!(id, "color_a_input"))
                    .ratio(1.0)
                    .label("a")
                    .ui(ui, &mut a_str);

                if let Ok(r) = r_str.parse::<f32>() {
                    self.color.r = r;
                }

                if let Ok(g) = g_str.parse::<f32>() {
                    self.color.g = g;
                }

                if let Ok(b) = b_str.parse::<f32>() {
                    self.color.b = b;
                }

                if let Ok(a) = a_str.parse::<f32>() {
                    self.color.a = a;
                }
            });

        let layer_list_size = vec2((size.x * 0.65) - ELEMENT_MARGIN, size.y * 0.5);

        widgets::Group::new(hash!(id, "layer_list"), layer_list_size)
            .position(vec2((size.x * 0.35) + ELEMENT_MARGIN, 0.0))
            .ui(ui, |ui| {
                let mut i = 0;
                while i < self.layers.len() {
                    let layer = self.layers.get(i).unwrap();

                    let mut should_delete = false;

                    widgets::Group::new(hash!(id, "layer_list_entry", i), vec2(layer_list_size.x, Self::LAYER_LIST_ENTRY_HEIGHT))
                        .position(vec2(0.0, i as f32 * Self::LAYER_LIST_ENTRY_HEIGHT))
                        .ui(ui, |ui| {
                            ui.label(vec2(0.0, 0.0), &layer.texture_id);
                        });

                    if should_delete {
                        self.layers.remove(i);
                        continue;
                    }

                    i += 1;
                }
            });


        widgets::Group::new(hash!(id, "layer_attributes"), vec2(size.x, (size.y * 0.5) - ELEMENT_MARGIN))
            .position(vec2(0.0, (size.y * 0.5) + ELEMENT_MARGIN))
            .ui(ui, |ui| {
                let resources = storage::get::<Resources>();
                let mut texture_ids = resources.textures
                    .values()
                    .filter_map(|texture_res| {
                        let mut res = None;

                        if let Some(kind) = texture_res.meta.kind {
                            if kind == TextureKind::Background {
                                res = Some(texture_res.meta.id.as_str());
                            }
                        }

                        res
                    })
                    .collect::<Vec<&str>>();

                texture_ids.sort();

                let mut texture_index = texture_ids
                    .iter()
                    .enumerate()
                    .find_map(|(i, id)| {
                        if let Some(texture_id) = &self.layer_texture_id {
                            if *id == texture_id {
                                return Some(i);
                            }
                        }

                        None
                    })
                    .unwrap_or(0);

                widgets::ComboBox::new(hash!(id, "layer_texture_input"), &texture_ids)
                    .ratio(0.8)
                    .label("Texture")
                    .ui(ui, &mut texture_index);

                self.layer_texture_id = texture_ids
                    .get(texture_index)
                    .map(|str| {
                        str.to_string()
                    });

                let mut depth_str = self.layer_depth
                    .unwrap_or(0.0)
                    .to_string();

                widgets::InputText::new(hash!(id, "layer_depth_input"))
                    .ratio(0.4)
                    .label("Depth")
                    .ui(ui, &mut depth_str);

                if let Ok(depth) = depth_str.parse::<f32>() {
                    self.layer_depth = Some(depth);
                }

                ui.same_line(0.0);

                let add_btn = widgets::Button::new("Add");

                if add_btn.ui(ui) && self.layer_texture_id.is_some() && self.layer_depth.is_some() {
                    let texture_id = self.layer_texture_id.take().unwrap();
                    let depth = self.layer_depth.take().unwrap();

                    self.layers.push(MapBackgroundLayer {
                        texture_id,
                        depth,
                        offset: Vec2::ZERO,
                    });
                }
            });

        None
    }
}
