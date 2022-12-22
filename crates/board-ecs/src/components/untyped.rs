use crate::prelude::*;

use aligned_vec::AVec;
use std::{
    alloc::Layout,
    ptr::{self},
};

/// Holds components of a given type indexed by `Entity`.
///
/// We do not check if the given entity is alive here, this should be done using `Entities`.
pub struct UntypedComponentStore {
    pub(crate) bitset: BitSetVec,
    pub(crate) storage: AVec<u8>,
    pub(crate) layout: Layout,
    pub(crate) max_id: usize,
    pub(crate) drop_fn: Option<unsafe extern "C" fn(*mut u8)>,
    pub(crate) clone_fn: unsafe extern "C" fn(*const u8, *mut u8),
}

impl Clone for UntypedComponentStore {
    fn clone(&self) -> Self {
        let size = self.layout.size();
        let mut new_storage = self.storage.clone();

        for i in 0..self.max_id {
            if self.bitset.bit_test(i) {
                // SAFE: constructing an UntypedComponent store is unsafe, and the user affirms that
                // clone_fn will not do anything unsound.
                //
                // - And our previous pointer is a valid pointer to component data
                // - And our new pointer is a writable pointer with the same layout
                unsafe {
                    let prev_ptr = self.storage.as_ptr().add(i * size);
                    let new_ptr = new_storage.as_mut_ptr().add(i * size);
                    (self.clone_fn)(prev_ptr, new_ptr);
                }
            }
        }

        Self {
            bitset: self.bitset.clone(),
            storage: new_storage,
            layout: self.layout,
            max_id: self.max_id,
            drop_fn: self.drop_fn,
            clone_fn: self.clone_fn,
        }
    }
}

impl Drop for UntypedComponentStore {
    fn drop(&mut self) {
        if let Some(drop_fn) = self.drop_fn {
            let size = self.layout().size();
            if size < 1 {
                return;
            }
            for i in 0..(self.storage.len() / size) {
                if self.bitset.bit_test(i) {
                    // SAFE: constructing an UntypedComponent store is unsafe, and the user affirms
                    // that clone_fn will not do anything unsound.
                    //
                    // And our pointer is valid.
                    unsafe {
                        let ptr = self.storage.as_mut_ptr().add(i * size);
                        drop_fn(ptr);
                    }
                }
            }
        }
    }
}

impl UntypedComponentStore {
    /// Create a arbitrary [`UntypedComponentStore`].
    ///
    /// In Rust, you will usually not use [`UntypedComponentStore`] and will use the statically
    /// typed [`ComponentStore<T>`] instead.
    ///
    /// # Safety
    ///
    /// The `clone_fn` and `drop_fn`, if specified, must not do anything unsound, when given valid
    /// pointers to clone or drop.
    pub unsafe fn new(
        layout: Layout,
        clone_fn: unsafe extern "C" fn(*const u8, *mut u8),
        drop_fn: Option<unsafe extern "C" fn(*mut u8)>,
    ) -> Self {
        Self {
            bitset: create_bitset(),
            // Approximation of a good default.
            storage: AVec::with_capacity(layout.align(), (BITSET_SIZE >> 4) * layout.size()),
            layout,
            max_id: 0,
            clone_fn,
            drop_fn,
        }
    }

    /// Create an [`UntypedComponentStore`] that is valid for the given type `T`.
    pub fn for_type<T: Clone + 'static>() -> Self {
        let layout = Layout::new::<T>();
        Self {
            bitset: create_bitset(),
            // Approximation of a good default.
            storage: AVec::with_capacity(layout.align(), (BITSET_SIZE >> 4) * layout.size()),
            layout,
            max_id: 0,
            clone_fn: T::raw_clone,
            drop_fn: Some(T::raw_drop),
        }
    }

    /// Get the layout of the components stored.
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Returns true if the entity already had a component of this type.
    ///
    /// If true is returned, the previous value of the pointer will be written to `data`.
    ///
    /// # Safety
    ///
    /// The data pointer must be valid for reading and writing objects with the layout that the
    /// [`UntypedComponentStore`] was created with.
    pub unsafe fn insert(&mut self, entity: Entity, data: *mut u8) -> bool {
        let size = self.layout.size();

        let index = entity.index() as usize;
        self.allocate_enough(index * size);
        let ptr = self.storage.as_mut_ptr().add(index * size);

        // If the component already exists on the entity
        if self.bitset.bit_test(entity.index() as usize) {
            // Swap the data with the data already there
            ptr::swap_nonoverlapping(ptr, data, size);

            // There was already a component of this type
            true
        } else {
            self.max_id = self.max_id.max(index + 1);
            self.allocate_enough(index * size);
            self.bitset.bit_set(index);
            ptr::swap_nonoverlapping(ptr, data, size);

            // There was not already a component of this type
            false
        }
    }

    /// Ensures that we have the vec filled at least until the `until` variable.
    ///
    /// Usually, set this to `entity.index`.
    fn allocate_enough(&mut self, until: usize) {
        if self.storage.len() <= until {
            let qty = ((until - self.storage.len()) + 1) * self.layout.size();
            for _ in 0..qty {
                self.storage.push(0);
            }
        }
    }

    /// Get a read-only pointer to the component for the given [`Entity`] if the entity has this
    /// component.
    pub fn get(&self, entity: Entity) -> Option<*const u8> {
        let index = entity.index() as usize;

        if self.bitset.bit_test(index) {
            let size = self.layout.size();
            // SAFE: we've already validated that the contents of storage is valid for type T.
            unsafe {
                let ptr = self.storage.as_ptr().add(index * size);
                Some(ptr)
            }
        } else {
            None
        }
    }

    /// Get a mutable pointer to the component for the given [`Entity`]
    pub fn get_mut(&mut self, entity: Entity) -> Option<*mut u8> {
        let index = entity.index() as usize;

        if self.bitset.bit_test(index) {
            let size = self.layout.size();
            // SAFE: we've already validated that the contents of storage is valid for type T.
            unsafe {
                let ptr = self.storage.as_mut_ptr().add(index * size);
                Some(ptr)
            }
        } else {
            None
        }
    }

    /// If there is a previous value, `true` will be returned.
    ///
    /// If `out` is set, the previous value will be written to it.
    ///
    /// # Safety
    ///
    /// If set, the `out` pointer, must not overlap the internal component storage.
    pub unsafe fn remove(&mut self, entity: Entity, out: Option<*mut u8>) -> bool {
        let index = entity.index() as usize;
        let size = self.layout.size();

        if self.bitset.bit_test(index) {
            self.bitset.bit_reset(index);

            let ptr = self.storage.as_mut_ptr().add(index * size);

            if let Some(out) = out {
                // SAFE: user asserts `out` is non-overlapping
                ptr::copy_nonoverlapping(ptr, out, size);
            } else if let Some(drop_fn) = self.drop_fn {
                // SAFE: construcing `UntypedComponentStore` asserts the soundess of the drop_fn
                //
                // And ptr is a valid pointer to the component type.
                drop_fn(ptr);
            }

            // Found previous component
            true
        } else {
            // No previous component
            false
        }
    }

    /// Iterates immutably over all components of this type.
    ///
    /// Very fast but doesn't allow joining with other component types.
    pub fn iter(&self) -> impl Iterator<Item = &[u8]> {
        let Self {
            storage: components,
            bitset,
            layout,
            ..
        } = self;

        if layout.size() > 0 {
            either::Left(
                components
                    .chunks(layout.size())
                    .enumerate()
                    .filter(move |(i, _)| bitset.bit_test(*i))
                    .map(|(_i, x)| x),
            )
        } else {
            // TODO: More tests for this iterator
            let mut idx = 0usize;
            let max_id = self.max_id;
            let iterator = std::iter::from_fn(move || loop {
                if idx >= max_id {
                    break None;
                }

                if !bitset.bit_test(idx) {
                    idx += 1;
                    continue;
                }

                idx += 1;
                break Some(&[] as &[u8]);
            });

            either::Right(iterator)
        }
    }

    /// Iterates mutably over all components of this type.
    ///
    /// Very fast but doesn't allow joining with other component types.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut [u8]> {
        let Self {
            storage,
            bitset,
            layout,
            ..
        } = self;

        if layout.size() > 0 {
            either::Left(
                storage
                    .chunks_mut(layout.size())
                    .enumerate()
                    .filter(move |(i, _)| bitset.bit_test(*i))
                    .map(|(_i, x)| x),
            )
        } else {
            let mut idx = 0usize;
            let max_id = self.max_id;
            let iterator = std::iter::from_fn(move || loop {
                if idx >= max_id {
                    break None;
                }

                if !bitset.bit_test(idx) {
                    idx += 1;
                    continue;
                }

                idx += 1;
                break Some(&mut [] as &mut [u8]);
            });

            either::Right(iterator)
        }
    }

    /// Iterates immutably over the components of this type where `bitset` indicates the indices of
    /// entities.
    ///
    /// Slower than `iter()` but allows joining between multiple component types.
    pub fn iter_with_bitset(
        &self,
        bitset: std::rc::Rc<BitSetVec>,
    ) -> UntypedComponentBitsetIterator {
        UntypedComponentBitsetIterator {
            current_id: 0,
            components: self,
            bitset,
        }
    }

    /// Iterates mutable over the components of this type where `bitset` indicates the indices of
    /// entities.
    ///
    /// Slower than `iter()` but allows joining between multiple component types.
    pub fn iter_mut_with_bitset(
        &mut self,
        bitset: std::rc::Rc<BitSetVec>,
    ) -> UntypedComponentBitsetIteratorMut {
        UntypedComponentBitsetIteratorMut {
            current_id: 0,
            components: if self.layout.size() > 0 {
                Some(self.storage.chunks_exact_mut(self.layout.size()))
            } else {
                None
            },
            components_bitset: &self.bitset,
            bitset,
            layout: self.layout,
            max_id: self.max_id,
        }
    }

    /// Returns the bitset indicating which entity indices have a component associated to them.
    ///
    /// Useful to build conditions between multiple `Components`' bitsets.
    ///
    /// For example, take two bitsets from two different `Components` types. Then,
    /// bitset1.clone().bit_and(bitset2); And finally, you can use bitset1 in `iter_with_bitset` and
    /// `iter_mut_with_bitset`. This will iterate over the components of the entity only for
    /// entities that have both components.
    pub fn bitset(&self) -> &BitSetVec {
        &self.bitset
    }
}
