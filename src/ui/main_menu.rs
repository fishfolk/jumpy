use std::marker::PhantomData;

use bevy::{
    app::AppExit,
    ecs::system::{SystemParam, SystemState},
};
use bevy_egui::*;
use bevy_fluent::Localization;
use iyes_loopless::condition::IntoConditionalExclusiveSystem;

use crate::{
    input::MenuAction,
    localization::LocalizationExt,
    metadata::{GameMeta, Settings},
    platform::Storage,
    player::PlayerIdx,
    prelude::*,
    utils::ResetController,
};

// use self::settings::ControlInputBindingEvents;

use self::settings::{ModifiedSettings, SettingsTab};

use super::{
    widget,
    widgets::{
        bordered_button::BorderedButton, bordered_frame::BorderedFrame, EguiContextExt,
        EguiResponseExt, EguiUiExt,
    },
    DisableMenuInput, WidgetAdjacencies, WidgetId, WidgetSystem,
};

mod map_select;
mod player_select;
mod settings;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MainMenuBackground>()
            .init_resource::<MenuPage>()
            .init_resource::<settings::SettingsTab>()
            .init_resource::<settings::ModifiedSettings>()
            .init_resource::<player_select::PlayerSelectState>()
            .add_system(main_menu_system.run_in_state(GameState::MainMenu).at_end())
            .add_enter_system(GameState::MainMenu, setup_main_menu)
            .add_exit_system(GameState::MainMenu, clean_up_main_menu);
    }
}

#[derive(Component, Reflect)]
pub struct MainMenuBackground;

/// Spawns the background image for the main menu
pub fn setup_main_menu(
    mut commands: Commands,
    game: Res<GameMeta>,
    mut reset_controller: ResetController,
) {
    // Reset the game world
    reset_controller.reset_world();

    let bg_handle = game.main_menu.background_image.image_handle.clone();
    let img_size = game.main_menu.background_image.image_size;
    let ratio = img_size.x / img_size.y;
    let height = game.camera_height as f32;
    let width = height * ratio;
    commands
        .spawn()
        .insert(Name::new("Menu Background Parent"))
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
pub fn clean_up_main_menu(
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
    Home,
    Settings,
    PlayerSelect,
    MapSelect,
}

impl Default for MenuPage {
    fn default() -> Self {
        Self::Home
    }
}

impl SettingsTab {
    const TABS: &'static [(Self, &'static str)] = &[
        (Self::Controls, "controls"),
        // For now, hide the sound tab because we don't have it working yet.
        // (Self::Sound, "sound")
    ];
}

/// Render the main menu UI
pub fn main_menu_system(world: &mut World) {
    world.resource_scope(|world: &mut World, mut egui_ctx: Mut<EguiContext>| {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(egui_ctx.ctx_mut(), |ui| {
                widget::<MainMenu>(world, ui, WidgetId::new("main-menu"), ());
            });
    });
}

#[derive(SystemParam)]
struct MainMenu<'w, 's> {
    menu_page: ResMut<'w, MenuPage>,
    disable_menu_input: ResMut<'w, DisableMenuInput>,
    menu_input: Query<'w, 's, &'static mut ActionState<MenuAction>>,
    keyboard_input: Res<'w, Input<KeyCode>>,
}

impl<'w, 's> WidgetSystem for MainMenu<'w, 's> {
    type Args = ();
    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut egui::Ui,
        id: WidgetId,
        _args: (),
    ) {
        let mut params: MainMenu = state.get_mut(world);

        let menu_input = params.menu_input.single();

        // Disable menu input handling on player select page, so each player can control their own
        // player selection independently.
        let is_player_select = matches!(*params.menu_page, MenuPage::PlayerSelect);
        **params.disable_menu_input = is_player_select;

        // Go to previous menu if back button is pressed
        if menu_input.just_pressed(MenuAction::Back) && !is_player_select {
            match *params.menu_page {
                MenuPage::Settings { .. } | MenuPage::PlayerSelect => {
                    *params.menu_page = MenuPage::Home;
                    ui.ctx().clear_focus();
                }
                MenuPage::MapSelect => {
                    *params.menu_page = MenuPage::PlayerSelect;
                    ui.ctx().clear_focus();
                }
                MenuPage::Home => (),
            }
        } else if is_player_select && params.keyboard_input.just_pressed(KeyCode::Escape) {
            *params.menu_page = MenuPage::Home;
            ui.ctx().clear_focus();
        }

        // Render the menu based on the current menu selection
        match *params.menu_page {
            MenuPage::Home => widget::<HomeMenu>(world, ui, id.with("home"), ()),
            MenuPage::PlayerSelect => {
                widget::<player_select::PlayerSelectMenu>(world, ui, id.with("player-select"), ())
            }
            MenuPage::MapSelect => {
                widget::<map_select::MapSelectMenu>(world, ui, id.with("map-select"), ())
            }
            MenuPage::Settings => {
                widget::<settings::SettingsMenu>(world, ui, id.with("settings"), ())
            }
        }
    }
}

#[derive(SystemParam)]
struct HomeMenu<'w, 's> {
    commands: Commands<'w, 's>,
    menu_page: ResMut<'w, MenuPage>,
    player_select_state: ResMut<'w, player_select::PlayerSelectState>,
    modified_settings: ResMut<'w, ModifiedSettings>,
    game: Res<'w, GameMeta>,
    localization: Res<'w, Localization>,
    app_exit: EventWriter<'w, 's, AppExit>,
    storage: ResMut<'w, Storage>,
}

impl<'w, 's> WidgetSystem for HomeMenu<'w, 's> {
    type Args = ();
    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut egui::Ui,
        _: WidgetId,
        _: (),
    ) {
        let mut params: HomeMenu = state.get_mut(world);

        // Reset player selection when comming to the home menu
        if params.player_select_state.is_changed() {
            *params.player_select_state = default();
        }

        let ui_theme = &params.game.ui_theme;

        ui.vertical_centered(|ui| {
            ui.add_space(&params.game.main_menu.title_font.size / 4.0);
            ui.themed_label(
                &params.game.main_menu.title_font,
                &params.localization.get("title"),
            );
            ui.themed_label(
                &params.game.main_menu.subtitle_font,
                &params.localization.get("subtitle"),
            );
        });

        ui.add_space(params.game.main_menu.subtitle_font.size / 2.0);

        // Create a vertical list of items, centered horizontally
        ui.vertical_centered(|ui| {
            let available_size = ui.available_size();

            let menu_width = params.game.main_menu.menu_width;
            let x_margin = (available_size.x - menu_width) / 2.0;
            let outer_margin = egui::style::Margin::symmetric(x_margin, 0.0);

            BorderedFrame::new(&params.game.ui_theme.panel.border)
                .margin(outer_margin)
                .padding(params.game.ui_theme.panel.padding.into())
                .show(ui, |ui| {
                    let min_button_size = egui::vec2(ui.available_width(), 0.0);

                    // Start button
                    let start_button = BorderedButton::themed(
                        &ui_theme.button_styles.normal,
                        &params.localization.get("local-game"),
                    )
                    .min_size(min_button_size)
                    .show(ui)
                    .focus_by_default(ui);

                    if start_button.clicked() {
                        *params.menu_page = MenuPage::PlayerSelect;
                    }

                    // Map editor
                    if BorderedButton::themed(
                        &ui_theme.button_styles.normal,
                        &params.localization.get("editor"),
                    )
                    .min_size(min_button_size)
                    .show(ui)
                    .clicked()
                    {
                        params
                            .commands
                            .insert_resource(NextState(InGameState::Editing));
                        params
                            .commands
                            .insert_resource(NextState(GameState::InGame));
                    }

                    // Settings button
                    if BorderedButton::themed(
                        &ui_theme.button_styles.normal,
                        &params.localization.get("settings"),
                    )
                    .min_size(min_button_size)
                    .show(ui)
                    .clicked()
                    {
                        *params.menu_page = MenuPage::Settings;
                        **params.modified_settings = Some(
                            params
                                .storage
                                .get(Settings::STORAGE_KEY)
                                .unwrap_or_else(|| params.game.default_settings.clone()),
                        );
                    }

                    // Quit button
                    #[cfg(not(target_arch = "wasm32"))]
                    // Quitting doesn't make sense in a web context
                    if BorderedButton::themed(
                        &ui_theme.button_styles.normal,
                        &params.localization.get("quit"),
                    )
                    .min_size(min_button_size)
                    .show(ui)
                    .clicked()
                    {
                        params.app_exit.send(AppExit);
                    }

                    // use the app exit variable on WASM to avoid warnings
                    #[cfg(target_arch = "wasm32")]
                    let _ = params.app_exit;
                });
        });
    }
}
