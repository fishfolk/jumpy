use bevy::ecs::system::SystemState;

use crate::prelude::*;

use self::input::MenuAction;

pub mod input;
pub mod widgets;

pub mod debug_tools;
// pub mod editor;
pub mod main_menu;
pub mod pause_menu;

pub struct JumpyUiPlugin;

impl Plugin for JumpyUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_egui::EguiPlugin)
            .add_plugin(input::UiInputPlugin)
            .add_plugin(main_menu::MainMenuPlugin)
            // .add_plugin(editor::EditorPlugin)
            .add_plugin(debug_tools::DebugToolsPlugin)
            .add_plugin(pause_menu::PausePlugin)
            .init_resource::<WidgetAdjacencies>()
            .init_resource::<DisableMenuInput>()
            .add_system_to_stage(
                CoreStage::PreUpdate,
                handle_menu_input
                    .run_if_resource_exists::<GameMeta>()
                    .after(leafwing_input_manager::plugin::InputManagerSystem::Update)
                    .after(bevy_egui::EguiSystem::ProcessInput)
                    .before(bevy_egui::EguiSystem::BeginFrame),
            )
            .add_system(update_egui_fonts)
            .add_system(update_ui_scale.run_if_resource_exists::<GameMeta>());
    }
}

/// Awesome widget system shared by @aevyrie:
/// <https://github.com/bevyengine/bevy/discussions/5522>
pub trait WidgetSystem: SystemParam {
    type Args;
    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut egui::Ui,
        id: WidgetId,
        args: Self::Args,
    );
}

pub fn widget<S: 'static + WidgetSystem>(
    world: &mut World,
    ui: &mut egui::Ui,
    id: WidgetId,
    args: S::Args,
) {
    // We need to cache `SystemState` to allow for a system's locally tracked state
    if !world.contains_resource::<StateInstances<S>>() {
        // Note, this message should only appear once! If you see it twice in the logs, the function
        // may have been called recursively, and will panic.
        trace!("Init widget system state {}", std::any::type_name::<S>());
        world.insert_resource(StateInstances::<S> {
            instances: HashMap::new(),
        });
    }
    world.resource_scope(|world, mut states: Mut<StateInstances<S>>| {
        if !states.instances.contains_key(&id) {
            trace!(
                "Registering widget system state for widget {id:?} of type {}",
                std::any::type_name::<S>()
            );
            states.instances.insert(id, SystemState::new(world));
        }
        let cached_state = states.instances.get_mut(&id).unwrap();
        S::system(world, cached_state, ui, id, args);
        cached_state.apply(world);
    });
}

/// A UI widget may have multiple instances. We need to ensure the local state of these instances is
/// not shared. This hashmap allows us to dynamically store instance states.
#[derive(Resource, Default)]
struct StateInstances<S: WidgetSystem + 'static> {
    instances: HashMap<WidgetId, SystemState<S>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);
impl WidgetId {
    pub fn new(name: &str) -> Self {
        use std::hash::Hasher;
        let bytes = name.as_bytes();
        let mut hasher = bevy::utils::AHasher::default();
        hasher.write(bytes);
        WidgetId(hasher.finish())
    }
    pub fn with(&self, name: &str) -> WidgetId {
        Self::new(&format!("{}{name}", self.0))
    }
}

impl From<&str> for WidgetId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Resource that stores which ui widgets are adjacent to which other widgets.
///
/// This is used to figure out which widget to focus on next when you press a direction on the
/// gamepad, for instance.
#[derive(Resource, Debug, Clone, Default)]
pub struct WidgetAdjacencies {
    pub map: HashMap<egui::Id, WidgetAdjacency>,
    /// These widgets will have the focus change when pressing directional inputs
    pub text_boxes: HashSet<egui::Id>,
}

/// The list of widgets in each direction from another widget
#[derive(Debug, Clone, Default)]
pub struct WidgetAdjacency {
    pub up: Option<egui::Id>,
    pub down: Option<egui::Id>,
    pub left: Option<egui::Id>,
    pub right: Option<egui::Id>,
}

impl WidgetAdjacencies {
    pub fn widget(&mut self, resp: &egui::Response) -> WidgetAdjacencyEntry {
        WidgetAdjacencyEntry {
            id: resp.id,
            adjacencies: self,
        }
    }
}

pub struct WidgetAdjacencyEntry<'a> {
    id: egui::Id,
    adjacencies: &'a mut WidgetAdjacencies,
}

#[allow(clippy::wrong_self_convention)]
impl<'a> WidgetAdjacencyEntry<'a> {
    pub fn to_left_of(self, resp: &egui::Response) -> Self {
        let other_id = resp.id;
        self.adjacencies.map.entry(self.id).or_default().right = Some(other_id);
        self.adjacencies.map.entry(other_id).or_default().left = Some(self.id);
        self
    }
    pub fn to_right_of(self, resp: &egui::Response) -> Self {
        let other_id = resp.id;
        self.adjacencies.map.entry(self.id).or_default().left = Some(other_id);
        self.adjacencies.map.entry(other_id).or_default().right = Some(self.id);
        self
    }
    pub fn above(self, resp: &egui::Response) -> Self {
        let other_id = resp.id;
        self.adjacencies.map.entry(self.id).or_default().down = Some(other_id);
        self.adjacencies.map.entry(other_id).or_default().up = Some(self.id);
        self
    }
    pub fn below(self, resp: &egui::Response) -> Self {
        let other_id = resp.id;
        self.adjacencies.map.entry(self.id).or_default().up = Some(other_id);
        self.adjacencies.map.entry(other_id).or_default().down = Some(self.id);
        self
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct DisableMenuInput(pub bool);

fn handle_menu_input(
    disable_menu_input: Res<DisableMenuInput>,
    mut windows: ResMut<Windows>,
    input: Query<&ActionState<MenuAction>>,
    keyboard: Res<Input<KeyCode>>,
    mut egui_inputs: ResMut<bevy_egui::EguiRenderInputContainer>,
    adjacencies: Res<WidgetAdjacencies>,
    mut egui_ctx: ResMut<bevy_egui::EguiContext>,
) {
    let input = input.single();

    // Handle fullscreen toggling
    if input.just_pressed(MenuAction::ToggleFullscreen) {
        if let Some(window) = windows.get_primary_mut() {
            window.set_mode(match window.mode() {
                WindowMode::BorderlessFullscreen => WindowMode::Windowed,
                _ => WindowMode::BorderlessFullscreen,
            });
        }
    }

    let events = &mut egui_inputs
        .get_mut(&bevy::window::WindowId::primary())
        .unwrap()
        .events;

    if **disable_menu_input {
        events.retain(|event| match event {
            egui::Event::Key { key, .. } => key == &egui::Key::Escape,
            _ => true,
        });
        return;
    }

    if input.just_pressed(MenuAction::Confirm) {
        events.push(egui::Event::Key {
            key: egui::Key::Enter,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        });
    }

    // Helper to fall back on using tab order instead of adjacency map to determine next focused
    // widget.
    let mut tab_fallback = || {
        if input.just_pressed(MenuAction::Up) || input.just_pressed(MenuAction::Left) {
            events.push(egui::Event::Key {
                key: egui::Key::Tab,
                pressed: true,
                modifiers: egui::Modifiers::SHIFT,
            });
        } else if input.just_pressed(MenuAction::Down) || input.just_pressed(MenuAction::Right) {
            events.push(egui::Event::Key {
                key: egui::Key::Tab,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            });
        }
    };

    let mut memory = egui_ctx.ctx_mut().memory();
    let focused = memory.focus();
    let is_text_box = focused
        .map(|id| adjacencies.text_boxes.contains(&id))
        .unwrap_or(false);

    if !(is_text_box
        && (keyboard.pressed(KeyCode::Up)
            || keyboard.pressed(KeyCode::Down)
            || keyboard.pressed(KeyCode::Left)
            || keyboard.pressed(KeyCode::Right)))
    {
        if let Some(adjacency) = memory.focus().and_then(|id| adjacencies.map.get(&id)) {
            if input.just_pressed(MenuAction::Up) {
                if let Some(adjacent) = adjacency.up {
                    memory.request_focus(adjacent);
                } else {
                    tab_fallback()
                }
            } else if input.just_pressed(MenuAction::Down) {
                if let Some(adjacent) = adjacency.down {
                    memory.request_focus(adjacent);
                } else {
                    tab_fallback()
                }
            } else if input.just_pressed(MenuAction::Left) {
                if let Some(adjacent) = adjacency.left {
                    memory.request_focus(adjacent);
                } else {
                    tab_fallback()
                }
            } else if input.just_pressed(MenuAction::Right) {
                if let Some(adjacent) = adjacency.right {
                    memory.request_focus(adjacent);
                } else {
                    tab_fallback()
                }
            }
        } else {
            tab_fallback();
        }
    }
}

/// Resource containing the font definitions to use for Egui.
#[derive(Resource, Deref, DerefMut)]
pub struct EguiFontDefinitions(pub egui::FontDefinitions);

/// Watches for asset events for [`EguiFont`] assets and updates the corresponding fonts from the
/// [`GameMeta`], inserting the font data into the egui context.
fn update_egui_fonts(
    mut font_queue: Local<Vec<Handle<EguiFont>>>,
    mut egui_ctx: ResMut<bevy_egui::EguiContext>,
    egui_font_definitions: Option<ResMut<EguiFontDefinitions>>,
    game: Option<Res<GameMeta>>,
    mut events: EventReader<AssetEvent<EguiFont>>,
    assets: Res<Assets<EguiFont>>,
) {
    // Add any newly updated/created fonts to the queue
    for event in events.iter() {
        if let AssetEvent::Created { handle } | AssetEvent::Modified { handle } = event {
            font_queue.push(handle.clone_weak());
        }
    }

    // Update queued fonts if the game is ready
    if let Some((game, mut egui_font_definitions)) = game.zip(egui_font_definitions) {
        for handle in font_queue.drain(..) {
            // Get the game font name associated to this handle
            let name = game
                .ui_theme
                .font_families
                .iter()
                .find_map(|(font_name, font_handle)| {
                    if font_handle.inner == handle {
                        Some(font_name.clone())
                    } else {
                        None
                    }
                });

            // If we were able to find the font handle in our game fonts
            if let Some(font_name) = name {
                // Get the font asset
                if let Some(font) = assets.get(&handle) {
                    // And insert it into the Egui font definitions
                    let ctx = egui_ctx.ctx_mut();
                    egui_font_definitions
                        .font_data
                        .insert(font_name.clone(), font.0.clone());

                    egui_font_definitions
                        .families
                        .get_mut(&egui::FontFamily::Name(font_name.clone().into()))
                        .unwrap()
                        .push(font_name);

                    ctx.set_fonts(egui_font_definitions.clone());
                }
            }
        }
    }
}

fn update_ui_scale(
    game_meta: Res<GameMeta>,
    mut egui_settings: ResMut<bevy_egui::EguiSettings>,
    windows: Res<Windows>,
    projection: Query<&OrthographicProjection, With<MenuCamera>>,
) {
    if let Some(window) = windows.get_primary() {
        if let Ok(projection) = projection.get_single() {
            match projection.scaling_mode {
                bevy::render::camera::ScalingMode::FixedVertical(height) => {
                    let window_height = window.height();
                    let scale = window_height / height;
                    egui_settings.scale_factor = (scale * game_meta.ui_theme.scale) as f64;
                }
                bevy::render::camera::ScalingMode::FixedHorizontal(width) => {
                    let window_width = window.width();
                    let scale = window_width / width;
                    egui_settings.scale_factor = (scale * game_meta.ui_theme.scale) as f64;
                }
                bevy::render::camera::ScalingMode::Auto { .. } => (),
                bevy::render::camera::ScalingMode::None => (),
                bevy::render::camera::ScalingMode::WindowSize => (),
            }
        }
    }
}
