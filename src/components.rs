//! Optional small pieces of logic, usefull to build weapons and items
//! If weapon is too unique and is not similar to anything - it is totally fine
//! to just skip this.
//! However, most weapons are throwable and use laws of fish-physics.
//! Peeking "Throwable" and "PhysicsBody" component will help making such a weapon.
//!
//! Some weapons may have a similar to, say, "PhysicsBody" behavior, but slightly different
//! There are two ways to achieve this - use the component, but somehow post-process
//! results from component's calls
//! Or just copy-paste the whole component code into a weapon and modify it. This is fine!

mod animation_player;
mod armed_grenade;
mod bullet;
mod erupted_item;
mod gunlike_animation;
mod physics_body;
mod sprite;
mod throwable_item;

pub use armed_grenade::ArmedGrenade;
pub use bullet::Bullet;
pub use erupted_item::EruptedItem;
pub use gunlike_animation::GunlikeAnimation;
pub use physics_body::PhysicsBody;
pub use throwable_item::ThrowableItem;

pub use sprite::{Sprite, SpriteParams};

pub use animation_player::{AnimationParams, AnimationPlayer};
