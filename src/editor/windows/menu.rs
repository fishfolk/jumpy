use std::ops::ControlFlow;

pub struct MenuWindow {
    can_save: bool,
}

pub enum MenuResult {
    OpenCreateMapWindow,
    OpenLoadMapWindow,
    SaveMap,
    OpenSaveMapWindow,
    ExitToMainMenu,
    QuitToDesktop,
}

impl MenuWindow {
    pub fn new(can_save: bool) -> Self {
        Self { can_save }
    }

    pub fn ui(&mut self, egui_ctx: &egui::Context) -> ControlFlow<MenuResult> {
        let mut action = ControlFlow::Continue(());

        egui::Window::new("Pause")
            .title_bar(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(egui_ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
                    ui.spacing_mut().item_spacing.y = ui.spacing().window_margin.sum().y;
                    if ui.button("New").clicked() {
                        action = ControlFlow::Break(MenuResult::OpenCreateMapWindow);
                    }
                    if ui.button("Open/Import").clicked() {
                        action = ControlFlow::Break(MenuResult::OpenLoadMapWindow);
                    }
                    ui.add_enabled_ui(self.can_save, |ui| {
                        if ui.button("Save").clicked() {
                            action = ControlFlow::Break(MenuResult::SaveMap);
                        }
                    });
                    if ui.button("Save As").clicked() {
                        action = ControlFlow::Break(MenuResult::OpenSaveMapWindow);
                    }
                    if ui.button("Exit To Main Menu").clicked() {
                        action = ControlFlow::Break(MenuResult::ExitToMainMenu);
                    }
                    if ui.button("Exit To Desktop").clicked() {
                        action = ControlFlow::Break(MenuResult::QuitToDesktop);
                    }
                });
            });

        action
    }
}