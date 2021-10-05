use macroquad::{prelude::*, ui::Ui};

use super::{EditorAction, EditorDrawParams, Map, Window, WindowParams};
use crate::editor::gui::windows::ButtonParams;

pub struct ConfirmDialog {
    params: WindowParams,
    body: Vec<String>,
    confirm_action: EditorAction,
}

impl ConfirmDialog {
    const WINDOW_TITLE: &'static str = "Please Confirm";
    const CONFIRM_LABEL: &'static str = "Ok";
    const CANCEL_LABEL: &'static str = "Cancel";

    pub fn new(size: Vec2, body: &[&str], confirm_action: EditorAction) -> Self {
        let params = WindowParams {
            title: Some(Self::WINDOW_TITLE.to_string()),
            size,
            is_static: true,
            ..Default::default()
        };

        let body = body.iter().map(|line| line.to_string()).collect();

        ConfirmDialog {
            params,
            body,
            confirm_action,
        }
    }
}

impl Window for ConfirmDialog {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn get_buttons(&self, _map: &Map, _draw_params: &EditorDrawParams) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let action = self.get_close_action().then(self.confirm_action.clone());

        res.push(ButtonParams {
            label: Self::CONFIRM_LABEL,
            action: Some(action),
            ..Default::default()
        });

        res.push(ButtonParams {
            label: Self::CANCEL_LABEL,
            action: Some(self.get_close_action()),
            ..Default::default()
        });

        res
    }

    fn draw(
        &mut self,
        ui: &mut Ui,
        _size: Vec2,
        _map: &Map,
        _draw_params: &EditorDrawParams,
    ) -> Option<EditorAction> {
        for line in &self.body {
            ui.label(None, line);
        }

        ui.separator();
        ui.separator();
        ui.separator();
        ui.separator();

        None
    }
}
