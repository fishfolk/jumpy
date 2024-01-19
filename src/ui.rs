use crate::prelude::*;

pub mod main_menu;
pub mod map_select;
pub mod pause_menu;

#[cfg(not(target_arch = "wasm32"))]
pub mod network_game;

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
