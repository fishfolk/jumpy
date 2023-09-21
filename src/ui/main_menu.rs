use crate::prelude::*;

use super::ImageMeta;

#[derive(HasSchema, Debug, Default, Clone)]
#[repr(C)]
pub struct MainMenuMeta {
    title_font: FontMeta,
    subtitle_font: FontMeta,
    background_image: ImageMeta,
    menu_width: f32,
}

pub fn session_plugin(session: &mut Session) {
    session
        // Install the default bones_framework plugin for this session
        .install_plugin(DefaultSessionPlugin)
        .add_startup_system(setup_menu)
        // Add our menu system to the update stage
        .add_system_to_stage(Update, menu_system);
}

fn setup_menu(meta: Root<GameMeta>, mut egui_settings: ResMutInit<EguiSettings>) {
    egui_settings.scale = meta.theme.scale;
}

/// System to render the home menu.
fn menu_system(
    meta: Root<GameMeta>,
    assets: Res<AssetServer>,
    ctx: Egui,
    mut sessions: ResMutInit<Sessions>,
    mut session_options: ResMutInit<SessionOptions>,
    localization: Localization<GameMeta>,
) {
    egui::CentralPanel::default().show(&ctx, |ui| {
        if BorderedButton::themed(&meta.theme.buttons.normal, localization.get("local-game"))
            .show(ui)
            .clicked()
        {
            session_options.delete = true;

            let session = sessions.create("game");
            session.install_plugin(crate::core::MatchPlugin {
                map: assets.get(meta.core.stable_maps[3]).clone(),
                selected_players: [
                    Some(meta.core.players[0]),
                    Some(meta.core.players[1]),
                    None,
                    None,
                ],
            });
        }
    });
}
