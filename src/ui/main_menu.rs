use crate::prelude::*;

use super::ImageMeta;

mod credits;
mod map_select;
pub mod player_select;
pub(super) mod settings;
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

#[allow(clippy::const_is_empty)]
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
    let mut close_settings_menu = false;

    // Go to player select menu if either of the `TEST_PLAYER` or `TEST_MAP`
    // debug env vars are present.
    #[cfg(debug_assertions)]
    {
        use std::env::var_os;
        use std::sync::atomic::{AtomicBool, Ordering};
        static DEBUG_DID_CHECK_ENV_VARS: AtomicBool = AtomicBool::new(false);
        if DEBUG_DID_CHECK_ENV_VARS
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            let test_vars = [
                var_os("TEST_MAP"),
                var_os("TEST_PLAYER"),
                var_os("TEST_HAT"),
                var_os("TEST_CONTROLLER"),
            ];
            if test_vars.iter().any(Option::is_some) {
                ctx.set_state(MenuPage::PlayerSelect);
            }
        }
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(&ctx, |ui| match ctx.get_state::<MenuPage>() {
            MenuPage::Home => world.run_system(home_menu, ui),
            MenuPage::Settings => {
                world.run_system(settings::widget, (ui, &mut close_settings_menu))
            }
            MenuPage::PlayerSelect => world.run_system(player_select::widget, ui),
            MenuPage::MapSelect { .. } => world.run_system(map_select::widget, ui),
            MenuPage::Credits => world.run_system(credits::widget, ui),
            MenuPage::NetworkGame =>
            {
                #[cfg(not(target_arch = "wasm32"))]
                world.run_system(network_game::widget, ui)
            }
        });

    if close_settings_menu {
        ctx.set_state(MenuPage::Home);
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(&ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Max), |ui| {
                ui.add_space(5.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    ui.add_space(5.0);

                    ui.label(
                        egui::RichText::new(VERSION_STRING.as_str()).color(egui::Color32::WHITE),
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
    #[cfg(not(target_arch = "wasm32"))] exit_game: Option<ResMut<ExitBones>>,
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
                    if let Some(mut exit) = exit_game {
                        **exit = true;
                    }
                }
            });
    });
}

#[cfg(debug_assertions)]
fn handle_names_to_string<'handles, T, It, F>(it: It, get_name: F) -> String
where
    T: 'handles,
    It: IntoIterator<Item = Handle<T>>,
    F: Fn(Handle<T>) -> &'static str,
{
    let mut names = String::new();
    let mut is_first = true;
    for h in it.into_iter() {
        if is_first {
            is_first = false;
        } else {
            names.push_str(", ");
        }
        names.push('"');
        names.push_str(get_name(h));
        names.push('"');
    }
    names
}
