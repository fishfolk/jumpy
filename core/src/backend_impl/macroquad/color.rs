use macroquad::color::Color;

impl From<Color> for crate::color::Color {
    fn from(color: Color) -> Self {
        crate::color::Color::new(color.r, color.g, color.b, color.a)
    }
}

impl From<crate::color::Color> for Color {
    fn from(color: crate::color::Color) -> Self {
        Color::new(color.red, color.green, color.blue, color.alpha)
    }
}
