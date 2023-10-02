use crate::prelude::*;

pub mod main_menu;
pub mod pause_menu;

pub fn game_plugin(game: &mut Game) {
    game.insert_shared_resource(EguiInputHook::new(update_gamepad_ui_inputs));
}

/// Takes the player controls from gamepads and converts them to arrow key inputs for egui so that
/// you can navigate the menu with the gamepad.
fn update_gamepad_ui_inputs(game: &mut Game, input: &mut egui::RawInput) {
    let Some(player_controls) = game.shared_resource::<GlobalPlayerControls>() else { return };

    for player_control in player_controls.iter() {
        if player_control.just_moved {
            if player_control.move_direction.y > 0.1 {
                input.events.push(egui::Event::Key {
                    key: egui::Key::ArrowUp,
                    pressed: true,
                    repeat: false,
                    modifiers: default(),
                });
            } else if player_control.move_direction.y < -0.1 {
                input.events.push(egui::Event::Key {
                    key: egui::Key::ArrowDown,
                    pressed: true,
                    repeat: false,
                    modifiers: default(),
                });
            } else if player_control.move_direction.x < -0.1 {
                input.events.push(egui::Event::Key {
                    key: egui::Key::ArrowLeft,
                    pressed: true,
                    repeat: false,
                    modifiers: default(),
                });
            } else if player_control.move_direction.x > 0.1 {
                input.events.push(egui::Event::Key {
                    key: egui::Key::ArrowRight,
                    pressed: true,
                    repeat: false,
                    modifiers: default(),
                });
            }
        }

        if player_control.menu_confirm_just_pressed {
            input.events.push(egui::Event::Key {
                key: egui::Key::Enter,
                pressed: true,
                repeat: false,
                modifiers: default(),
            });
        }

        if player_control.menu_back_just_pressed {
            input.events.push(egui::Event::Key {
                key: egui::Key::Escape,
                pressed: true,
                repeat: false,
                modifiers: default(),
            });
        }
    }
}

#[derive(HasSchema, Clone, Debug)]
#[repr(C)]
pub struct UiTheme {
    pub scale: f64,
    pub colors: UiThemeColors,
    pub widgets: UiThemeWidgets,
    pub fonts: SVec<Handle<Font>>,
    pub font_styles: UiThemeFontStyles,
    pub buttons: UiThemeButtons,
    pub panel: UiThemePanel,
    pub editor: UiThemeEditor,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            scale: 1.0,
            colors: default(),
            widgets: default(),
            fonts: default(),
            buttons: default(),
            font_styles: default(),
            panel: default(),
            editor: default(),
        }
    }
}

#[derive(HasSchema, Debug, Default, Clone)]
#[repr(C)]
pub struct ImageMeta {
    image: Handle<Image>,
    image_size: Vec2,
}

#[derive(HasSchema, Default, Debug, Clone)]
#[repr(C)]
pub struct UiThemeColors {
    pub positive: Color,
}

#[derive(HasSchema, Default, Debug, Clone)]
#[repr(C)]
pub struct UiThemeWidgets {
    pub border_radius: f32,
    pub default: UiThemeWidgetColors,
    pub hovered: UiThemeWidgetColors,
    pub active: UiThemeWidgetColors,
    pub noninteractive: UiThemeWidgetColors,
    pub menu: UiThemeWidgetColors,
    pub window_fill: Color,
    pub panel: UiThemePanel,
}

#[derive(HasSchema, Default, Debug, Clone)]
#[repr(C)]
pub struct UiThemeWidgetColors {
    pub bg_fill: Color,
    pub bg_stroke: Color,
    pub text: Color,
}

#[derive(HasSchema, Default, Debug, Clone)]
#[repr(C)]
pub struct UiThemeFontStyles {
    pub heading: FontMeta,
    pub bigger: FontMeta,
    pub normal: FontMeta,
    pub smaller: FontMeta,
}

#[derive(HasSchema, Default, Debug, Clone)]
#[repr(C)]
pub struct UiThemeButtons {
    pub normal: ButtonThemeMeta,
    pub small: ButtonThemeMeta,
}

#[derive(HasSchema, Default, Debug, Clone)]
#[repr(C)]
pub struct UiThemePanel {
    pub font_color: Color,
    pub padding: MarginMeta,
    pub border: BorderImageMeta,
}

#[derive(HasSchema, Default, Debug, Clone)]
#[repr(C)]
pub struct UiThemeEditor {
    pub icons: UiThemeEditorIcons,
}

#[derive(HasSchema, Debug, Default, Clone)]
#[repr(C)]
pub struct UiThemeEditorIcons {
    pub elements: ImageMeta,
    pub tiles: ImageMeta,
    pub collisions: ImageMeta,
    pub select: ImageMeta,
}
