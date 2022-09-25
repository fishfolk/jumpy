use std::sync::Arc;

use bevy::utils::HashMap;
use bevy_egui::egui;

use crate::assets::EguiFont;

use super::*;

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UIThemeMeta {
    pub font_families: HashMap<String, String>,
    #[serde(skip)]
    pub font_handles: HashMap<String, Handle<EguiFont>>,
    pub font_styles: HashMap<FontStyle, FontMeta>,
    // pub font_sizes: HashMap<FontSize, f32>,
    pub hud: HudThemeMeta,
    pub panel: PanelThemeMeta,
    pub button_styles: HashMap<ButtonStyle, ButtonThemeMeta>,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
#[serde(try_from = "String")]
pub enum FontStyle {
    Heading,
    Normal,
    Bigger,
}

impl TryFrom<String> for FontStyle {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        use FontStyle::*;
        Ok(match value.as_str() {
            "heading" => Heading,
            "bigger" => Bigger,
            "normal" => Normal,
            _ => {
                return Err("Invalid font style");
            }
        })
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

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
#[serde(try_from = "String")]
pub enum ButtonStyle {
    Normal,
    Small,
}

impl TryFrom<String> for ButtonStyle {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        use ButtonStyle::*;
        Ok(match value.as_str() {
            "normal" => Normal,
            "small" => Small,
            _ => {
                return Err("Invalid button style");
            }
        })
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
pub struct ColorMeta([u8; 3]);

impl From<ColorMeta> for egui::Color32 {
    fn from(c: ColorMeta) -> Self {
        let [r, g, b] = c.0;
        egui::Color32::from_rgb(r, g, b)
    }
}

impl From<ColorMeta> for Color {
    fn from(c: ColorMeta) -> Self {
        let [r, g, b] = c.0;
        Color::rgb_u8(r, g, b)
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
