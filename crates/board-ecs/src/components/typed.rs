use std::{alloc::Layout, marker::PhantomData, sync::Arc};

use bytemuck::Pod;

use crate::prelude::*;

mod ops;
pub use ops::TypedComponentOps;

use super::{
    iterator::{ComponentBitsetIterator, ComponentBitsetIteratorMut},
    untyped::UntypedComponentStore,
};

/// A typed wrapper around [`UntypedComponentStore`].
pub struct ComponentStore<T: TypedEcsData> {
    components: UntypedComponentStore,
    ops: TypedComponentOps<T>,
}

impl<T: TypedEcsData> Default for ComponentStore<T> {
    fn default() -> Self {
        Self {
            components: UntypedComponentStore::for_type::<T>(),
            // Safe: We will only use `TypedComponentOps` for the untyped components we created
            // above, which was initialized for the same type T.
            ops: unsafe { TypedComponentOps::<T>::new() },
        }
    }
}

impl<T: TypedEcsData + Pod> ComponentStore<T> {
    /// Create a new [`ComponentStore<T>`] by wrapping an [`UntypedComponentStore`].
    ///
    /// This method is safe because `T` is required to implement [`Pod`], which means `T` is valid
    /// for _any_ bit pattern.
    ///
    /// # Panics
    ///
    /// This will panic if the layout of `T` does not match the layout of `components`.
    pub fn from_components(components: UntypedComponentStore) -> Self {
        assert_eq!(
            components.layout(),
            Layout::new::<T>(),
            "Layout mismatch creating `TypedComponents<T>`"
        );

        Self {
            components,
            // Safe:
            // - We will only use `TypedComponentOps` for the untyped components above
            // - `T` is `Pod` so it is valid for any bit pattern
            // - We validated the layout matches with the assertion above
            ops: unsafe { TypedComponentOps::<T>::new() },
        }
    }
}

impl<T: TypedEcsData> ComponentStore<T> {
    /// Create a new [`ComponentStore<T>`] by wrapping an [`UntypedComponentStore`].
    ///
    /// # Safety
    ///
    /// The data stored in `components` data must be a valid bit pattern for the given type `T`.
    ///
    /// > **Note:** If `T` implements [`Pod`] you can safely create an instance of
    /// > [`ComponentStore`] with [`from_components`][Self::from_components].
    pub unsafe fn from_components_unsafe(components: UntypedComponentStore) -> Self {
        assert_eq!(
            components.layout(),
            Layout::new::<T>(),
            "Layout mismatch creating `TypedComponents<T>`"
        );

        Self {
            components,
            ops: TypedComponentOps::<T>::new(),
        }
    }

    /// Converts to the internal, untyped [`ComponentStore`].
    pub fn into_untyped(self) -> UntypedComponentStore {
        self.components
    }

    /// Inserts a component for the given `Entity` index.
    /// Returns the previous component, if any.
    pub fn insert(&mut self, entity: Entity, component: T) -> Option<T> {
        self.ops.insert(&mut self.components, entity, component)
    }

    /// Gets an immutable reference to the component of `Entity`.
    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.ops.get(&self.components, entity)
    }

    /// Gets a mutable reference to the component of `Entity`.
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.ops.get_mut(&mut self.components, entity)
    }

    /// Removes the component of `Entity`.
    /// Returns `Some(T)` if the entity did have the component.
    /// Returns `None` if the entity did not have the component.
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        self.ops.remove(&mut self.components, entity)
    }

    /// Iterates immutably over all components of this type.
    /// Very fast but doesn't allow joining with other component types.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.ops.iter(&self.components)
    }

    /// Iterates mutably over all components of this type.
    /// Very fast but doesn't allow joining with other component types.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.ops.iter_mut(&mut self.components)
    }

    /// Iterates immutably over the components of this type where `bitset`
    /// indicates the indices of entities.
    /// Slower than `iter()` but allows joining between multiple component types.
    pub fn iter_with_bitset(&self, bitset: std::rc::Rc<BitSetVec>) -> ComponentBitsetIterator<T> {
        self.ops.iter_with_bitset(&self.components, bitset)
    }

    /// Iterates mutable over the components of this type where `bitset`
    /// indicates the indices of entities.
    /// Slower than `iter()` but allows joining between multiple component types.
    pub fn iter_mut_with_bitset(
        &mut self,
        bitset: std::rc::Rc<BitSetVec>,
    ) -> ComponentBitsetIteratorMut<T> {
        self.ops.iter_mut_with_bitset(&mut self.components, bitset)
    }

    /// Read the bitset containing the list of entites with this component type on it.
    pub fn bitset(&self) -> &BitSetVec {
        self.components.bitset()
    }
}

/// A typed, wrapper handle around [`UntypedComponentStore`] that is runtime borrow checked and can
/// be cheaply cloned. Think can think of it like an `Arc<RwLock<ComponentStore>>`.
#[derive(Clone)]
pub struct AtomicComponentStore<T: Clone + 'static> {
    components: Arc<AtomicRefCell<UntypedComponentStore>>,
    _phantom: PhantomData<T>,
}

impl<T: Clone + 'static> Default for AtomicComponentStore<T> {
    fn default() -> Self {
        Self {
            components: Arc::new(AtomicRefCell::new(UntypedComponentStore::for_type::<T>())),
            _phantom: Default::default(),
        }
    }
}

impl<T: TypedEcsData> AtomicComponentStore<T> {
    /// # Safety
    ///
    /// The [`UntypedComponentStore`] underlying data must be valid for type `T`.
    pub unsafe fn from_components_unsafe(
        components: Arc<AtomicRefCell<UntypedComponentStore>>,
    ) -> Self {
        Self {
            components,
            _phantom: PhantomData,
        }
    }

    /// Borrow the component store.
    pub fn borrow(&self) -> AtomicComponentStoreRef<T> {
        AtomicComponentStoreRef {
            components: self.components.borrow(),
            // Safe: The component type T is the same as the one we already have
            ops: unsafe { TypedComponentOps::<T>::new() },
        }
    }

    /// Mutably borrow the component store.
    pub fn borrow_mut(&self) -> AtomicComponentStoreRefMut<T> {
        AtomicComponentStoreRefMut {
            components: self.components.borrow_mut(),
            // Safe: The construction of an [`AtomicComponents`] is unsafe, and this has the same
            // invariants.
            ops: unsafe { TypedComponentOps::<T>::new() },
        }
    }
}

/// A read-only borrow of [`AtomicComponentStore`].
pub struct AtomicComponentStoreRef<'a, T: TypedEcsData> {
    components: AtomicRef<'a, UntypedComponentStore>,
    ops: TypedComponentOps<T>,
}

impl<'a, T: TypedEcsData> AtomicComponentStoreRef<'a, T> {
    /// Gets an immutable reference to the component of `Entity`.
    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.ops.get(&self.components, entity)
    }

    /// Iterates immutably over all components of this type.
    /// Very fast but doesn't allow joining with other component types.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.ops.iter(&self.components)
    }

    /// Iterates immutably over the components of this type where `bitset`
    /// indicates the indices of entities.
    /// Slower than `iter()` but allows joining between multiple component types.
    pub fn iter_with_bitset(&self, bitset: std::rc::Rc<BitSetVec>) -> ComponentBitsetIterator<T> {
        self.ops.iter_with_bitset(&self.components, bitset)
    }

    /// Read the bitset containing the list of entites with this component type on it.
    pub fn bitset(&self) -> &BitSetVec {
        self.components.bitset()
    }
}

/// A mutable borrow of [`AtomicComponentStore`].
pub struct AtomicComponentStoreRefMut<'a, T: TypedEcsData> {
    components: AtomicRefMut<'a, UntypedComponentStore>,
    ops: TypedComponentOps<T>,
}

impl<'a, T: TypedEcsData> AtomicComponentStoreRefMut<'a, T> {
    /// Inserts a component for the given [`Entity`] index.
    ///
    /// Returns the previous component, if any.
    pub fn insert(&mut self, entity: Entity, component: T) -> Option<T> {
        self.ops.insert(&mut self.components, entity, component)
    }

    /// Gets an immutable reference to the component of [`Entity`].
    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.ops.get(&self.components, entity)
    }

    /// Gets a mutable reference to the component of [`Entity`].
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.ops.get_mut(&mut self.components, entity)
    }

    /// Removes the component of [`Entity`].
    ///
    /// Returns the component that was on the entity, if any.
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        self.ops.remove(&mut self.components, entity)
    }

    /// Iterates immutably over all components of this type.
    ///
    /// Very fast but doesn't allow joining with other component types.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.ops.iter(&self.components)
    }

    /// Iterates mutably over all components of this type.
    ///
    /// Very fast but doesn't allow joining with other component types.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.ops.iter_mut(&mut self.components)
    }

    /// Iterates immutably over the components of this type where `bitset` indicates the indices of
    /// entities.
    ///
    /// Slower than `iter()` but allows joining between multiple component types.
    pub fn iter_with_bitset(&self, bitset: std::rc::Rc<BitSetVec>) -> ComponentBitsetIterator<T> {
        self.ops.iter_with_bitset(&self.components, bitset)
    }

    /// Iterates mutable over the components of this type where `bitset` indicates the indices of
    /// entities.
    ///
    /// Slower than `iter()` but allows joining between multiple component types.
    pub fn iter_mut_with_bitset(
        &mut self,
        bitset: std::rc::Rc<BitSetVec>,
    ) -> ComponentBitsetIteratorMut<T> {
        self.ops.iter_mut_with_bitset(&mut self.components, bitset)
    }

    /// Get the bitset representing which entities have this component on it.
    pub fn bitset(&self) -> &BitSetVec {
        self.components.bitset()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn create_remove_components() {
        #[derive(Debug, Clone, PartialEq, Eq, TypeUuid)]
        #[uuid = "f78f06cb-5e6e-4e28-99bd-2294c95e9348"]
        struct A(String);

        let mut entities = Entities::default();
        let e1 = entities.create();
        let e2 = entities.create();

        let mut storage = ComponentStore::<A>::default();
        storage.insert(e1, A("hello".into()));
        storage.insert(e2, A("world".into()));
        assert!(storage.get(e1).is_some());
        storage.remove(e1);
        assert!(storage.get(e1).is_none());
        assert_eq!(
            storage.iter().cloned().collect::<Vec<_>>(),
            vec![A("world".into())]
        )
    }
}
