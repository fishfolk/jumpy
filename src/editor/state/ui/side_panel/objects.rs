use egui_extras::TableBuilder;
use macroquad::prelude::collections::storage;

use crate::{
    editor::{
        actions::{UiAction, UiActionExt},
        state::{EditorTool, SelectableEntity, SelectableEntityKind},
        util::EguiTextureHandler,
    },
    map::MapLayer,
    resources::TextureResource,
    Resources,
};

use super::{Editor, TABLE_ROW_HEIGHT};

impl Editor {
    pub(super) fn draw_object_info(&self, ui: &mut egui::Ui, layer: &MapLayer) {
        println!("Selection: {:?}", self.selection);

        egui_extras::TableBuilder::new(ui)
            .column(egui_extras::Size::remainder())
            .striped(true)
            .header(20.0, |mut row| {
                row.col(|ui| {
                    ui.heading("Objects");
                });
            })
            .body(|body| {
                body.rows(
                    TABLE_ROW_HEIGHT,
                    layer.objects.len(),
                    |row_index, mut row| {
                        let object = &layer.objects[row_index];
                        row.col(|ui| {
                            if matches!(
                                self.selection.as_ref(),
                                Some(SelectableEntity {
                                    kind: SelectableEntityKind::Object {
                                        layer_id,
                                        index
                                    },
                                    ..
                                }) if layer_id == &layer.id && index == &row_index
                            ) {
                                ui.label(format!("[selected] {}", &object.id));
                            } else {
                                ui.label(&object.id);
                            }
                        });
                    },
                )
            })
    }
}
