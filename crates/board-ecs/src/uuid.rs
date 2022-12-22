//! UUID-related utilities such as UUID map and type UUIDs.
//!
//! - [`TypeUuid`] comes from the [`type_uuid`] crate
//! - [`Uuid`] comes from the [`uuid`] crate.
//!
//! [`type_uuid`]: https://docs.rs/type_uuid
//! [`uuid`]: https://docs.rs/uuid

use fxhash::FxHashMap;

pub use type_uuid::TypeUuid;
pub use uuid::Uuid;

/// Faster hash map using [`FxHashMap`] and a UUID key.
pub type UuidMap<T> = FxHashMap<Uuid, T>;

/// Extension trait for [`TypeUuid`] that adds a method returning a [`Uuid`] instead of a byte
/// sequence.
pub trait TypeUuidExt {
    /// Get the [`Uuid`] assocated to the type.
    fn uuid() -> Uuid;
}

impl<T: TypeUuid> TypeUuidExt for T {
    fn uuid() -> Uuid {
        Uuid::from_bytes(T::UUID)
    }
}
