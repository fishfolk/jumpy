//! ECS component storage.

use std::{any::TypeId, sync::Arc};

use crate::prelude::*;

mod iterator;
mod typed;
mod untyped;

pub use iterator::*;
pub use typed::*;
pub use untyped::*;

/// Makes sure that the component type `T` matches the component type previously registered with
/// the same UUID.
fn validate_type_uuid_match<T: TypeUuid + 'static>(
    type_ids: &UuidMap<TypeId>,
) -> Result<(), EcsError> {
    if type_ids.get(&T::uuid()).ok_or(EcsError::NotInitialized)? != &TypeId::of::<T>() {
        Err(EcsError::TypeUuidCollision)
    } else {
        Ok(())
    }
}

/// A collection of [`ComponentStore<T>`].
///
/// [`ComponentStores`] is used to in [`World`] to store all component types that have been
/// initialized for that world.
#[derive(Default)]
pub struct ComponentStores {
    pub(crate) components: UuidMap<Arc<AtomicRefCell<UntypedComponentStore>>>,
    type_ids: UuidMap<TypeId>,
}

impl Clone for ComponentStores {
    fn clone(&self) -> Self {
        Self {
            components: self
                .components
                .iter()
                // Be sure to clone the inner data of the components, so we don't just end up with
                // new `Arc`s pointing to the same data.
                .map(|(&k, v)| (k, Arc::new((**v).clone())))
                .collect(),
            type_ids: self.type_ids.clone(),
        }
    }
}

impl ComponentStores {
    /// Initialize component storage for type `T`.
    pub fn init<T: Clone + TypeUuid + Send + Sync + 'static>(&mut self) {
        self.try_init::<T>().unwrap();
    }

    /// Initialize component storage for type `T`.
    pub fn try_init<T: Clone + TypeUuid + Send + Sync + 'static>(
        &mut self,
    ) -> Result<(), EcsError> {
        match self.components.entry(T::uuid()) {
            std::collections::hash_map::Entry::Occupied(_) => {
                validate_type_uuid_match::<T>(&self.type_ids)
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(Arc::new(AtomicRefCell::new(
                    UntypedComponentStore::for_type::<T>(),
                )));
                self.type_ids.insert(T::uuid(), TypeId::of::<T>());

                Ok(())
            }
        }
    }

    /// Get the components of a certain type
    ///
    /// # Panics
    ///
    /// Panics if the component type has not been initialized.
    pub fn get<T: Clone + TypeUuid + Send + Sync + 'static>(&self) -> AtomicComponentStore<T> {
        self.try_get::<T>().unwrap()
    }

    /// Get the components of a certain type
    pub fn try_get<T: Clone + TypeUuid + Send + Sync + 'static>(
        &self,
    ) -> Result<AtomicComponentStore<T>, EcsError> {
        validate_type_uuid_match::<T>(&self.type_ids)?;
        let untyped = self.try_get_by_uuid(T::uuid())?;

        // Safe: We've made sure that the data initialized in the untyped components matches T
        unsafe { Ok(AtomicComponentStore::from_components_unsafe(untyped)) }
    }

    /// Get the untyped component storage by the component's UUID
    ///
    /// # Panics
    ///
    /// Panics if the component type has not been initialized.
    pub fn get_by_uuid(&self, uuid: Uuid) -> Arc<AtomicRefCell<UntypedComponentStore>> {
        self.try_get_by_uuid(uuid).unwrap()
    }

    /// Get the untyped component storage by the component's UUID
    pub fn try_get_by_uuid(
        &self,
        uuid: Uuid,
    ) -> Result<Arc<AtomicRefCell<UntypedComponentStore>>, EcsError> {
        self.components
            .get(&uuid)
            .cloned()
            .ok_or(EcsError::NotInitialized)
    }
}
