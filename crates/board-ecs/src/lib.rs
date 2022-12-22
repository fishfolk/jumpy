//! A minimal ECS ( Entity Component System ) designed for easy snapshotting and future moddability.
//!
//! Originally forked from the [Planck ECS][planck], with heavy modifications.
//!
//! Currently under development for use in the [Jumpy] game.
//!
//! [Jumpy]: https://github.com/fishfolk/jumpy
//! [planck]: https://github.com/jojolepro/planck_ecs

#![warn(missing_docs)]

pub mod atomic {
    //! Atomic Refcell implmentation.
    //!
    //! Atomic Refcells are from the [`atomic_refcell`] crate.
    //!
    //! [`atomic_refcell`]: https://docs.rs/atomic_refcell
    pub use atomic_refcell::*;
}
pub mod bitset;
pub mod components;
pub mod dispatcher;
pub mod entities;
pub mod resources;
pub mod system;
pub mod uuid;

#[doc(hidden)]
#[macro_use]
pub mod join;

mod error;
pub use error::EcsError;

mod world;
pub use world::World;

/// The prelude.
pub mod prelude {
    pub use atomic_refcell::*;
    pub use bitset_core::BitSet;
    pub use type_uuid::TypeUuid;
    pub use uuid::Uuid;

    pub use crate::bitset::*;
    pub use crate::components::*;
    pub use crate::dispatcher::*;
    pub use crate::entities::*;
    pub use crate::error::*;
    pub use crate::join;
    pub use crate::resources::*;
    pub use crate::system::*;
    pub use crate::uuid::*;
    pub use crate::World;
    pub use crate::{EcsData, RawFns, TypedEcsData};
    pub use bevy_derive::{Deref, DerefMut};
}

/// Helper trait that is auto-implemented for anything that may be stored in the ECS's untyped
/// storage.
///
/// Examples of untyped storage are [`UntypedResources`][crate::resources::UntypedResources] and
/// [`UntypedComponentStore`][crate::components::UntypedComponentStore].
pub trait EcsData: Clone + Sync + Send + 'static {}
impl<T: Clone + Sync + Send + 'static> EcsData for T {}

/// Helper trait that is auto-implemented for anything that may be stored in the ECS's typed
/// storage.
///
/// Examples of typed storage are [`Resources<T>`][crate::resources::Resources] and
/// [`ComponentStore<T>`][crate::components::ComponentStore].
pub trait TypedEcsData: type_uuid::TypeUuid + EcsData {}
impl<T: type_uuid::TypeUuid + EcsData> TypedEcsData for T {}

/// Helper trait that is auto-implemented for all `Clone`-able types. Provides easy access to drop
/// and clone funcitons for raw pointers.
///
/// This simply serves as a convenient way to obtain a drop/clone function implementation for
/// [`UntypedResource`][crate::resources::UntypedResource] or
/// [`UntypedComponentStore`][crate::components::UntypedComponentStore].
///
/// > **Note:** This is an advanced feature that you don't need if you aren't working with some sort
/// > of scripting or otherwise untyped data access.
///
/// # Example
///
/// ```
/// # use board_ecs::prelude::*;
/// # use core::alloc::Layout;
/// let components = unsafe {
///     UntypedComponentStore::new(Layout::new::<String>(), String::raw_clone, Some(String::raw_drop));
/// };
/// ```
pub trait RawFns {
    /// Drop the value at `ptr`.
    ///
    /// # Safety
    /// Pointer must point to a valid instance of the type that this implementation is assocated
    /// wit
    unsafe extern "C" fn raw_drop(ptr: *mut u8);

    /// Clone the value at `src`, writing the new value to `dst`.
    ///
    /// # Safety
    /// Pointer must point to a valid instance of the type that this implementation is assocated
    /// with, and the destination pointer must be properly aligned and writable.
    unsafe extern "C" fn raw_clone(src: *const u8, dst: *mut u8);
}

impl<T: Clone> RawFns for T {
    unsafe extern "C" fn raw_drop(ptr: *mut u8) {
        if std::mem::needs_drop::<T>() {
            (ptr as *mut T).drop_in_place()
        }
    }

    unsafe extern "C" fn raw_clone(src: *const u8, dst: *mut u8) {
        let t = &*(src as *const T);
        let t = t.clone();
        (dst as *mut T).write(t)
    }
}
