use std::{
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    ops::DerefMut,
};

use crate::prelude::*;

/// Implements typed operations on top of a [`UntypedComponentStore`].
///
/// This is a utility used to help represent the unsafty of interpreting the [`UntypedComponentStore`]
/// as a particular type.
///
/// It is unsafe to construct a [`TypedComponentOps`] to indicate that you are taking responsibility
/// for only calling it's functions on an [`UntypedComponentStore`] that actually corresponds to the
/// type `T` that the [`TypedComponentOps`] was created for.
///
/// > **Note:** The alternative to this approach would be to make every method of this type
/// > `unsafe`, which may be a better option. It really seems like a matter of preference, but if
/// > you have an opinion, @zicklag would be happy to discuss on GitHub!
pub struct TypedComponentOps<T>(PhantomData<T>);

impl<T: Clone + 'static> TypedComponentOps<T> {
    /// # Safety
    /// Creating `TypedComponentOps` must only be used on an [`UntypedComponentStore`] where the
    /// underlying, untyped component data is valid for `T`.
    pub unsafe fn new() -> Self {
        Self(PhantomData)
    }

    /// Insert a component into the store.
    pub fn insert(
        &self,
        components: &mut UntypedComponentStore,
        entity: Entity,
        component: T,
    ) -> Option<T> {
        let mut component = ManuallyDrop::new(component);
        let ptr = component.deref_mut() as *mut T as *mut u8;

        // SAFE: constructing TypedComponentOps is unsafe, and user asserts that component storage
        // is valid for type T.
        unsafe {
            let already_existed = components.insert(entity, ptr);

            if already_existed {
                let previous_component = ManuallyDrop::take(&mut component);
                Some(previous_component)
            } else {
                None
            }
        }
    }

    /// Borrow a component in the store, if it exists for the given entity.
    pub fn get(&self, components: &UntypedComponentStore, entity: Entity) -> Option<&T> {
        // SAFE: constructing TypedComponentOps is unsafe, and user asserts that component storage
        // is valid for type T.
        components.get(entity).map(|x| unsafe { &*(x as *const T) })
    }

    /// Mutably borrow a component in the store, if it exists for the given entity.
    pub fn get_mut(
        &self,
        components: &mut UntypedComponentStore,
        entity: Entity,
    ) -> Option<&mut T> {
        components
            .get_mut(entity)
            // SAFE: constructing TypedComponentOps is unsafe, and user asserts that component storage
            // is valid for type T.
            .map(|x| unsafe { &mut *(x as *mut T) })
    }

    /// Remove a component from an entity, returning the previous component if one existed.
    pub fn remove(&self, components: &mut UntypedComponentStore, entity: Entity) -> Option<T> {
        let mut r = MaybeUninit::<T>::zeroed();
        let ptr = r.as_mut_ptr() as *mut u8;

        // SAFE: ptr doesn't overlap the component's internal storage
        let had_previous = unsafe { components.remove(entity, Some(ptr)) };
        if had_previous {
            // SAFE: According to `components.remove` the if it returns `true` then the previous
            // component has been written to the pointer ( aka. initialized ).
            unsafe {
                let r = r.assume_init();
                Some(r)
            }
        } else {
            None
        }
    }

    /// Iterate over all components in the store.
    pub fn iter<'a>(
        &'a self,
        components: &'a UntypedComponentStore,
    ) -> impl Iterator<Item = &'a T> {
        components
            .iter()
            // SAFE: constructing TypedComponentOps is unsafe, and user asserts that component
            // storage is valid for type T.
            .map(|x| unsafe { &*(x.as_ptr() as *const T) })
    }

    /// Mutably iterate over all components in the store.
    pub fn iter_mut<'a>(
        &'a self,
        components: &'a mut UntypedComponentStore,
    ) -> impl Iterator<Item = &'a mut T> {
        components
            .iter_mut()
            // SAFE: constructing TypedComponentOps is unsafe, and user asserts that component
            // storage is valid for type T.
            .map(|x| unsafe { &mut *(x.as_mut_ptr() as *mut T) })
    }

    /// Iterate over all the components in the store that match the entities in the given bitset.
    pub fn iter_with_bitset<'a>(
        &'a self,
        components: &'a UntypedComponentStore,
        bitset: std::rc::Rc<BitSetVec>,
    ) -> ComponentBitsetIterator<'a, T> {
        // SAFE: Constructing `TypedComponentOps` is unsafe and user affirms the type T is valid for
        // the underlying, untyped data.
        unsafe { ComponentBitsetIterator::new(components.iter_with_bitset(bitset)) }
    }

    /// Mutably iterate over all the components in the store that match the entities in the given
    /// bitset.
    pub fn iter_mut_with_bitset<'a>(
        &'a self,
        components: &'a mut UntypedComponentStore,
        bitset: std::rc::Rc<BitSetVec>,
    ) -> ComponentBitsetIteratorMut<T> {
        // SAFE: Constructing `TypedComponentOps` is unsafe and user affirms the type T is valid for
        // the underlying, untyped data.
        unsafe { ComponentBitsetIteratorMut::new(components.iter_mut_with_bitset(bitset)) }
    }
}
