use crate::prelude::viewport;
use crate::texture::Texture2D;
use crate::viewport::Viewport;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum RenderTarget {
    Viewport,
    Texture(Texture2D),
}

impl RenderTarget {
    pub fn is_viewport(&self) -> bool {
        matches!(self, Self::Viewport)
    }

    pub fn is_texture(&self) -> bool {
        matches!(self, Self::Texture(..))
    }
}

impl Default for RenderTarget {
    fn default() -> Self {
        Self::Viewport
    }
}
