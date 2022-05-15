use std::ops;
use std::ops::{Div, DivAssign, Mul, MulAssign};

use serde::{Deserialize, Serialize};

use super::{cfg_if, ivec2, uvec2, vec2, IVec2, Num, UVec2, Vec2};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Size<T: Num + Copy> {
    #[serde(alias = "x")]
    pub width: T,
    #[serde(alias = "y")]
    pub height: T,
}

impl<T: Num + Copy> Size<T> {
    pub const fn new(width: T, height: T) -> Self {
        Size { width, height }
    }

    pub fn zero() -> Self {
        Size::new(T::zero(), T::zero())
    }
}

impl Size<f32> {
    pub fn to_scaled(self, scale: f32) -> Size<f32> {
        let mut res = self;
        res.width *= scale;
        res.height *= scale;
        res
    }
}

impl Mul<f32> for Size<f32> {
    type Output = Size<f32>;

    fn mul(self, rhs: f32) -> Self::Output {
        Size::new(self.width * rhs, self.height * rhs)
    }
}

impl MulAssign<f32> for Size<f32> {
    fn mul_assign(&mut self, rhs: f32) {
        self.width *= rhs;
        self.height *= rhs;
    }
}

impl Div<f32> for Size<f32> {
    type Output = Size<f32>;

    fn div(self, rhs: f32) -> Self::Output {
        Size::new(self.width / rhs, self.height / rhs)
    }
}

impl DivAssign<f32> for Size<f32> {
    fn div_assign(&mut self, rhs: f32) {
        self.width /= rhs;
        self.height /= rhs;
    }
}

impl Mul<u32> for Size<u32> {
    type Output = Size<u32>;

    fn mul(self, rhs: u32) -> Self::Output {
        Size::new(self.width * rhs, self.height * rhs)
    }
}

impl MulAssign<u32> for Size<u32> {
    fn mul_assign(&mut self, rhs: u32) {
        self.width *= rhs;
        self.height *= rhs;
    }
}

impl Div<u32> for Size<u32> {
    type Output = Size<u32>;

    fn div(self, rhs: u32) -> Self::Output {
        Size::new(self.width / rhs, self.height / rhs)
    }
}

impl DivAssign<i32> for Size<i32> {
    fn div_assign(&mut self, rhs: i32) {
        self.width /= rhs;
        self.height /= rhs;
    }
}

impl Mul<i32> for Size<i32> {
    type Output = Size<i32>;

    fn mul(self, rhs: i32) -> Self::Output {
        Size::new(self.width * rhs, self.height * rhs)
    }
}

impl MulAssign<i32> for Size<i32> {
    fn mul_assign(&mut self, rhs: i32) {
        self.width *= rhs;
        self.height *= rhs;
    }
}

impl Div<i32> for Size<i32> {
    type Output = Size<i32>;

    fn div(self, rhs: i32) -> Self::Output {
        Size::new(self.width / rhs, self.height / rhs)
    }
}

impl DivAssign<u32> for Size<u32> {
    fn div_assign(&mut self, rhs: u32) {
        self.width /= rhs;
        self.height /= rhs;
    }
}

impl Mul<Vec2> for Size<f32> {
    type Output = Size<f32>;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Size::new(self.width * rhs.x, self.height * rhs.y)
    }
}

impl MulAssign<Vec2> for Size<f32> {
    fn mul_assign(&mut self, rhs: Vec2) {
        self.width *= rhs.x;
        self.height *= rhs.y;
    }
}

impl Div<Vec2> for Size<f32> {
    type Output = Size<f32>;

    fn div(self, rhs: Vec2) -> Self::Output {
        Size::new(self.width / rhs.x, self.height / rhs.y)
    }
}

impl DivAssign<Vec2> for Size<f32> {
    fn div_assign(&mut self, rhs: Vec2) {
        self.width /= rhs.x;
        self.height /= rhs.y;
    }
}

impl Mul<IVec2> for Size<i32> {
    type Output = Size<i32>;

    fn mul(self, rhs: IVec2) -> Self::Output {
        Size::new(self.width * rhs.x, self.height * rhs.y)
    }
}

impl MulAssign<IVec2> for Size<i32> {
    fn mul_assign(&mut self, rhs: IVec2) {
        self.width *= rhs.x;
        self.height *= rhs.y;
    }
}

impl Div<IVec2> for Size<i32> {
    type Output = Size<i32>;

    fn div(self, rhs: IVec2) -> Self::Output {
        Size::new(self.width / rhs.x, self.height / rhs.y)
    }
}

impl DivAssign<IVec2> for Size<i32> {
    fn div_assign(&mut self, rhs: IVec2) {
        self.width /= rhs.x;
        self.height /= rhs.y;
    }
}

impl Mul<UVec2> for Size<u32> {
    type Output = Size<u32>;

    fn mul(self, rhs: UVec2) -> Self::Output {
        Size::new(self.width * rhs.x, self.height * rhs.y)
    }
}

impl MulAssign<UVec2> for Size<u32> {
    fn mul_assign(&mut self, rhs: UVec2) {
        self.width *= rhs.x;
        self.height *= rhs.y;
    }
}

impl Div<UVec2> for Size<u32> {
    type Output = Size<u32>;

    fn div(self, rhs: UVec2) -> Self::Output {
        Size::new(self.width / rhs.x, self.height / rhs.y)
    }
}

impl DivAssign<UVec2> for Size<u32> {
    fn div_assign(&mut self, rhs: UVec2) {
        self.width /= rhs.x;
        self.height /= rhs.y;
    }
}

impl<T> Default for Size<T>
where
    T: Num + Copy,
{
    fn default() -> Self {
        Size::new(T::zero(), T::zero())
    }
}

impl<T> ops::Add for Size<T>
where
    T: Num + Copy,
{
    type Output = Size<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Size::new(self.width + rhs.width, self.height + rhs.height)
    }
}

impl<T> ops::AddAssign for Size<T>
where
    T: Num + Copy + ops::AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.width += rhs.width;
        self.height += rhs.height;
    }
}

impl<T> ops::Sub for Size<T>
where
    T: Num + Copy,
{
    type Output = Size<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Size::new(self.width - rhs.width, self.height - rhs.height)
    }
}

impl<T> ops::SubAssign for Size<T>
where
    T: Num + Copy + ops::SubAssign,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.width -= rhs.width;
        self.height -= rhs.height;
    }
}

impl<T> ops::Mul for Size<T>
where
    T: Num + Copy,
{
    type Output = Size<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Size::new(self.width * rhs.width, self.height * rhs.height)
    }
}

impl<T> ops::MulAssign for Size<T>
where
    T: Num + Copy + ops::MulAssign,
{
    fn mul_assign(&mut self, rhs: Self) {
        self.width *= rhs.width;
        self.height *= rhs.height;
    }
}

impl<T> ops::Div for Size<T>
where
    T: Num + Copy,
{
    type Output = Size<T>;

    fn div(self, rhs: Self) -> Self::Output {
        Size::new(self.width / rhs.width, self.height / rhs.height)
    }
}

impl<T> ops::DivAssign for Size<T>
where
    T: Num + Copy + ops::DivAssign,
{
    fn div_assign(&mut self, rhs: Self) {
        self.width /= rhs.width;
        self.height /= rhs.height;
    }
}

impl Size<f32> {
    pub fn as_u32(&self) -> Size<u32> {
        Size {
            width: self.width as u32,
            height: self.height as u32,
        }
    }

    pub fn as_i32(&self) -> Size<i32> {
        Size {
            width: self.width as i32,
            height: self.height as i32,
        }
    }

    pub fn as_vec2(&self) -> Vec2 {
        vec2(self.width, self.height)
    }

    pub fn as_ivec2(&self) -> IVec2 {
        ivec2(self.width as i32, self.height as i32)
    }

    pub fn as_uvec2(&self) -> UVec2 {
        uvec2(self.width as u32, self.height as u32)
    }
}

impl Size<u32> {
    pub fn as_f32(&self) -> Size<f32> {
        Size {
            width: self.width as f32,
            height: self.height as f32,
        }
    }

    pub fn as_i32(&self) -> Size<i32> {
        Size {
            width: self.width as i32,
            height: self.height as i32,
        }
    }

    pub fn as_vec2(&self) -> Vec2 {
        vec2(self.width as f32, self.height as f32)
    }

    pub fn as_ivec2(&self) -> IVec2 {
        ivec2(self.width as i32, self.height as i32)
    }

    pub fn as_uvec2(&self) -> UVec2 {
        uvec2(self.width, self.height)
    }
}

impl Size<i32> {
    pub fn as_f32(&self) -> Size<f32> {
        Size {
            width: self.width as f32,
            height: self.height as f32,
        }
    }

    pub fn as_u32(&self) -> Size<u32> {
        Size {
            width: self.width as u32,
            height: self.height as u32,
        }
    }

    pub fn as_vec2(&self) -> Vec2 {
        vec2(self.width as f32, self.height as f32)
    }

    pub fn as_ivec2(&self) -> IVec2 {
        ivec2(self.width, self.height)
    }

    pub fn as_uvec2(&self) -> UVec2 {
        uvec2(self.width as u32, self.height as u32)
    }
}

impl<T> From<(T, T)> for Size<T>
where
    T: Num + Copy,
{
    fn from(tpl: (T, T)) -> Self {
        Size::new(tpl.0, tpl.1)
    }
}

impl<T> From<Size<T>> for (T, T)
where
    T: Num + Copy,
{
    fn from(size: Size<T>) -> Self {
        (size.width, size.height)
    }
}

impl<T> From<&[T; 2]> for Size<T>
where
    T: Num + Copy,
{
    fn from(slice: &[T; 2]) -> Self {
        Size::new(slice[0], slice[1])
    }
}

impl<T> From<&Size<T>> for [T; 2]
where
    T: Num + Copy,
{
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
    if #[cfg(feature = "glutin")] {
        impl<T> From<glutin::dpi::PhysicalSize<T>> for Size<T> where T: Num + Copy {
            fn from(size: glutin::dpi::PhysicalSize<T>) -> Self {
                Size::new(size.width, size.height)
            }
        }

        impl<T> From<Size<T>> for glutin::dpi::PhysicalSize<T> where T: Num + Copy {
            fn from(size: Size<T>) -> Self {
                glutin::dpi::PhysicalSize::new(size.width, size.height)
            }
        }
    }
}
