use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("flappy_jellyfish"))]
#[repr(C)]
pub struct FlappyJellyfishMeta {
    pub atlas: Handle<Atlas>,
    pub start_frame: u32,
    pub end_frame: u32,
    pub fps: f32,
}

impl FlappyJellyfishMeta {
    pub fn frames(&self) -> SVec<u32> {
        (self.start_frame..self.end_frame).collect()
    }
}

#[derive(Clone, Debug, HasSchema, Default)]
pub struct FlappyJellyfish {
    pub owner: Entity,
}
