use std::path::Path;

use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{hash, widgets, Ui},
};

use core::text::ToStringHelper;

use super::{ButtonParams, EditorAction, EditorContext, Window, WindowParams};

use crate::map::Map;
use crate::resources::{map_name_to_filename, MAP_EXPORTS_DEFAULT_DIR, MAP_EXPORTS_EXTENSION};
use crate::Resources;

pub struct CreateMapWindow {
    params: WindowParams,
    name: String,
    description: String,
    grid_size: UVec2,
    tile_size: Vec2,
    map_export_path: String,
}

impl CreateMapWindow {
    pub fn new() -> Self {
        let params = WindowParams {
            title: Some("Create Map".to_string()),
            size: vec2(350.0, 425.0),
            ..Default::default()
        };

        let map_export_path = {
            let resources = storage::get::<Resources>();
            Path::new(&resources.assets_dir).join(MAP_EXPORTS_DEFAULT_DIR)
        };

        CreateMapWindow {
            params,
            name: "unnamed_map".to_string(),
            description: "".to_string(),
            grid_size: uvec2(100, 75),
            tile_size: vec2(16.0, 16.0),
            map_export_path: map_export_path.to_string_helper(),
        }
    }
}

impl Window for CreateMapWindow {
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
        let id = hash!("create_map_window");

        ui.separator();

        {
            let size = vec2(275.0, 25.0);

            widgets::InputText::new(hash!(id, "name_input"))
                .size(size)
                .ratio(1.0)
                .ui(ui, &mut self.name);
        }

        ui.separator();

        {
            let path_label = Path::new(&self.map_export_path)
                .join(map_name_to_filename(&self.name))
                .with_extension(MAP_EXPORTS_EXTENSION);

            widgets::Label::new(path_label.to_string_lossy().as_ref()).ui(ui);
        }

        ui.separator();

        {
            let size = vec2(275.0, 75.0);

            widgets::InputText::new(hash!(id, "description_input"))
                .size(size)
                .ratio(1.0)
                .ui(ui, &mut self.description);
        }

        ui.separator();

        {
            let mut grid_width = self.grid_size.x.to_string();
            let mut grid_height = self.grid_size.y.to_string();

            let mut tile_width = self.tile_size.x.to_string();
            let mut tile_height = self.tile_size.y.to_string();

            let size = vec2(75.0, 25.0);

            widgets::InputText::new(hash!(id, "tile_width_input"))
                .size(size)
                .ratio(1.0)
                .label("x")
                .ui(ui, &mut tile_width);

            ui.same_line(size.x + 25.0);

            widgets::InputText::new(hash!(id, "tile_height_input"))
                .size(size)
                .ratio(1.0)
                .label("Tile size")
                .ui(ui, &mut tile_height);

            widgets::InputText::new(hash!(id, "grid_width_input"))
                .size(size)
                .ratio(1.0)
                .label("x")
                .ui(ui, &mut grid_width);

            ui.same_line(size.x + 25.0);

            widgets::InputText::new(hash!(id, "grid_height_input"))
                .size(size)
                .ratio(1.0)
                .label("Grid size")
                .ui(ui, &mut grid_height);

            self.grid_size = uvec2(
                grid_width.parse::<u32>().unwrap(),
                grid_height.parse::<u32>().unwrap(),
            );

            self.tile_size = vec2(
                tile_width.parse::<f32>().unwrap(),
                tile_height.parse::<f32>().unwrap(),
            );
        }

        ui.separator();

        None
    }

    fn get_buttons(&self, _map: &Map, _ctx: &EditorContext) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let mut action = None;

        if self.grid_size > UVec2::ZERO && self.tile_size > Vec2::ZERO {
            let mut description = None;
            if !self.description.is_empty() {
                description = Some(self.description.clone());
            }

            let batch = self.get_close_action().then(EditorAction::CreateMap {
                name: self.name.clone(),
                description,
                tile_size: self.tile_size,
                grid_size: self.grid_size,
            });

            action = Some(batch);
        }

        res.push(ButtonParams {
            label: "Create",
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

impl Default for CreateMapWindow {
    fn default() -> Self {
        Self::new()
    }
}
