#![allow(dead_code)] // We haven't used all of these methods yet but we still want them there

/// Bordered button rendering
///
/// Adapted from <https://docs.rs/egui/0.18.1/src/egui/widgets/button.rs.html>
use bevy_egui::egui::{self, *};

use crate::{
    metadata::{BorderImageMeta, ButtonThemeMeta},
    ColorMetaExt,
};

use super::bordered_frame::BorderedFrame;

/// A button rendered with a [`BorderImageMeta`]
pub struct BorderedButton<'a> {
    text: WidgetText,
    wrap: Option<bool>,
    sense: Sense,
    min_size: Vec2,
    focus_on_hover: bool,
    default_border: Option<&'a BorderImageMeta>,
    on_focus_border: Option<&'a BorderImageMeta>,
    on_click_border: Option<&'a BorderImageMeta>,
    margin: egui::style::Margin,
    padding: egui::style::Margin,
}

impl<'a> BorderedButton<'a> {
    // Create a new button
    #[must_use = "You must call .show() to render the button"]
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            sense: Sense::click(),
            min_size: Vec2::ZERO,
            focus_on_hover: true,
            wrap: None,
            default_border: None,
            on_focus_border: None,
            on_click_border: None,
            margin: Default::default(),
            padding: Default::default(),
        }
    }

    #[must_use = "You must call .show() to render the button"]
    pub fn themed(
        button_theme: &'a ButtonThemeMeta,
        label: impl Into<RichText>,
    ) -> BorderedButton<'a> {
        BorderedButton::new(
            label
                .into()
                .font(button_theme.font.font_id())
                .color(button_theme.font.color.into_egui()),
        )
        .border(&button_theme.borders.default)
        .on_click_border(button_theme.borders.clicked.as_ref())
        .on_focus_border(button_theme.borders.focused.as_ref())
        .padding(button_theme.padding.into())
    }

    /// Set whether or not the button focuses itself automatically when it is hovered over.
    pub fn focus_on_hover(mut self, focus_on_hover: bool) -> Self {
        self.focus_on_hover = focus_on_hover;
        self
    }

    /// If `true`, the text will wrap to stay within the max width of the [`Ui`].
    ///
    /// By default [`Self::wrap`] will be true in vertical layouts
    /// and horizontal layouts with wrapping,
    /// and false on non-wrapping horizontal layouts.
    ///
    /// Note that any `\n` in the text will always produce a new line.
    #[inline]
    #[must_use = "You must call .show() to render the button"]
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = Some(wrap);
        self
    }

    /// Set the margin. This will be applied on the outside of the border.
    #[must_use = "You must call .show() to render the button"]
    pub fn margin(mut self, margin: egui::style::Margin) -> Self {
        self.margin = margin;

        self
    }

    /// Set the padding. This will be applied on the inside of the border.
    #[must_use = "You must call .show() to render the button"]
    pub fn padding(mut self, padding: egui::style::Margin) -> Self {
        self.padding = padding;

        self
    }

    /// Set the button border image
    #[must_use = "You must call .show() to render the button"]
    pub fn border(mut self, border: &'a BorderImageMeta) -> Self {
        self.default_border = Some(border);
        self
    }

    /// Set a different border to use when focusing / hovering over the button
    #[must_use = "You must call .show() to render the button"]
    pub fn on_focus_border(mut self, border: Option<&'a BorderImageMeta>) -> Self {
        self.on_focus_border = border;
        self
    }

    /// Set a different border to use when the mouse is clicking on the button
    #[must_use = "You must call .show() to render the button"]
    pub fn on_click_border(mut self, border: Option<&'a BorderImageMeta>) -> Self {
        self.on_click_border = border;
        self
    }

    /// By default, buttons senses clicks.
    /// Change this to a drag-button with `Sense::drag()`.
    #[must_use = "You must call .show() to render the button"]
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }

    /// Set the minimum size for the button
    #[must_use = "You must call .show() to render the button"]
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    /// Render the button
    #[must_use = "You must call .show() to render the button"]
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        self.ui(ui)
    }
}

impl<'a> Widget for BorderedButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let BorderedButton {
            text,
            sense,
            min_size,
            focus_on_hover,
            wrap,
            default_border,
            on_focus_border,
            on_click_border,
            margin,
            padding,
        }: BorderedButton = self;

        let total_extra = padding.sum() + margin.sum();

        let wrap_width = ui.available_width() - total_extra.x;
        let text = text.into_galley(ui, wrap, wrap_width, TextStyle::Button);

        let mut desired_size = text.size() + total_extra;
        desired_size = desired_size.at_least(min_size);

        let (rect, response) = ui.allocate_at_least(desired_size, sense);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, text.text()));

        // Focus the button automatically when it is hovered and the mouse is moving
        if response.hovered()
            && ui.ctx().input().pointer.velocity().length_sq() > 0.0
            && focus_on_hover
        {
            response.request_focus();
        }

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            let mut text_rect = rect;
            text_rect.min += padding.left_top() + margin.left_top();
            text_rect.max -= padding.right_bottom() + margin.right_bottom();
            text_rect.max.x = text_rect.max.x.max(text_rect.min.x);
            text_rect.max.y = text_rect.max.y.max(text_rect.min.y);

            let label_pos = ui
                .layout()
                .align_size_within_rect(text.size(), text_rect)
                .min;

            let border = if response.is_pointer_button_down_on() {
                on_click_border.or(default_border)
            } else if response.has_focus() || response.hovered() {
                on_focus_border.or(default_border)
            } else {
                default_border
            };

            let mut border_rect = rect;
            border_rect.min += margin.left_top();
            border_rect.max -= margin.right_bottom();
            border_rect.max.x = border_rect.max.x.max(border_rect.min.x);
            border_rect.max.y = border_rect.max.y.max(border_rect.min.y);

            if let Some(border) = border {
                ui.painter()
                    .add(BorderedFrame::new(border).paint(border_rect));
            }

            text.paint_with_visuals(ui.painter(), label_pos, visuals);
        }

        response
    }
}
