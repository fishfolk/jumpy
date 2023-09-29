use crate::prelude::*;

pub fn session_plugin(session: &mut Session) {
    session.add_system_to_stage(Update, pause_menu_system);
}

fn pause_menu_system(mut sessions: ResMut<Sessions>, ctx: Egui, controls: Res<PlayerControls>) {
    if let Some(session) = sessions.get_mut(SessionNames::GAME) {
        if !session.active {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(&ctx, |ui| {
                    ui.heading("Paused");
                });

            if controls.iter().any(|x| x.pause_just_pressed) {
                session.active = true;
            }
        }
    }
}
