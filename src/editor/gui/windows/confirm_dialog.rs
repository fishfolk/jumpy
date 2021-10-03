use macroquad::{
    ui::{
        Ui,
    },
    prelude::*,
};

use super::{
    Map,
    EditorAction,
    EditorDrawParams,
    Window,
    WindowParams,
    WindowResult,
};

pub struct ConfirmDialog {
    params: WindowParams,
    body: Vec<String>,
    confirm_action: EditorAction,
}

impl ConfirmDialog {
    const WINDOW_TITLE: &'static str = "Please Confirm";
    const CONFIRM_LABEL: &'static str = "Ok";
    const CANCEL_LABEL: &'static str = "Cancel";

    pub fn new(size: Vec2, body: &[&str], confirm_action: EditorAction) -> Box<Self> {
        let params = WindowParams {
            title: Some(Self::WINDOW_TITLE.to_string()),
            size,
            is_static: true,
            ..Default::default()
        };

        let body = body
            .into_iter()
            .map(|line| line.to_string())
            .collect();

        Box::new(ConfirmDialog {
            params,
            body,
            confirm_action,
        })
    }
}

impl Window for ConfirmDialog {
    fn get_params(&self) -> &WindowParams {
        &self.params
    }

    fn draw(&mut self, ui: &mut Ui, _size: Vec2, _map: &Map, _draw_params: &EditorDrawParams) -> Option<WindowResult> {
        for line in &self.body {
            ui.label(None, line);
        }

        ui.separator();
        ui.separator();
        ui.separator();
        ui.separator();

        if ui.button(None, Self::CONFIRM_LABEL) {
            return Some(WindowResult::Action(self.confirm_action.clone()));
        }

        if ui.button(None, Self::CANCEL_LABEL) {
            return Some(WindowResult::Cancel);
        }

        None
    }
}

