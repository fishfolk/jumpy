use std::sync::Arc;

use crate::assets::EguiFont;

use super::*;

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UIThemeMeta {
    pub scale: f32,
    pub font_families: HashMap<String, AssetHandle<EguiFont>>,
    pub font_styles: FontStylesMeta,
    pub button_styles: ButtonStylesMeta,
    pub hud: HudThemeMeta,
    pub panel: PanelThemeMeta,
    pub colors: UiThemeColors,
    pub widgets: UiThemeWidgets,
    pub debug_window_fill: ColorMeta,
    pub editor: UiThemeEditor,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UiThemeEditor {
    pub icons: UiThemeEditorIcons,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
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

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct UiThemeColors {
    pub positive: ColorMeta,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
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

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
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
            bg_fill: self.bg_fill.into_egui(),
            bg_stroke: egui::Stroke::new(2.0, self.bg_stroke.into_egui()),
            fg_stroke: egui::Stroke::new(2.0, self.text.into_egui()),
            rounding: egui::Rounding::same(border_radius),
            expansion: self.expansion,
        }
    }
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FontStylesMeta {
    pub normal: FontMeta,
    pub heading: FontMeta,
    pub bigger: FontMeta,
    pub smaller: FontMeta,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
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

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FontMeta {
    #[asset(deserialize_only)]
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

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct HudThemeMeta {
    pub player_hud_width: f32,
    pub portrait_frame: BorderImageMeta,
    pub font: FontMeta,
    pub lifebar: ProgressBarMeta,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PanelThemeMeta {
    #[serde(default)]
    pub font_color: ColorMeta,
    #[serde(default)]
    pub padding: MarginMeta,
    pub border: BorderImageMeta,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ButtonThemeMeta {
    pub font: FontMeta,
    #[serde(default)]
    pub padding: MarginMeta,
    pub borders: ButtonBordersMeta,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct BorderImageMeta {
    pub image: AssetHandle<Image>,
    pub image_size: UVec2,
    #[serde(default)]
    pub border_size: MarginMeta,
    #[serde(default = "f32_one")]
    pub scale: f32,
    #[serde(skip)]
    #[asset(deserialize_only)]
    pub egui_texture: egui::TextureId,
}

fn f32_one() -> f32 {
    1.0
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProgressBarMeta {
    pub height: f32,
    pub background_image: BorderImageMeta,
    pub progress_image: BorderImageMeta,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ButtonBordersMeta {
    pub default: BorderImageMeta,
    #[serde(default)]
    pub focused: Option<BorderImageMeta>,
    #[serde(default)]
    pub clicked: Option<BorderImageMeta>,
}

#[derive(BonesBevyAssetLoad, Default, Deserialize, Clone, Copy, Debug)]
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
