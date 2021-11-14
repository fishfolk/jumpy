use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{widgets, Ui},
};

use super::{
    EditorAction, EditorContext, GuiResources, Map, Toolbar, ToolbarElement, ToolbarElementParams,
};

use crate::{
    editor::{gui::ButtonParams, EditorCamera},
    map::MapLayerKind,
};

pub struct ObjectListElement {
    params: ToolbarElementParams,
}

impl ObjectListElement {
    pub fn new() -> Self {
        let params = ToolbarElementParams {
            header: Some("Objects".to_string()),
            has_buttons: true,
            has_margins: false,
        };

        ObjectListElement { params }
    }
}

impl ToolbarElement for ObjectListElement {
    fn get_params(&self) -> &ToolbarElementParams {
        &self.params
    }

    fn draw(
        &mut self,
        ui: &mut Ui,
        size: Vec2,
        map: &Map,
        ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let mut res = None;

        let entry_size = vec2(size.x, Toolbar::LIST_ENTRY_HEIGHT);
        let mut position = Vec2::ZERO;

        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.editor_skins.menu);
        }

        let layer_id = ctx.selected_layer.as_ref().unwrap();
        let layer = map.layers.get(layer_id).unwrap();

        for (i, object) in layer.objects.iter().enumerate() {
            let is_selected = if let Some(selected_index) = ctx.selected_object {
                selected_index == i
            } else {
                false
            };

            if is_selected {
                let gui_resources = storage::get::<GuiResources>();
                ui.push_skin(&gui_resources.editor_skins.menu_selected);
            }

            let was_clicked = widgets::Button::new("")
                .size(entry_size)
                .position(position)
                .ui(ui);

            ui.label(position, &object.id);

            if was_clicked {
                res = Some(EditorAction::SelectObject {
                    index: i,
                    layer_id: layer_id.clone(),
                });
            }

            if is_selected {
                ui.pop_skin();
            }

            position.y += entry_size.y;
        }

        ui.pop_skin();

        res
    }

    fn get_buttons(&self, map: &Map, ctx: &EditorContext) -> Vec<ButtonParams> {
        let layer_id = ctx.selected_layer.clone().unwrap();

        let position = {
            let camera = scene::find_node_by_type::<EditorCamera>().unwrap();
            let view_rect = camera.get_view_rect();
            let offset = vec2(view_rect.w, view_rect.h) / 2.0;
            (view_rect.point() + offset) - map.world_offset
        };

        let create_action = Some(EditorAction::OpenCreateObjectWindow {
            layer_id: layer_id.clone(),
            position,
        });

        let mut delete_action = None;
        let mut properties_action = None;

        if let Some(index) = ctx.selected_object {
            delete_action = Some(EditorAction::DeleteObject {
                index,
                layer_id: layer_id.clone(),
            });

            properties_action = Some(EditorAction::OpenObjectPropertiesWindow { layer_id, index });
        }

        vec![
            ButtonParams {
                label: "+",
                width_override: Some(0.25),
                action: create_action,
            },
            ButtonParams {
                label: "-",
                width_override: Some(0.25),
                action: delete_action,
            },
            ButtonParams {
                label: "Properties",
                width_override: Some(0.5),
                action: properties_action,
            },
        ]
    }

    fn is_drawn(&self, map: &Map, ctx: &EditorContext) -> bool {
        if let Some(layer_id) = &ctx.selected_layer {
            if let Some(layer) = map.layers.get(layer_id) {
                return layer.kind == MapLayerKind::ObjectLayer;
            }
        }

        false
    }
}

impl Default for ObjectListElement {
    fn default() -> Self {
        Self::new()
    }
}
