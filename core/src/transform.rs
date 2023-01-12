use crate::internal_prelude::*;

#[derive(Clone, Copy, Debug, TypeUlid)]
#[ulid = "01GNFQWJWWXJJEXZQEDQJWZQWP"]
#[repr(C)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Vec2::ONE,
        }
    }
}
