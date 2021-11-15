use macroquad::{experimental::collections::storage, prelude::*, ui::Ui};

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
        _size: Vec2,
        _map: &Map,
        _ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let resources = storage::get::<Resources>();

        for map_resource in &resources.maps {
            ui.label(None, &map_resource.meta.path);
        }

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
