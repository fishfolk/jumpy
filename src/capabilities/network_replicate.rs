use crate::nodes::Player;

use macroquad::{
    experimental::{
        coroutines::Coroutine,
        scene::{CapabilityTrait, Handle, HandleUntyped, NodeWith},
    },
    math::{Rect, Vec2},
};

#[derive(Clone, Copy)]
pub struct NetworkReplicate {
    /// Analogue of "fixed_update", but is called from network system
    /// when all the inputs are aligned and it is time to one frame advance
    /// the simulation
    pub network_update: fn(node: HandleUntyped),
}
