use std::ops;

use serde::{Serialize, Deserialize};

use super::{Num, Vec2, vec2, UVec2, uvec2, IVec2, ivec2, cfg_if};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Size<T: Num + Copy> {
    #[serde(alias = "x")]
    pub width: T,
    #[serde(alias = "y")]
    pub height: T,
}

impl<T> Size<T> where T: Num + Copy {
    pub fn new(width: T, height: T) -> Self {
        Size {
            width,
            height,
        }
    }

    pub fn zero() -> Self {
        Size::new(T::zero(), T::zero())
    }
}

impl<T> Default for Size<T> where T: Num + Copy {
    fn default() -> Self {
        Size::new(T::zero(), T::zero())
    }
}

impl<T> ops::Add for Size<T> where T: Num + Copy {
    type Output = Size<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Size::new(self.width + rhs.width, self.height + rhs.height)
    }
}

impl<T> ops::AddAssign for Size<T> where T: Num + Copy + ops::AddAssign {
    fn add_assign(&mut self, rhs: Self) {
        self.width += rhs.width;
        self.height += rhs.height;
    }
}

impl<T> ops::Sub for Size<T> where T: Num + Copy {
    type Output = Size<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Size::new(self.width - rhs.width, self.height - rhs.height)
    }
}

impl<T> ops::SubAssign for Size<T> where T: Num + Copy + ops::SubAssign {
    fn sub_assign(&mut self, rhs: Self) {
        self.width -= rhs.width;
        self.height -= rhs.height;
    }
}

impl<T> ops::Mul for Size<T> where T: Num + Copy {
    type Output = Size<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Size::new(self.width * rhs.width, self.height * rhs.height)
    }
}

impl<T> ops::MulAssign for Size<T> where T: Num + Copy + ops::MulAssign {
    fn mul_assign(&mut self, rhs: Self) {
        self.width *= rhs.width;
        self.height *= rhs.height;
    }
}

impl<T> ops::Div for Size<T> where T: Num + Copy {
    type Output = Size<T>;

    fn div(self, rhs: Self) -> Self::Output {
        Size::new(self.width / rhs.width, self.height / rhs.height)
    }
}

impl<T> ops::DivAssign for Size<T> where T: Num + Copy + ops::DivAssign {
    fn div_assign(&mut self, rhs: Self) {
        self.width /= rhs.width;
        self.height /= rhs.height;
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
