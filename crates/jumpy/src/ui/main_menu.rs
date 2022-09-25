use bevy::{app::AppExit, ecs::system::SystemParam, prelude::*};
use bevy_egui::*;
use bevy_fluent::Localization;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    input::MenuAction,
    localization::LocalizationExt,
    metadata::{settings::Settings, GameMeta},
    platform::Storage,
};

use self::settings::ControlInputBindingEvents;

use super::{
    widgets::{bordered_button::BorderedButton, bordered_frame::BorderedFrame, EguiUIExt},
    EguiContextExt, EguiResponseExt, WidgetAdjacencies,
};

mod player_select;
mod settings;

#[derive(Component)]
pub struct MainMenuBackground;

/// Spawns the background image for the main menu
pub fn spawn_main_menu_background(mut commands: Commands, game: Res<GameMeta>) {
    let bg_handle = game.main_menu.background_image.image_handle.clone();
    let img_size = game.main_menu.background_image.image_size;
    let ratio = img_size.x / img_size.y;
    let height = game.camera_height as f32;
    let width = height * ratio;
    commands
        .spawn_bundle(SpriteBundle {
            texture: bg_handle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(width, height)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-width, 0.0, 0.0)),
            ..default()
        })
        .insert(Name::new("Main Menu Background Left"))
        .insert(MainMenuBackground);
    commands
        .spawn_bundle(SpriteBundle {
            texture: bg_handle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(width, height)),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Main Menu Background Middle"))
        .insert(MainMenuBackground);
    commands
        .spawn_bundle(SpriteBundle {
            texture: bg_handle,
            sprite: Sprite {
                custom_size: Some(Vec2::new(width, height)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(width, 0.0, 0.0)),
            ..default()
        })
        .insert(Name::new("Main Menu Background Right"))
        .insert(MainMenuBackground);
}

/// Despawns the background image for the main menu
pub fn despawn_main_menu_background(
    mut commands: Commands,
    backgrounds: Query<Entity, With<MainMenuBackground>>,
) {
    for bg in &backgrounds {
        commands.entity(bg).despawn();
    }
}

/// Which page of the menu we are on
#[derive(Clone, Copy)]
pub enum MenuPage {
    Main,
    Settings { tab: SettingsTab },
    PlayerSelect,
}

/// Which settings tab we are on
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Controls,
    #[allow(unused)] // TODO: Just for now until we get sound settings setup
    Sound,
}

impl Default for MenuPage {
    fn default() -> Self {
        Self::Main
    }
}

impl Default for SettingsTab {
    fn default() -> Self {
        Self::Controls
    }
}

impl SettingsTab {
    const TABS: &'static [(Self, &'static str)] = &[
        (Self::Controls, "controls"),
        // For now, hide the sound tab because we don't have it working yet.
        // (Self::Sound, "sound")
    ];
}

/// Group of parameters needed by the main menu system
#[derive(SystemParam)]
pub struct MenuSystemParams<'w, 's> {
    menu_page: Local<'s, MenuPage>,
    modified_settings: Local<'s, Option<Settings>>,
    currently_binding_input_idx: Local<'s, Option<usize>>,
    game: Res<'w, GameMeta>,
    localization: Res<'w, Localization>,
    menu_input: Query<'w, 's, &'static mut ActionState<MenuAction>>,
    app_exit: EventWriter<'w, 's, AppExit>,
    storage: ResMut<'w, Storage>,
    adjacencies: ResMut<'w, WidgetAdjacencies>,
    control_inputs: ControlInputBindingEvents<'w, 's>,
}

/// Render the main menu UI
pub fn main_menu_system(mut params: MenuSystemParams, mut egui_context: ResMut<EguiContext>) {
    let menu_input = params.menu_input.single();

    // Go to previous menu if back button is pressed
    if menu_input.pressed(MenuAction::Back) {
        match *params.menu_page {
            MenuPage::Settings { .. } | MenuPage::PlayerSelect => {
                *params.menu_page = MenuPage::Main;
                egui_context.ctx_mut().clear_focus();
            }
            _ => (),
        }
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(egui_context.ctx_mut(), |ui| {
            // Render the menu based on the current menu selection
            match *params.menu_page {
                MenuPage::Main => main_menu_ui(&mut params, ui),
                MenuPage::PlayerSelect => player_select::player_select_ui(&mut params, ui),
                MenuPage::Settings { tab } => settings::settings_menu_ui(&mut params, ui, tab),
            }
        });
}

/// Render the main menu
fn main_menu_ui(params: &mut MenuSystemParams, ui: &mut egui::Ui) {
    let MenuSystemParams {
        menu_page,
        modified_settings,
        game,
        localization,
        app_exit,
        storage,
        ..
    } = params;

    let ui_theme = &game.ui_theme;

    if matches!(**menu_page, MenuPage::Main) {
        ui.vertical_centered(|ui| {
            ui.add_space(game.main_menu.title_font.size / 4.0);
            ui.themed_label(&game.main_menu.title_font, &localization.get("title"));
            ui.themed_label(&game.main_menu.subtitle_font, &localization.get("subtitle"));
        });

        ui.add_space(game.main_menu.subtitle_font.size / 2.0);
    }

    // Create a vertical list of items, centered horizontally
    ui.vertical_centered(|ui| {
        let available_size = ui.available_size();

        let pause_menu_width = 300.0;
        let x_margin = (available_size.x - pause_menu_width) / 2.0;
        let outer_margin = egui::style::Margin::symmetric(x_margin, 0.0);

        BorderedFrame::new(&game.ui_theme.panel.border)
            .margin(outer_margin)
            .padding(game.ui_theme.panel.padding.into())
            .show(ui, |ui| {
                let min_button_size = egui::vec2(ui.available_width(), 0.0);

                // Start button
                let start_button = BorderedButton::themed(
                    &ui_theme.button_styles.normal,
                    &localization.get("local-game"),
                )
                .min_size(min_button_size)
                .show(ui)
                .focus_by_default(ui);

                if start_button.clicked() {
                    **menu_page = MenuPage::PlayerSelect;
                }

                // Settings button
                if BorderedButton::themed(
                    &ui_theme.button_styles.normal,
                    &localization.get("settings"),
                )
                .min_size(min_button_size)
                .show(ui)
                .clicked()
                {
                    **menu_page = MenuPage::Settings { tab: default() };
                    **modified_settings = Some(
                        storage
                            .get(Settings::STORAGE_KEY)
                            .unwrap_or_else(|| game.default_settings.clone()),
                    );
                }

                // Quit button
                #[cfg(not(target_arch = "wasm32"))] // Quitting doesn't make sense in a web context
                if BorderedButton::themed(&ui_theme.button_styles.normal, &localization.get("quit"))
                    .min_size(min_button_size)
                    .show(ui)
                    .clicked()
                {
                    app_exit.send(AppExit);
                }

                // use the app exit variable on WASM to avoid warnings
                #[cfg(target_arch = "wasm32")]
                let _ = app_exit;
            });
    });
}
