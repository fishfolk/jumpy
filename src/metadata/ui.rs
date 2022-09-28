use std::sync::Arc;

use bevy::utils::HashMap;
use bevy_egui::egui;

use crate::assets::EguiFont;

use super::*;

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UIThemeMeta {
    pub scale: f32,
    pub font_families: HashMap<String, String>,
    #[serde(skip)]
    pub font_handles: HashMap<String, Handle<EguiFont>>,
    pub font_styles: FontStylesMeta,
    pub button_styles: ButtonStylesMeta,
    pub hud: HudThemeMeta,
    pub panel: PanelThemeMeta,
    pub colors: UiThemeColors,
    pub widgets: UiThemeWidgets,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UiThemeColors {
    pub positive: ColorMeta,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UiThemeWidgets {
    pub border_radius: f32,

    pub default: WidgetColors,
    pub disabled: WidgetColors,
    pub hovered: WidgetColors,
    pub noninteractive: WidgetColors,
    pub menu: WidgetColors,
}

impl UiThemeWidgets {
    pub fn get_egui_widget_style(&self) -> egui::style::Widgets {
        egui::style::Widgets {
            noninteractive: self
                .noninteractive
                .get_egui_widget_visuals(self.border_radius, 0.0),
            inactive: self
                .disabled
                .get_egui_widget_visuals(self.border_radius, 0.0),
            hovered: self
                .hovered
                .get_egui_widget_visuals(self.border_radius, 1.0),
            active: self
                .default
                .get_egui_widget_visuals(self.border_radius, 1.0),
            open: self.menu.get_egui_widget_visuals(self.border_radius, 0.0),
        }
    }
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WidgetColors {
    bg_fill: ColorMeta,
    bg_stroke: ColorMeta,
    text: ColorMeta,
}

impl WidgetColors {
    pub fn get_egui_widget_visuals(
        &self,
        border_radius: f32,
        expansion: f32,
    ) -> egui::style::WidgetVisuals {
        egui::style::WidgetVisuals {
            bg_fill: self.bg_fill.into(),
            bg_stroke: egui::Stroke::new(2.0, self.bg_stroke),
            fg_stroke: egui::Stroke::new(2.0, self.text),
            rounding: egui::Rounding::same(border_radius),
            expansion,
        }
    }
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FontStylesMeta {
    pub normal: FontMeta,
    pub heading: FontMeta,
    pub bigger: FontMeta,
    pub smaller: FontMeta,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ButtonStylesMeta {
    pub normal: ButtonThemeMeta,
    pub small: ButtonThemeMeta,
}

impl ButtonStylesMeta {
    pub fn as_list(&mut self) -> [&mut ButtonThemeMeta; 2] {
        [&mut self.normal, &mut self.small]
    }
}

/// This is a helper to reduce clones on font family strings for egui
#[derive(Deserialize, Clone, Debug)]
#[serde(from = "String")]
pub struct FontFamily(Arc<str>);

impl From<String> for FontFamily {
    fn from(s: String) -> Self {
        Self(Arc::from(s))
    }
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[has_load_progress(none)]
pub struct FontMeta {
    pub family: FontFamily,
    pub size: f32,
    #[serde(default)]
    pub color: ColorMeta,
}

impl FontMeta {
    pub fn colored(&self, color: ColorMeta) -> Self {
        let mut meta = self.clone();
        meta.color = color;
        meta
    }

    pub fn font_id(&self) -> egui::FontId {
        egui::FontId {
            size: self.size,
            family: egui::FontFamily::Name(self.family.0.clone()),
        }
    }
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct HudThemeMeta {
    pub player_hud_width: f32,
    pub portrait_frame: BorderImageMeta,
    pub font: FontMeta,
    pub lifebar: ProgressBarMeta,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PanelThemeMeta {
    #[serde(default)]
    pub font_color: ColorMeta,
    #[serde(default)]
    pub padding: MarginMeta,
    pub border: BorderImageMeta,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ButtonThemeMeta {
    pub font: FontMeta,
    #[serde(default)]
    pub padding: MarginMeta,
    pub borders: ButtonBordersMeta,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct BorderImageMeta {
    pub image: String,
    pub image_size: UVec2,
    #[serde(default)]
    pub border_size: MarginMeta,
    #[serde(default = "f32_one")]
    pub scale: f32,

    #[serde(skip)]
    pub handle: Handle<Image>,
    #[serde(skip)]
    pub egui_texture: egui::TextureId,
}

fn f32_one() -> f32 {
    1.0
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProgressBarMeta {
    pub height: f32,
    pub background_image: BorderImageMeta,
    pub progress_image: BorderImageMeta,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ButtonBordersMeta {
    pub default: BorderImageMeta,
    #[serde(default)]
    pub focused: Option<BorderImageMeta>,
    #[serde(default)]
    pub clicked: Option<BorderImageMeta>,
}

#[derive(HasLoadProgress, Default, Deserialize, Clone, Copy, Debug)]
#[serde(deny_unknown_fields)]
#[has_load_progress(none)]
pub struct ColorMeta([u8; 4]);

impl From<ColorMeta> for egui::Color32 {
    fn from(c: ColorMeta) -> Self {
        let [r, g, b, a] = c.0;
        egui::Color32::from_rgba_premultiplied(r, g, b, a)
    }
}

impl From<ColorMeta> for Color {
    fn from(c: ColorMeta) -> Self {
        let [r, g, b, a] = c.0;
        Color::rgba_u8(r, g, b, a)
    }
}

#[derive(HasLoadProgress, Default, Deserialize, Clone, Copy, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct MarginMeta {
    #[serde(default)]
    pub top: f32,
    #[serde(default)]
    pub bottom: f32,
    #[serde(default)]
    pub left: f32,
    #[serde(default)]
    pub right: f32,
}

impl From<MarginMeta> for bevy_egui::egui::style::Margin {
    fn from(m: MarginMeta) -> Self {
        Self {
            left: m.left,
            right: m.right,
            top: m.top,
            bottom: m.bottom,
        }
    }
}
