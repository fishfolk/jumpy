#[derive(Copy, Clone, Eq, PartialEq)]
pub enum RenderTarget {
    Context,
    Texture(usize),
}

impl RenderTarget {
    pub fn is_context(&self) -> bool {
        matches!(self, Self::Context)
    }

    pub fn is_texture(&self) -> bool {
        matches!(self, Self::Texture(..))
    }
}
