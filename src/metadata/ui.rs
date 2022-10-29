use std::sync::Arc;

use bevy::utils::HashMap;
use bevy_egui::egui;
use serde::Deserializer;

use crate::assets::EguiFont;

use super::*;

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UIThemeMeta {
    pub scale: f32,
    pub font_families: HashMap<String, String>,
    #[serde(skip)]
    pub font_handles: HashMap<String, AssetHandle<EguiFont>>,
    pub font_styles: FontStylesMeta,
    pub button_styles: ButtonStylesMeta,
    pub hud: HudThemeMeta,
    pub panel: PanelThemeMeta,
    pub colors: UiThemeColors,
    pub widgets: UiThemeWidgets,
    pub editor: UiThemeEditor,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UiThemeEditor {
    pub icons: UiThemeEditorIcons,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UiThemeEditorIcons {
    pub select: ImageMeta,
    pub spawn: ImageMeta,
    pub tile: ImageMeta,
    pub erase: ImageMeta,
}

impl UiThemeEditorIcons {
    pub fn as_mut_list(&mut self) -> [&mut ImageMeta; 4] {
        [
            &mut self.select,
            &mut self.spawn,
            &mut self.tile,
            &mut self.erase,
        ]
    }
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

    pub active: WidgetStyle,
    pub default: WidgetStyle,
    pub hovered: WidgetStyle,
    pub noninteractive: WidgetStyle,
    pub menu: WidgetStyle,
}

impl UiThemeWidgets {
    pub fn get_egui_widget_style(&self) -> egui::style::Widgets {
        egui::style::Widgets {
            noninteractive: self
                .noninteractive
                .get_egui_widget_visuals(self.border_radius),
            inactive: self.default.get_egui_widget_visuals(self.border_radius),
            hovered: self.hovered.get_egui_widget_visuals(self.border_radius),
            active: self.active.get_egui_widget_visuals(self.border_radius),
            open: self.menu.get_egui_widget_visuals(self.border_radius),
        }
    }
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WidgetStyle {
    pub bg_fill: ColorMeta,
    pub bg_stroke: ColorMeta,
    pub text: ColorMeta,
    #[serde(default)]
    pub expansion: f32,
}

impl WidgetStyle {
    pub fn get_egui_widget_visuals(&self, border_radius: f32) -> egui::style::WidgetVisuals {
        egui::style::WidgetVisuals {
            bg_fill: self.bg_fill.into(),
            bg_stroke: egui::Stroke::new(2.0, self.bg_stroke),
            fg_stroke: egui::Stroke::new(2.0, self.text),
            rounding: egui::Rounding::same(border_radius),
            expansion: self.expansion,
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

impl From<FontFamily> for egui::FontFamily {
    fn from(family: FontFamily) -> Self {
        Self::Name(family.0)
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

impl From<FontMeta> for egui::FontSelection {
    fn from(meta: FontMeta) -> Self {
        egui::FontSelection::FontId(egui::FontId {
            size: meta.size,
            family: meta.family.into(),
        })
    }
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
            family: self.family.clone().into(),
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
    pub handle: AssetHandle<Image>,
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

#[derive(Reflect, HasLoadProgress, Default, Clone, Copy, Debug)]
#[has_load_progress(none)]
pub struct ColorMeta(pub Color);

impl<'de> Deserialize<'de> for ColorMeta {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorVisitor)
    }
}

impl Serialize for ColorMeta {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!(
            "{:X}{:X}{:X}{:X}",
            (255.0 * self.0.r()) as u8,
            (255.0 * self.0.g()) as u8,
            (255.0 * self.0.b()) as u8,
            (255.0 * self.0.a()) as u8,
        ))
    }
}

struct ColorVisitor;
impl<'de> serde::de::Visitor<'de> for ColorVisitor {
    type Value = ColorMeta;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A hex-encoded RGB or RGBA color")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ColorMeta(Color::hex(v).map_err(|e| E::custom(e))?))
    }
}

impl From<ColorMeta> for egui::Color32 {
    fn from(c: ColorMeta) -> Self {
        let [r, g, b, a] = c.0.as_linear_rgba_f32();
        egui::Rgba::from_rgba_premultiplied(r, g, b, a).into()
    }
}

impl From<ColorMeta> for Color {
    fn from(c: ColorMeta) -> Self {
        c.0
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
