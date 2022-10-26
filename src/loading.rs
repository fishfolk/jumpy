use bevy::{ecs::system::SystemParam, render::camera::ScalingMode};
use bevy_egui::{egui, EguiContext};
use bevy_fluent::Locale;
use bevy_has_load_progress::{HasLoadProgress, LoadingResources};
use bevy_mod_js_scripting::ActiveScripts;
use leafwing_input_manager::{
    axislike::{AxisType, SingleAxis},
    prelude::InputMap,
    InputManagerBundle,
};

use crate::{
    camera::{spawn_editor_camera, spawn_game_camera},
    config::ENGINE_CONFIG,
    metadata::{BorderImageMeta, GameMeta, PlayerMeta, Settings},
    platform::Storage,
    player::{input::PlayerInputs, MAX_PLAYERS},
    prelude::*,
    ui::input::MenuAction,
    GameState,
};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system(
            load_game
                .run_in_state(GameState::LoadingGameData)
                .run_if(game_assets_loaded),
        );

        // Configure hot reload
        if ENGINE_CONFIG.hot_reload {
            app.add_system_to_stage(CoreStage::Last, hot_reload_game)
                .add_system_set_to_stage(
                    CoreStage::Last,
                    ConditionSet::new().run_in_state(GameState::InGame).into(),
                );
        }
    }
}

#[derive(Component)]
pub struct PlayerInputCollector(pub usize);

fn setup(mut commands: Commands) {
    commands
        .spawn()
        .insert(Name::new("Menu Input Collector"))
        .insert_bundle(InputManagerBundle {
            input_map: menu_input_map(),
            ..default()
        });

    // Spawn the game camera
    spawn_game_camera(&mut commands);

    // Spawn the editor camera
    spawn_editor_camera(&mut commands);
}

// Condition system used to make sure game assets have loaded
fn game_assets_loaded(
    game_handle: Res<Handle<GameMeta>>,
    loading_resources: LoadingResources,
    game_assets: Res<Assets<GameMeta>>,
) -> bool {
    if let Some(game) = game_assets.get(&game_handle) {
        // Track load progress
        let load_progress = game.load_progress(&loading_resources);
        debug!(
            %load_progress,
            "Loading game assets: {:.2}% ",
            load_progress.as_percent() * 100.0
        );

        // Wait until assets are loaded to start game
        let done = load_progress.as_percent() >= 1.0;

        if done {
            info!("Game assets loaded");
        }

        done
    } else {
        false
    }
}

/// System param used to load and hot reload the game
#[derive(SystemParam)]
pub struct GameLoader<'w, 's> {
    skip_next_asset_update_event: Local<'s, bool>,
    camera_projections: Query<'w, 's, &'static mut OrthographicProjection>,
    commands: Commands<'w, 's>,
    game_handle: Res<'w, Handle<GameMeta>>,
    clear_color: ResMut<'w, ClearColor>,
    game_assets: ResMut<'w, Assets<GameMeta>>,
    egui_ctx: Option<ResMut<'w, EguiContext>>,
    events: EventReader<'w, 's, AssetEvent<GameMeta>>,
    active_scripts: ResMut<'w, ActiveScripts>,
    storage: ResMut<'w, Storage>,
    player_assets: ResMut<'w, Assets<PlayerMeta>>,
    texture_atlas_assets: Res<'w, Assets<TextureAtlas>>,
    player_inputs: ResMut<'w, PlayerInputs>,
}

impl<'w, 's> GameLoader<'w, 's> {
    /// This function is called once when the game starts up and, when hot reload is enabled, on
    /// update, to check for asset changed events and to update the [`GameMeta`] resource.
    ///
    /// The `is_hot_reload` argument is used to indicate whether the function should check for asset
    /// updates and reload, or whether it should run the one-time initialization of the game.
    fn load(mut self, is_hot_reload: bool) {
        // Check to make sure we shouldn't skip this execution
        // ( i.e. if this is a hot reload run without any changed assets )
        if self.should_skip_run(is_hot_reload) {
            return;
        }

        let Self {
            mut skip_next_asset_update_event,
            mut camera_projections,
            mut commands,
            game_handle,
            mut game_assets,
            mut egui_ctx,
            mut active_scripts,
            mut clear_color,
            mut storage,
            mut player_assets,
            texture_atlas_assets,
            mut player_inputs,
            ..
        } = self;

        if let Some(game) = game_assets.get_mut(&game_handle) {
            // Hot reload preparation
            if is_hot_reload {
                // Since we are modifying the game asset, which will trigger another asset changed
                // event, we need to skip the next update event.
                *skip_next_asset_update_event = true;

                // Clear the active scripts
                active_scripts.clear();

            // One-time initialization
            } else {
                if let Some(ctx) = egui_ctx.as_mut() {
                    // Initialize empty fonts for all game fonts.
                    //
                    // This makes sure Egui will not panic if we try to use a font that is still loading.
                    let mut egui_fonts = egui::FontDefinitions::default();
                    for font_name in game.ui_theme.font_families.keys() {
                        let font_family = egui::FontFamily::Name(font_name.clone().into());
                        egui_fonts.families.insert(font_family, vec![]);
                    }
                    ctx.ctx_mut().set_fonts(egui_fonts.clone());
                    commands.insert_resource(egui_fonts);
                }

                // Spawn player input collectors.
                let settings = storage.get(Settings::STORAGE_KEY);
                let settings = settings.as_ref().unwrap_or(&game.default_settings);
                for player in 0..MAX_PLAYERS {
                    commands
                        .spawn()
                        .insert(Name::new(format!("Player Input Collector {player}")))
                        .insert(PlayerInputCollector(player))
                        .insert_bundle(InputManagerBundle {
                            input_map: settings.player_controls.get_input_map(player),
                            ..default()
                        });
                }

                // Select default character for all players
                for player in &mut player_inputs.players {
                    player.selected_player = game.player_handles[0].clone_weak();
                }

                if ENGINE_CONFIG.server_mode {
                    // Go to server player selection state
                    commands.insert_resource(NextState(GameState::ServerPlayerSelect));
                } else {
                    // Transition to the main menu when we are done
                    commands.insert_resource(NextState(GameState::MainMenu));
                }
            }

            // Update camera scaling mode
            for mut projection in &mut camera_projections {
                projection.scaling_mode = ScalingMode::FixedVertical(game.camera_height as f32);
            }

            // set the clear color
            clear_color.0 = game.clear_color.into();

            // Set the locale resource
            let translations = &game.translations;
            commands.insert_resource(
                Locale::new(translations.detected_locale.clone())
                    .with_default(translations.default_locale.clone()),
            );

            if let Some(egui_ctx) = &mut egui_ctx {
                let mut visuals = egui::Visuals::dark();
                visuals.widgets = game.ui_theme.widgets.get_egui_widget_style();
                egui_ctx.ctx_mut().set_visuals(visuals);

                // Helper to load border images
                let mut load_border_image = |border: &mut BorderImageMeta| {
                    border.egui_texture = egui_ctx.add_image(border.handle.inner.clone_weak());
                };

                // Add Border images to egui context
                load_border_image(&mut game.ui_theme.hud.portrait_frame);
                load_border_image(&mut game.ui_theme.panel.border);
                load_border_image(&mut game.ui_theme.hud.lifebar.background_image);
                load_border_image(&mut game.ui_theme.hud.lifebar.progress_image);
                for button in game.ui_theme.button_styles.as_list() {
                    load_border_image(&mut button.borders.default);
                    if let Some(border) = &mut button.borders.clicked {
                        load_border_image(border);
                    }
                    if let Some(border) = &mut button.borders.focused {
                        load_border_image(border);
                    }
                }
                // Add player sprite sheets to egui context
                for player_handle in &game.player_handles {
                    let player_meta = player_assets.get_mut(player_handle).unwrap();
                    let atlas = texture_atlas_assets
                        .get(&player_meta.spritesheet.atlas_handle)
                        .unwrap();
                    player_meta.spritesheet.egui_texture_id =
                        egui_ctx.add_image(atlas.texture.clone_weak());
                }

                // Add editor icons to egui context
                for icon in game.ui_theme.editor.icons.as_mut_list() {
                    icon.egui_texture_id = egui_ctx.add_image(icon.image_handle.inner.clone_weak());
                }
            }

            // Set the active scripts
            for script_handle in &game.script_handles {
                active_scripts.insert(script_handle.inner.clone_weak());
            }
            if ENGINE_CONFIG.server_mode {
                for script_handle in &game.server_script_handles {
                    active_scripts.insert(script_handle.inner.clone_weak());
                }
            } else {
                for script_handle in &game.client_script_handles {
                    active_scripts.insert(script_handle.inner.clone_weak());
                }
            }

            // Insert the game resource
            commands.insert_resource(game.clone());

            // If the game asset isn't loaded yet
        } else {
            trace!("Awaiting game load")
        }
    }

    // Run checks to see if we should skip running the system
    fn should_skip_run(&mut self, is_hot_reload: bool) -> bool {
        // If this is a hot reload run, check for modified asset events
        if is_hot_reload {
            let mut has_update = false;
            for (event, event_id) in self.events.iter_with_id() {
                if let AssetEvent::Modified { .. } = event {
                    // We may need to skip an asset update event
                    if *self.skip_next_asset_update_event {
                        *self.skip_next_asset_update_event = false;
                    } else {
                        debug!(%event_id, "Game updated");
                        has_update = true;
                    }
                }
            }

            // If there was no update, skip execution
            if !has_update {
                return true;
            }
        }

        false
    }
}

fn menu_input_map() -> InputMap<MenuAction> {
    InputMap::default()
        // Up
        .insert(KeyCode::Up, MenuAction::Up)
        .insert(GamepadButtonType::DPadUp, MenuAction::Up)
        .insert(
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
                positive_low: 0.5,
                negative_low: -1.0,
                value: None,
            },
            MenuAction::Up,
        )
        // Left
        .insert(KeyCode::Left, MenuAction::Left)
        .insert(GamepadButtonType::DPadLeft, MenuAction::Left)
        .insert(
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
                positive_low: 1.0,
                negative_low: -0.5,
                value: None,
            },
            MenuAction::Left,
        )
        // Down
        .insert(KeyCode::Down, MenuAction::Down)
        .insert(GamepadButtonType::DPadDown, MenuAction::Down)
        .insert(
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
                positive_low: 1.0,
                negative_low: -0.5,
                value: None,
            },
            MenuAction::Down,
        )
        // Right
        .insert(KeyCode::Right, MenuAction::Right)
        .insert(GamepadButtonType::DPadRight, MenuAction::Right)
        .insert(
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
                positive_low: 0.5,
                negative_low: -1.0,
                value: None,
            },
            MenuAction::Right,
        )
        // Confirm
        .insert(KeyCode::Return, MenuAction::Confirm)
        .insert(GamepadButtonType::South, MenuAction::Confirm)
        .insert(GamepadButtonType::Start, MenuAction::Confirm)
        // Back
        .insert(KeyCode::Escape, MenuAction::Back)
        .insert(GamepadButtonType::East, MenuAction::Back)
        // Toggle Fullscreen
        .insert(KeyCode::F11, MenuAction::ToggleFullscreen)
        .insert(GamepadButtonType::Mode, MenuAction::ToggleFullscreen)
        // Pause
        .insert(KeyCode::Escape, MenuAction::Pause)
        .insert(GamepadButtonType::Start, MenuAction::Pause)
        .build()
}

/// System to run the initial game load
fn load_game(loader: GameLoader) {
    loader.load(false);
}

/// System to check for asset changes and hot reload the game
fn hot_reload_game(loader: GameLoader) {
    loader.load(true);
}
