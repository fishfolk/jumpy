use serde::{Serialize, Deserialize};

use super::{Num, Vec2, vec2, UVec2, uvec2, IVec2, ivec2, cfg_if};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Size<T: Num> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> where T: Num {
    pub fn new(width: T, height: T) -> Self {
        Size {
            width,
            height,
        }
    }
}

impl<T> From<(T, T)> for Size<T> where T: Num + Copy {
    fn from(tpl: (T, T)) -> Self {
        Size::new(tpl.0, tpl.1)
    }
}

impl<T> From<Size<T>> for (T, T) where T: Num + Copy {
    fn from(size: Size<T>) -> Self {
        (size.width, size.height)
    }
}

impl<T> From<&[T; 2]> for Size<T> where T: Num + Copy {
    fn from(slice: &[T; 2]) -> Self {
        Size::new(slice[0], slice[1])
    }
}


impl<T> From<&Size<T>> for [T; 2] where T: Num + Copy {
    fn from(size: &Size<T>) -> Self {
        [size.width, size.height]
    }
}

impl From<IVec2> for Size<i32> {
    fn from(vec: IVec2) -> Self {
        Size::new(vec.x, vec.y)
    }
}

impl From<UVec2> for Size<u32> {
    fn from(vec: UVec2) -> Self {
        Size::new(vec.x, vec.y)
    }
}

impl From<Vec2> for Size<f32> {
    fn from(vec: Vec2) -> Self {
        Size::new(vec.x, vec.y)
    }
}

impl From<Size<i32>> for IVec2 {
    fn from(size: Size<i32>) -> Self {
        ivec2(size.width, size.height)
    }
}

impl From<Size<u32>> for UVec2 {
    fn from(size: Size<u32>) -> Self {
        uvec2(size.width, size.height)
    }
}

impl From<Size<f32>> for Vec2 {
    fn from(size: Size<f32>) -> Self {
        vec2(size.width, size.height)
    }
}

cfg_if! {
    if #[cfg(feature = "winit")] {
        impl<T> From<winit::dpi::PhysicalSize<T>> for Size<T> where T: Num {
            fn from(size: winit::dpi::PhysicalSize<T>) -> Self {
                Size::new(size.width, size.height)
            }
        }

        impl<T> From<Size<T>> for winit::dpi::PhysicalSize<T> where T: Num {
            fn from(size: Size<T>) -> Self {
                winit::dpi::PhysicalSize::new(size.width, size.height)
            }
        }
    }
}
