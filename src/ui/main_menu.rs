use crate::prelude::*;

use super::ImageMeta;

mod credits;
mod map_select;
pub mod player_select;
mod settings;
use shadow_rs::shadow;

// Generate build info.
shadow!(build_info);

#[cfg(not(target_arch = "wasm32"))]
mod network_game;

#[derive(HasSchema, Debug, Default, Clone)]
#[repr(C)]
pub struct MainMenuMeta {
    pub title_font: FontMeta,
    pub subtitle_font: FontMeta,
    pub background_image: ImageMeta,
    pub menu_width: f32,
}

pub fn session_plugin(session: &mut Session) {
    session
        // Install the default bones_framework plugin for this session
        .install_plugin(DefaultSessionPlugin)
        .add_startup_system(setup_menu)
        // Add our menu system to the update stage
        .add_system_to_stage(Update, main_menu_system);
}

fn setup_menu(
    meta: Root<GameMeta>,
    mut egui_settings: ResMutInit<EguiSettings>,
    mut entities: ResMut<Entities>,
    mut sprites: CompMut<Sprite>,
    mut transforms: CompMut<Transform>,
    mut cameras: CompMut<Camera>,
    mut clear_color: ResMutInit<ClearColor>,
) {
    egui_settings.scale = meta.theme.scale;
    **clear_color = Color::BLACK;
    spawn_default_camera(&mut entities, &mut transforms, &mut cameras);

    for i in -1..=1 {
        let ent = entities.create();
        transforms.insert(
            ent,
            Transform::from_translation(vec3(
                meta.main_menu.background_image.image_size.x * i as f32,
                0.,
                0.,
            )),
        );
        sprites.insert(
            ent,
            Sprite {
                image: meta.main_menu.background_image.image,
                ..default()
            },
        );
    }
}

/// Which page of the menu we are on
#[derive(HasSchema, Clone, Copy, Default)]
#[repr(C, u8)]
pub enum MenuPage {
    #[default]
    Home,
    Settings,
    PlayerSelect,
    MapSelect {
        /// Indicates the client is waiting for the map to be selected, not actually picking the
        /// map.
        is_waiting: bool,
    },
    Credits,
    NetworkGame,
}

static VERSION_STRING: Lazy<String> = Lazy::new(|| {
    format!(
        "{}{}",
        build_info::PKG_VERSION,
        if !build_info::SHORT_COMMIT.is_empty() {
            format!(
                "-{}{}",
                build_info::SHORT_COMMIT,
                if build_info::GIT_CLEAN {
                    ""
                } else {
                    " (dirty)"
                }
            )
        } else {
            String::default()
        }
    )
});

fn main_menu_system(world: &World) {
    let ctx = (*world.resource::<EguiCtx>()).clone();

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(&ctx, |ui| match ctx.get_state::<MenuPage>() {
            MenuPage::Home => world.run_system(home_menu, ui),
            MenuPage::Settings => world.run_system(settings::widget, ui),
            MenuPage::PlayerSelect => world.run_system(player_select::widget, ui),
            MenuPage::MapSelect { .. } => world.run_system(map_select::widget, ui),
            MenuPage::Credits => world.run_system(credits::widget, ui),
            MenuPage::NetworkGame =>
            {
                #[cfg(not(target_arch = "wasm32"))]
                world.run_system(network_game::widget, ui)
            }
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(&ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Max), |ui| {
                ui.add_space(5.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    ui.add_space(5.0);

                    ui.add(
                        egui::TextEdit::singleline(&mut VERSION_STRING.as_str())
                            .text_color(egui::Color32::WHITE)
                            .horizontal_align(egui::Align::Max),
                    );
                })
            });
        });
}

/// System to render the home menu.
fn home_menu(
    mut ui: In<&mut egui::Ui>,
    meta: Root<GameMeta>,
    localization: Localization<GameMeta>,
) {
    let ui = &mut *ui;
    ui.vertical_centered(|ui| {
        ui.add_space(meta.main_menu.title_font.size / 2.0);
        ui.label(meta.main_menu.title_font.rich(localization.get("title")));
        ui.label(
            meta.main_menu
                .subtitle_font
                .rich(localization.get("subtitle")),
        );

        ui.add_space(meta.main_menu.subtitle_font.size / 2.0);

        BorderedFrame::new(&meta.theme.panel.border)
            .padding(meta.theme.panel.padding)
            .show(ui, |ui| {
                ui.set_width(meta.main_menu.menu_width);

                // Local game
                if BorderedButton::themed(
                    &meta.theme.buttons.normal,
                    localization.get("local-game"),
                )
                .min_size(vec2(ui.available_width(), 0.0))
                .show(ui)
                .focus_by_default(ui)
                .clicked()
                {
                    ui.ctx().set_state(MenuPage::PlayerSelect);
                }

                // Online game
                #[cfg(not(target_arch = "wasm32"))]
                if BorderedButton::themed(
                    &meta.theme.buttons.normal,
                    localization.get("online-game"),
                )
                .min_size(vec2(ui.available_width(), 0.0))
                .show(ui)
                .clicked()
                {
                    ui.ctx().set_state(MenuPage::NetworkGame);
                }

                // Settings
                if BorderedButton::themed(&meta.theme.buttons.normal, localization.get("settings"))
                    .min_size(vec2(ui.available_width(), 0.0))
                    .show(ui)
                    .clicked()
                {
                    ui.ctx().set_state(MenuPage::Settings);
                }

                // Credits
                if BorderedButton::themed(&meta.theme.buttons.normal, localization.get("credits"))
                    .min_size(vec2(ui.available_width(), 0.0))
                    .show(ui)
                    .clicked()
                {
                    ui.ctx().set_state(MenuPage::Credits);
                }

                #[cfg(not(target_arch = "wasm32"))]
                if BorderedButton::themed(&meta.theme.buttons.normal, localization.get("quit"))
                    .min_size(vec2(ui.available_width(), 0.0))
                    .show(ui)
                    .clicked()
                {
                    // TODO: Gracefully exit game on quit.
                    // Right now we don't have a way for bones to trigger a Bevy graceful shutdown.
                    // We need to have a way for bones games to communicate that they want to exit,
                    // and then the Bones Bevy Renderer can gracefully shutdown.
                    std::process::exit(0);
                }
            });
    });
}
