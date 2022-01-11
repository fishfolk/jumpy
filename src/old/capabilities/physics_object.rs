use macroquad::{
    experimental::scene::{CapabilityTrait, HandleUntyped, NodeWith},
    math::Rect,
};

/// Anything that can be thrown, pushed, dragged etc
#[derive(Clone, Copy, CapabilityTrait)]
pub struct PhysicsObject {
    /// Indicates if the object wants to interact
    /// For example, picked up weapons do not really want to interact with
    /// sproingers, but they still have a collider
    pub active: fn(node: HandleUntyped) -> bool,
    /// Get an object rectangle
    pub collider: fn(node: HandleUntyped) -> Rect,

    pub set_speed_x: fn(node: HandleUntyped, speed: f32),
    pub set_speed_y: fn(node: HandleUntyped, speed: f32),
}
