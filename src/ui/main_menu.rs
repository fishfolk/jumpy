use std::marker::PhantomData;

use bevy::{
    app::AppExit,
    ecs::system::{SystemParam, SystemState},
};
use bevy_egui::*;
use bevy_fluent::Localization;

use crate::{
    localization::LocalizationExt,
    metadata::{GameMeta, Settings},
    platform::Storage,
    prelude::*,
    ui::input::MenuAction,
};

use self::settings::{ModifiedSettings, SettingsTab};

use super::{
    widget,
    widgets::{
        bordered_button::BorderedButton, bordered_frame::BorderedFrame, EguiContextExt,
        EguiResponseExt, EguiUiExt,
    },
    DisableMenuInput, WidgetAdjacencies, WidgetId, WidgetSystem,
};

pub mod map_select;
// #[cfg(not(target_arch = "wasm32"))]
// pub mod matchmaking;
pub mod player_select;
pub mod settings;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MainMenuBackground>()
            .init_resource::<MenuPage>()
            .init_resource::<settings::SettingsTab>()
            .init_resource::<settings::ModifiedSettings>()
            .init_resource::<player_select::PlayerSelectState>()
            .add_system(
                main_menu_system
                    .run_in_state(EngineState::MainMenu)
                    .at_end(),
            )
            .add_enter_system(EngineState::MainMenu, setup_main_menu)
            .add_exit_system(EngineState::MainMenu, clean_up_main_menu);
    }
}

#[derive(Component, Reflect)]
pub struct MainMenuBackground;

/// Spawns the background image for the main menu
#[allow(unreachable_code)]
pub fn setup_main_menu(
    mut commands: Commands,
    game: Res<GameMeta>,
    core: Res<CoreMetaArc>,
    mut session_manager: SessionManager,
) {
    session_manager.stop();

    // Spawn menu background
    let bg_handle = game.main_menu.background_image.image.inner.clone_weak();
    let img_size = game.main_menu.background_image.image_size;
    let ratio = img_size.x / img_size.y;
    let height = core.camera_height;
    let width = height * ratio;
    commands
        .spawn((
            Name::new("Menu Background Parent"),
            VisibilityBundle::default(),
            TransformBundle::default(),
            MainMenuBackground,
        ))
        .with_children(|parent| {
            parent.spawn((
                SpriteBundle {
                    texture: bg_handle.clone_weak(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(width, height)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(-width, 0.0, 0.0)),
                    ..default()
                },
                Name::new("Main Menu Background Left"),
            ));
            parent.spawn((
                SpriteBundle {
                    texture: bg_handle.clone_weak(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(width, height)),
                        ..default()
                    },
                    ..default()
                },
                Name::new("Main Menu Background Middle"),
            ));
            parent.spawn((
                SpriteBundle {
                    texture: bg_handle,
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(width, height)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(width, 0.0, 0.0)),
                    ..default()
                },
                Name::new("Main Menu Background Right"),
            ));
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
#[derive(Resource, Clone, Copy)]
pub enum MenuPage {
    Home,
    Settings,
    PlayerSelect,
    MapSelect {
        /// Indicates the client is waiting for the map to be selected, not actually picking the
        /// map.
        is_waiting: bool,
    },
    // Matchmaking,
}

impl Default for MenuPage {
    fn default() -> Self {
        Self::Home
    }
}

impl SettingsTab {
    const TABS: &'static [(Self, &'static str)] = &[
        (Self::Controls, "controls"),
        (Self::Networking, "networking"), // For now, hide the sound tab because we don't have it working yet.
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
    #[system_param(ignore)]
    _phantom: PhantomData<&'s ()>,
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

        // Disable menu input handling on player select page, so each player can control their own
        // player selection independently.
        let is_player_select = matches!(*params.menu_page, MenuPage::PlayerSelect);
        **params.disable_menu_input = is_player_select;

        // Render the menu based on the current menu selection
        match *params.menu_page {
            MenuPage::Home => widget::<HomeMenu>(world, ui, id.with("home"), ()),
            // MenuPage::Matchmaking =>
            // {
            //     #[cfg(not(target_arch = "wasm32"))]
            //     widget::<matchmaking::MatchmakingMenu>(world, ui, id.with("matchmaking"), ())
            // }
            MenuPage::PlayerSelect => {
                widget::<player_select::PlayerSelectMenu>(world, ui, id.with("player-select"), ())
            }
            MenuPage::MapSelect { is_waiting } => {
                widget::<map_select::MapSelectMenu>(world, ui, id.with("map-select"), is_waiting)
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

                    // Local Game
                    let local_game_button = BorderedButton::themed(
                        &ui_theme.button_styles.normal,
                        &params.localization.get("local-game"),
                    )
                    .min_size(min_button_size)
                    .show(ui)
                    .focus_by_default(ui);

                    if local_game_button.clicked() {
                        *params.menu_page = MenuPage::PlayerSelect;
                    }

                    // Online Game
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.scope(|ui| {
                            ui.set_enabled(false);
                            let online_game_button = BorderedButton::themed(
                                &ui_theme.button_styles.normal,
                                &params.localization.get("online-game"),
                            )
                            .min_size(min_button_size)
                            .show(ui);

                            if online_game_button.clicked() {
                                todo!();
                                // *params.menu_page = MenuPage::Matchmaking;
                            }
                        });
                    }

                    // Map editor
                    ui.scope(|ui| {
                        ui.set_enabled(false);
                        if BorderedButton::themed(
                            &ui_theme.button_styles.normal,
                            &params.localization.get("map-editor"),
                        )
                        .min_size(min_button_size)
                        .show(ui)
                        .clicked()
                        {
                            params
                                .commands
                                .insert_resource(NextState(GameEditorState::Visible));
                            params
                                .commands
                                .insert_resource(NextState(EngineState::InGame));
                        }
                    });

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
