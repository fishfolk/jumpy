use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{hash, widgets, Ui},
};

use crate::gui::{GuiResources, LIST_BOX_ENTRY_HEIGHT};

use crate::map::{Map, MapTileset};

use super::{ButtonParams, EditorAction, EditorContext, Window, WindowParams};
use crate::Resources;

pub struct ImportTilesetsWindow {
    params: WindowParams,
    map_index: usize,
    tilesets: Vec<MapTileset>,
    selected: Vec<usize>,
    is_loaded: bool,
}

impl ImportTilesetsWindow {
    pub fn new(map_index: usize) -> Self {
        let params = WindowParams {
            title: Some("Import Tilesets".to_string()),
            size: vec2(350.0, 350.0),
            ..Default::default()
        };

        ImportTilesetsWindow {
            params,
            map_index,
            tilesets: Vec::new(),
            selected: Vec::new(),
            is_loaded: false,
        }
    }
}

impl Window for ImportTilesetsWindow {
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
        let id = hash!("import_tilesets_window");

        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.skins.list_box_no_bg);
        }

        if !self.is_loaded {
            let resources = storage::get::<Resources>();
            let map_resource = resources.maps.get(self.map_index).unwrap();
            self.tilesets = map_resource.map.tilesets.values().cloned().collect();
            self.is_loaded = true;
        }

        widgets::Group::new(hash!(id, "list_box"), size)
            .position(vec2(0.0, 0.0))
            .ui(ui, |ui| {
                let entry_size = vec2(size.x, LIST_BOX_ENTRY_HEIGHT);

                for (i, tileset) in self.tilesets.iter().enumerate() {
                    let is_selected = self.selected.contains(&i);

                    if is_selected {
                        let gui_resources = storage::get::<GuiResources>();
                        ui.push_skin(&gui_resources.skins.list_box_selected);
                    }

                    let entry_position = vec2(0.0, i as f32 * entry_size.y);

                    let entry_btn = widgets::Button::new("")
                        .size(entry_size)
                        .position(entry_position);

                    if entry_btn.ui(ui) {
                        if is_selected {
                            self.selected.retain(|selected| *selected != i);
                        } else {
                            self.selected.push(i);
                        }
                    }

                    ui.label(entry_position, &tileset.id);

                    if is_selected {
                        ui.pop_skin();
                    }
                }
            });

        ui.pop_skin();

        None
    }

    fn get_buttons(&self, _map: &Map, _ctx: &EditorContext) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let mut action = None;
        if !self.selected.is_empty() {
            let tilesets = self
                .tilesets
                .iter()
                .enumerate()
                .filter_map(|(i, tileset)| {
                    if self.selected.contains(&i) {
                        Some(tileset.clone())
                    } else {
                        None
                    }
                })
                .collect();

            let batch = self
                .get_close_action()
                .then(EditorAction::ImportTilesets(tilesets));

            action = Some(batch);
        }

        res.push(ButtonParams {
            label: "Import",
            action,
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
