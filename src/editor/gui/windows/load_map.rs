use macroquad::{
    experimental::collections::storage,
    ui::{
        Ui,
        hash,
        widgets,
    },
    prelude::*,
};

use crate::gui::{GuiResources, LIST_BOX_ENTRY_HEIGHT};

use crate::map::Map;

use super::{ButtonParams, EditorAction, EditorContext, Window, WindowParams};
use crate::Resources;

pub struct LoadMapWindow {
    params: WindowParams,
    index: Option<usize>,
}

impl LoadMapWindow {
    pub fn new() -> Self {
        let params = WindowParams {
            title: Some("Open".to_string()),
            size: vec2(350.0, 350.0),
            ..Default::default()
        };

        LoadMapWindow {
            params,
            index: None,
        }
    }
}

impl Window for LoadMapWindow {
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
        let id = hash!("load_map_window");

        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.skins.list_box_no_bg);
        }

        widgets::Group::new(hash!(id, "list_box"), size)
            .position(vec2(0.0, 0.0))
            .ui(ui, |ui| {
                let resources = storage::get::<Resources>();

                let entry_size = vec2(size.x, LIST_BOX_ENTRY_HEIGHT);

                for (i, map_resource) in resources.maps.iter().enumerate() {
                    let mut is_selected = false;
                    if let Some(index) = self.index {
                        is_selected = index == i;
                    }

                    if is_selected {
                        let gui_resources = storage::get::<GuiResources>();
                        ui.push_skin(&gui_resources.skins.list_box_selected);
                    }

                    let entry_position = vec2(0.0, i as f32 * entry_size.y);

                    let entry_btn = widgets::Button::new("")
                        .size(entry_size)
                        .position(entry_position);

                    if entry_btn.ui(ui) {
                        self.index = Some(i);
                    }

                    ui.label(entry_position, &map_resource.meta.path);

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
        if let Some(index) = self.index {
            let batch = self.get_close_action().then(EditorAction::LoadMap(index));

            action = Some(batch);
        }

        res.push(ButtonParams {
            label: "Open",
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

impl Default for LoadMapWindow {
    fn default() -> Self {
        Self::new()
    }
}
