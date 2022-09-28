use bevy::{app::AppExit, ecs::system::SystemParam};
use bevy_egui::*;
use bevy_fluent::Localization;

use crate::{
    input::{MenuAction, PlayerAction},
    localization::LocalizationExt,
    metadata::{GameMeta, PlayerMeta, Settings},
    platform::Storage,
    player::PlayerIdx,
    prelude::*,
};

use self::settings::ControlInputBindingEvents;

use super::{
    widgets::{bordered_button::BorderedButton, bordered_frame::BorderedFrame, EguiUIExt},
    DisableMenuInput, EguiContextExt, EguiResponseExt, WidgetAdjacencies,
};

mod map_select;
mod player_select;
mod settings;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MainMenuBackground>()
            .add_enter_system(GameState::MainMenu, spawn_main_menu_background)
            .add_exit_system(GameState::MainMenu, despawn_main_menu_background);
    }
}

#[derive(Component, Reflect)]
pub struct MainMenuBackground;

/// Spawns the background image for the main menu
pub fn spawn_main_menu_background(mut commands: Commands, game: Res<GameMeta>) {
    let bg_handle = game.main_menu.background_image.image_handle.clone();
    let img_size = game.main_menu.background_image.image_size;
    let ratio = img_size.x / img_size.y;
    let height = game.camera_height as f32;
    let width = height * ratio;
    commands
        .spawn()
        .insert_bundle(VisibilityBundle::default())
        .insert_bundle(TransformBundle::default())
        .insert(MainMenuBackground)
        .with_children(|parent| {
            parent
                .spawn_bundle(SpriteBundle {
                    texture: bg_handle.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(width, height)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(-width, 0.0, 0.0)),
                    ..default()
                })
                .insert(Name::new("Main Menu Background Left"));
            parent
                .spawn_bundle(SpriteBundle {
                    texture: bg_handle.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(width, height)),
                        ..default()
                    },
                    ..default()
                })
                .insert(Name::new("Main Menu Background Middle"));
            parent
                .spawn_bundle(SpriteBundle {
                    texture: bg_handle,
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(width, height)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(width, 0.0, 0.0)),
                    ..default()
                })
                .insert(Name::new("Main Menu Background Right"));
        });
}

/// Despawns the background image for the main menu
pub fn despawn_main_menu_background(
    mut commands: Commands,
    backgrounds: Query<Entity, With<MainMenuBackground>>,
) {
    for bg in &backgrounds {
        commands.entity(bg).despawn_recursive();
    }
}

/// Which page of the menu we are on
#[derive(Clone, Copy)]
pub enum MenuPage {
    Main,
    Settings { tab: SettingsTab },
    PlayerSelect,
    MapSelect,
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
    commands: Commands<'w, 's>,
    menu_page: Local<'s, MenuPage>,
    disable_menu_input: ResMut<'w, DisableMenuInput>,
    player_select_state: Local<'s, player_select::PlayerSelectState>,
    players: Query<'w, 's, (&'static PlayerIdx, &'static ActionState<PlayerAction>)>,
    modified_settings: Local<'s, Option<Settings>>,
    currently_binding_input_idx: Local<'s, Option<usize>>,
    game: Res<'w, GameMeta>,
    localization: Res<'w, Localization>,
    menu_input: Query<'w, 's, &'static mut ActionState<MenuAction>>,
    app_exit: EventWriter<'w, 's, AppExit>,
    storage: ResMut<'w, Storage>,
    adjacencies: ResMut<'w, WidgetAdjacencies>,
    control_inputs: ControlInputBindingEvents<'w, 's>,
    keyboard_input: Res<'w, Input<KeyCode>>,
    player_meta_assets: Res<'w, Assets<PlayerMeta>>,
}

/// Render the main menu UI
pub fn main_menu_system(mut params: MenuSystemParams, mut egui_context: ResMut<EguiContext>) {
    let menu_input = params.menu_input.single();

    // Disable menu input handling on player select page, so each player can control their own
    // player selection independently.
    let is_player_select = matches!(*params.menu_page, MenuPage::PlayerSelect);
    **params.disable_menu_input = is_player_select;

    // Go to previous menu if back button is pressed
    if menu_input.just_pressed(MenuAction::Back) && !is_player_select {
        match *params.menu_page {
            MenuPage::Settings { .. } | MenuPage::PlayerSelect => {
                *params.menu_page = MenuPage::Main;
                egui_context.ctx_mut().clear_focus();
            }
            MenuPage::MapSelect => {
                *params.menu_page = MenuPage::PlayerSelect;
                egui_context.ctx_mut().clear_focus();
            }
            MenuPage::Main => (),
        }
    } else if is_player_select && params.keyboard_input.just_pressed(KeyCode::Escape) {
        *params.menu_page = MenuPage::Main;
        egui_context.ctx_mut().clear_focus();
    }

    // Clear the player selection whenever we go to the main menu
    if matches!(*params.menu_page, MenuPage::Main) {
        *params.player_select_state = default();
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(egui_context.ctx_mut(), |ui| {
            // Render the menu based on the current menu selection
            match *params.menu_page {
                MenuPage::Main => main_menu_ui(&mut params, ui),
                MenuPage::PlayerSelect => player_select::player_select_ui(&mut params, ui),
                MenuPage::MapSelect => map_select::map_select_ui(&mut params, ui),
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
        commands,
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

                // Map editor
                if BorderedButton::themed(
                    &ui_theme.button_styles.normal,
                    &localization.get("editor"),
                )
                .min_size(min_button_size)
                .show(ui)
                .clicked()
                {
                    commands.insert_resource(NextState(InGameState::Editing));
                    commands.insert_resource(NextState(GameState::InGame));
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
