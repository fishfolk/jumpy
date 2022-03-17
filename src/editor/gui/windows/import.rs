use ff_core::prelude::*;

use ff_core::gui::{checkbox::Checkbox, get_gui_theme, ELEMENT_MARGIN, theme::LIST_BOX_ENTRY_HEIGHT};

use ff_core::macroquad::hash;
use ff_core::macroquad::ui::{Ui, widgets};

use ff_core::map::{Map, MapBackgroundLayer, MapTileset};
use crate::GuiTheme;

use super::{ButtonParams, EditorAction, EditorContext, Window, WindowParams};

pub struct ImportWindow {
    params: WindowParams,
    map_index: usize,
    tilesets: Vec<MapTileset>,
    selected_tilesets: Vec<usize>,
    should_import_background: bool,
    background_color: Option<Color>,
    background_layers: Vec<MapBackgroundLayer>,
    is_loaded: bool,
}

impl ImportWindow {
    pub fn new(map_index: usize) -> Self {
        let params = WindowParams {
            title: Some("Import".to_string()),
            size: vec2(350.0, 350.0),
            ..Default::default()
        };

        ImportWindow {
            params,
            map_index,
            tilesets: Vec::new(),
            selected_tilesets: Vec::new(),
            should_import_background: false,
            background_color: None,
            background_layers: Vec::new(),
            is_loaded: false,
        }
    }
}

impl Window for ImportWindow {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn draw(
        &mut self,
        ui: &mut Ui,
        size: Vec2,
        _map: &Map,
        _ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let id = hash!("import_window");

        if !self.is_loaded {
            let map_resource = get_map(self.map_index);
            self.tilesets = map_resource.map.tilesets.values().cloned().collect();

            self.background_color = Some(map_resource.map.background_color);
            self.background_layers = map_resource.map.background_layers.clone();

            self.is_loaded = true;
        }

        widgets::Group::new(hash!(id, "list_box"), vec2(size.x, size.y * 0.8))
            .position(vec2(0.0, 0.0))
            .ui(ui, |ui| {
                {
                    let gui_theme = get_gui_theme();
                    ui.push_skin(&gui_theme.list_box_no_bg);
                }

                let entry_size = vec2(size.x, LIST_BOX_ENTRY_HEIGHT);

                for (i, tileset) in self.tilesets.iter().enumerate() {
                    let is_selected = self.selected_tilesets.contains(&i);

                    if is_selected {
                        let gui_theme = get_gui_theme();
                        ui.push_skin(&gui_theme.list_box_selected);
                    }

                    let entry_position = vec2(0.0, i as f32 * entry_size.y);

                    let entry_btn = widgets::Button::new("")
                        .size(entry_size)
                        .position(entry_position);

                    if entry_btn.ui(ui) {
                        if is_selected {
                            self.selected_tilesets.retain(|selected| *selected != i);
                        } else {
                            self.selected_tilesets.push(i);
                        }
                    }

                    ui.label(entry_position, &tileset.id);

                    if is_selected {
                        ui.pop_skin();
                    }
                }

                ui.pop_skin();
            });

        {
            let position = vec2(0.0, (size.y * 0.8) + ELEMENT_MARGIN);

            let checkbox = Checkbox::new(
                hash!(id, "background_checkbox"),
                position,
                "Import Background",
            );

            checkbox
                .with_margin(ELEMENT_MARGIN)
                .ui(ui, &mut self.should_import_background);
        }

        None
    }

    fn get_buttons(&self, _map: &Map, _ctx: &EditorContext) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let tilesets = self
            .tilesets
            .iter()
            .enumerate()
            .filter_map(|(i, tileset)| {
                if self.selected_tilesets.contains(&i) {
                    Some(tileset.clone())
                } else {
                    None
                }
            })
            .collect();

        let mut background_color = None;
        let mut background_layers = Vec::new();

        if self.should_import_background {
            background_color = self.background_color;
            background_layers = self.background_layers.clone();
        }

        let batch = self.get_close_action().then(EditorAction::Import {
            tilesets,
            background_color,
            background_layers,
        });

        res.push(ButtonParams {
            label: "Import",
            action: Some(batch),
            ..Default::default()
        });

        res.push(ButtonParams {
            label: "Cancel",
            action: Some(self.get_close_action()),
            ..Default::default()
        });

        res
    }
}
