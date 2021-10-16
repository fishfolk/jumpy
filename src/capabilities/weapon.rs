use crate::nodes::Player;

use macroquad::{
    experimental::{
        coroutines::Coroutine,
        scene::{CapabilityTrait, Handle, HandleUntyped, NodeWith},
    },
    math::{Rect, Vec2},
};

/// Anything that can be picked up from a level
/// and than being used by a fish
#[derive(Clone, Copy, CapabilityTrait)]
pub struct Weapon {
    /// A world space weapon rectangle
    /// Collider is used for picking up weapons
    /// Things like sword hitbox are defined by `shoot` logic
    /// Therefore collider should never be affected by
    /// weapon's `facing` (left of right)
    /// It is just a Rectangle where .point() is top-left point of the weapon
    /// and .size() is a colliders dimensions
    pub collider: fn(node: HandleUntyped) -> Rect,
    /// Usually weapons are either rest on location
    /// or are being handled by a fish
    /// "mount" is being called to attach the weapon to a fish
    /// in other words - mount should move a weapon to a mount point on a fish
    pub mount: fn(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool, inverted: bool),
    pub is_thrown: fn(node: HandleUntyped) -> bool,
    pub pick_up: fn(node: HandleUntyped, owner: Handle<Player>),
    pub throw: fn(node: HandleUntyped, force: bool),
    pub shoot: fn(node: HandleUntyped, player: Handle<Player>) -> Coroutine,
}
