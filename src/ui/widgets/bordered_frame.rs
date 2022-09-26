use bevy_egui::egui;

use crate::metadata::ui::BorderImageMeta;

/// A 9-patch style bordered frame.
///
/// # See Also
///
/// - [`UiBorderImage`]
pub struct BorderedFrame {
    bg_texture: egui::TextureId,
    border_scale: f32,
    texture_size: egui::Vec2,
    texture_border_size: egui::style::Margin,
    padding: egui::style::Margin,
    margin: egui::style::Margin,
    border_only: bool,
}

impl BorderedFrame {
    /// Create a new frame with the given [`BorderImage`]
    #[must_use = "You must call .show() to render the frame"]
    pub fn new(border_image: &BorderImageMeta) -> Self {
        let s = border_image.image_size;
        Self {
            bg_texture: border_image.egui_texture,
            border_scale: border_image.scale,
            texture_size: egui::Vec2::new(s.x as f32, s.y as f32),
            texture_border_size: border_image.border_size.into(),
            padding: Default::default(),
            margin: Default::default(),
            border_only: false,
        }
    }

    /// Set the padding. This will be applied on the inside of the border.
    #[must_use = "You must call .show() to render the frame"]
    pub fn padding(mut self, margin: egui::style::Margin) -> Self {
        self.padding = margin;

        self
    }

    /// Set the margin. This will be applied on the outside of the border.
    #[must_use = "You must call .show() to render the frame"]
    pub fn margin(mut self, margin: egui::style::Margin) -> Self {
        self.margin = margin;

        self
    }

    #[allow(unused)] // We just haven't needed this method yet
    #[must_use = "You must call .show() to render the frame"]
    pub fn border_scale(mut self, scale: f32) -> Self {
        self.border_scale = scale;

        self
    }

    #[allow(unused)] // We just haven't needed this method yet
    /// If border_only is set to `true`, then the middle section of the frame will be transparent,
    /// only the border will be rendered.
    #[must_use = "You must call .show() to render the frame"]
    pub fn border_only(mut self, border_only: bool) -> Self {
        self.border_only = border_only;

        self
    }

    /// Render the frame
    pub fn show<R>(
        self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        self.show_dyn(ui, Box::new(add_contents))
    }

    fn show_dyn<'c, R>(
        self,
        ui: &mut egui::Ui,
        add_contents: Box<dyn FnOnce(&mut egui::Ui) -> R + 'c>,
    ) -> egui::InnerResponse<R> {
        let mut prepared = self.begin(ui);
        let ret = add_contents(&mut prepared.content_ui);
        let response = prepared.end(ui);

        egui::InnerResponse {
            inner: ret,
            response,
        }
    }

    fn begin(self, ui: &mut egui::Ui) -> BorderedFramePrepared {
        let background_shape_idx = ui.painter().add(egui::Shape::Noop);

        let mut content_rect = ui.available_rect_before_wrap();
        content_rect.min += self.padding.left_top() + self.margin.left_top();
        content_rect.max -= self.padding.right_bottom() + self.margin.right_bottom();

        // Avoid negative size
        content_rect.max.x = content_rect.max.x.max(content_rect.min.x);
        content_rect.max.y = content_rect.max.y.max(content_rect.min.y);

        let content_ui = ui.child_ui(content_rect, *ui.layout());

        BorderedFramePrepared {
            frame: self,
            background_shape_idx,
            content_ui,
        }
    }

    pub fn paint(&self, paint_rect: egui::Rect) -> egui::Shape {
        use egui::{Pos2, Rect, Vec2};
        let white = egui::Color32::WHITE;

        let mut mesh = egui::Mesh {
            texture_id: self.bg_texture,
            ..Default::default()
        };

        let s = self.texture_size;
        let b = self.texture_border_size;
        let pr = paint_rect;
        // UV border
        let buv = egui::style::Margin {
            left: b.left / s.x,
            right: b.right / s.x,
            top: b.top / s.y,
            bottom: b.bottom / s.y,
        };
        let b = egui::style::Margin {
            left: b.left * self.border_scale,
            right: b.right * self.border_scale,
            top: b.top * self.border_scale,
            bottom: b.bottom * self.border_scale,
        };

        // Build the 9-patches

        // Top left
        mesh.add_rect_with_uv(
            Rect::from_min_size(pr.min, Vec2::new(b.left, b.top)),
            egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(buv.left, buv.top)),
            white,
        );
        // Top center
        mesh.add_rect_with_uv(
            Rect::from_min_size(
                pr.min + Vec2::new(b.left, 0.0),
                Vec2::new(pr.width() - b.left - b.right, b.top),
            ),
            egui::Rect::from_min_size(
                Pos2::new(buv.left, 0.0),
                Vec2::new(1.0 - buv.left - buv.right, buv.top),
            ),
            white,
        );
        // Top right
        mesh.add_rect_with_uv(
            Rect::from_min_size(
                pr.right_top() - Vec2::new(b.right, 0.0),
                Vec2::new(b.right, b.top),
            ),
            egui::Rect::from_min_size(
                Pos2::new(1.0 - buv.right, 0.0),
                Vec2::new(buv.right, buv.top),
            ),
            white,
        );
        // Middle left
        mesh.add_rect_with_uv(
            Rect::from_min_size(
                pr.min + Vec2::new(0.0, b.top),
                Vec2::new(b.left, pr.height() - b.top - b.bottom),
            ),
            egui::Rect::from_min_size(
                Pos2::new(0.0, buv.top),
                Vec2::new(buv.left, 1.0 - buv.top - buv.bottom),
            ),
            white,
        );
        // Middle center
        if !self.border_only {
            mesh.add_rect_with_uv(
                Rect::from_min_size(
                    pr.min + Vec2::new(b.left, b.top),
                    Vec2::new(
                        pr.width() - b.left - b.right,
                        pr.height() - b.top - b.bottom,
                    ),
                ),
                egui::Rect::from_min_size(
                    Pos2::new(buv.left, buv.top),
                    Vec2::new(1.0 - buv.left - buv.top, 1.0 - buv.top - buv.bottom),
                ),
                white,
            );
        }
        // Middle right
        mesh.add_rect_with_uv(
            Rect::from_min_size(
                pr.min + Vec2::new(pr.width() - b.right, b.top),
                Vec2::new(b.right, pr.height() - b.top - b.bottom),
            ),
            egui::Rect::from_min_size(
                Pos2::new(1.0 - buv.right, buv.top),
                Vec2::new(buv.right, 1.0 - buv.top - buv.bottom),
            ),
            white,
        );
        // Bottom left
        mesh.add_rect_with_uv(
            Rect::from_min_size(
                pr.min + Vec2::new(0.0, pr.height() - b.bottom),
                Vec2::new(b.left, b.bottom),
            ),
            egui::Rect::from_min_size(
                Pos2::new(0.0, 1.0 - buv.bottom),
                Vec2::new(buv.left, buv.bottom),
            ),
            white,
        );
        // Bottom center
        mesh.add_rect_with_uv(
            Rect::from_min_size(
                pr.min + Vec2::new(b.left, pr.height() - b.bottom),
                Vec2::new(pr.width() - b.left - b.right, b.bottom),
            ),
            egui::Rect::from_min_size(
                Pos2::new(buv.left, 1.0 - buv.bottom),
                Vec2::new(1.0 - buv.left - buv.right, buv.bottom),
            ),
            white,
        );
        // Bottom right
        mesh.add_rect_with_uv(
            Rect::from_min_size(
                pr.min + Vec2::new(pr.width() - b.right, pr.height() - b.bottom),
                Vec2::new(b.right, b.bottom),
            ),
            egui::Rect::from_min_size(
                Pos2::new(1.0 - buv.right, 1.0 - buv.bottom),
                Vec2::new(buv.right, buv.bottom),
            ),
            white,
        );

        egui::Shape::Mesh(mesh)
    }
}

/// Internal helper struct for rendering the [`BorderedFrame`]
struct BorderedFramePrepared {
    frame: BorderedFrame,
    background_shape_idx: egui::layers::ShapeIdx,
    content_ui: egui::Ui,
}

impl BorderedFramePrepared {
    fn end(self, ui: &mut egui::Ui) -> egui::Response {
        use egui::Vec2;

        let min_rect = self.content_ui.min_rect();
        let m = self.frame.padding;
        let paint_rect = egui::Rect {
            min: min_rect.min - Vec2::new(m.left, m.top),
            max: min_rect.max + Vec2::new(m.right, m.bottom),
        };
        if ui.is_rect_visible(paint_rect) {
            let shape = self.frame.paint(paint_rect);
            ui.painter().set(self.background_shape_idx, shape);
        }

        ui.allocate_rect(paint_rect, egui::Sense::hover())
    }
}
