/// Re-export of itertools `izip` which is used in the `join` macro.
#[doc(hidden)]
pub use itertools::izip;

#[doc(hidden)]
#[macro_export]
macro_rules! gen_bitset {
    ($bitset:ident;) => {};
    ($bitset:ident; &mut $st:ident $($tail:tt)*) => {
        *std::rc::Rc::get_mut(&mut $bitset).unwrap() = $st.bitset().clone();
        $crate::gen_bitset!($bitset; $($tail)*);
    };
    ($bitset:ident; &$st:ident $($tail:tt)*) => {
        *std::rc::Rc::get_mut(&mut $bitset).unwrap() = $st.bitset().clone();
        $crate::gen_bitset!($bitset; $($tail)*);
    };
    ($bitset:ident; !&$st:ident $($tail:tt)*) => {
        let mut cloned = $st.bitset().clone();
        cloned.bit_not();
        *std::rc::Rc::get_mut(&mut $bitset).unwrap() = cloned;
        gen_bitset!($bitset; $($tail)*);
    };
    ($bitset:ident; && &mut $st:ident $($tail:tt)*) => {
        std::rc::Rc::get_mut(&mut $bitset).unwrap().bit_and($st.bitset());
        $crate::gen_bitset!($bitset; $($tail)*);
    };
    ($bitset:ident; && &$st:ident $($tail:tt)*) => {
        std::rc::Rc::get_mut(&mut $bitset).unwrap().bit_and($st.bitset());
        $crate::gen_bitset!($bitset; $($tail)*);
    };
    ($bitset:ident; && !&$st:ident $($tail:tt)*) => {
        std::rc::Rc::get_mut(&mut $bitset).unwrap().bit_andnot($st.bitset());
        $crate::gen_bitset!($bitset; $($tail)*);
    };
    ($bitset:ident; || &mut $st:ident $($tail:tt)*) => {
        std::rc::Rc::get_mut(&mut $bitset).unwrap().bit_or($st.bitset());
        $crate::gen_bitset!($bitset; $($tail)*);
    };
    ($bitset:ident; || &$st:ident $($tail:tt)*) => {
        std::rc::Rc::get_mut(&mut $bitset).unwrap().bit_or($st.bitset());
        $crate::gen_bitset!($bitset; $($tail)*);
    };
    ($bitset:ident; || !&$st:ident $($tail:tt)*) => {
        std::rc::Rc::get_mut(&mut $bitset).unwrap().bit_or($st.bitset().clone().bit_not());
        $crate::gen_bitset!($bitset; $($tail)*);
    };
    // scopes
    /*($bitset:ident; && ($($inner:tt)*) $($tail:tt)*) => {
        $bitset.bit_and({gen_bitset!($bitset; $($inner:tt)*); $bitset});
        gen_bitset!($bitset; $($tail)*);
    };
    ($bitset:ident; && !($($inner:tt)*) $($tail:tt)*) => {
        $bitset.bit_andnot({gen_bitset!($bitset; $($inner:tt)*); $bitset});
        gen_bitset!($bitset; $($tail)*);
    };
    ($bitset:ident; || ($($inner:tt)*) $($tail:tt)*) => {
        $bitset.bit_or({gen_bitset!($bitset; $($inner:tt)*); $bitset});
        gen_bitset!($bitset; $($tail)*);
    };
    ($bitset:ident; || !($($inner:tt)*) $($tail:tt)*) => {
        $bitset.bit_or({gen_bitset!($bitset; $($inner:tt)*); $bitset}.bit_not());
        gen_bitset!($bitset; $($tail)*);
    };*/
}

#[doc(hidden)]
#[macro_export]
macro_rules! iter_bitset {
    ($bitset:ident ; $(,)?$($idents:block),* ;) => {$crate::join::izip!($($idents),*)};
    ($bitset:ident ; $(,)?$($idents:block),* ; &mut $st:ident $($tail:tt)*) => {
        $crate::iter_bitset!($bitset; $($idents),* , {$st.iter_mut_with_bitset($bitset.clone())} ; $($tail)*)
    };
    ($bitset:ident ; $(,)?$($idents:block),* ; &$st:ident $($tail:tt)*) => {
        $crate::iter_bitset!($bitset; $($idents),* , {$st.iter_with_bitset($bitset.clone())} ; $($tail)*)
    };
    ($bitset:ident ; $(,)?$($idents:block),* ; !&$st:ident $($tail:tt)*) => {
        $crate::iter_bitset!($bitset; $($idents),* , {$st.iter_with_bitset($bitset.clone())} ; $($tail)*)
    };
    ($bitset:ident ; $(,)?$($idents:block),* ; && $($tail:tt)*) => {
        $crate::iter_bitset!($bitset; $($idents),* ; $($tail)*)
    };
    ($bitset:ident ; $(,)?$($idents:block),* ; || $($tail:tt)*) => {
        $crate::iter_bitset!($bitset; $($idents),* ; $($tail)*)
    };
    /*($bitset:ident; $(,)?$($idents:block),*; ($($tail:tt)*)) => {
        iter_bitset!($bitset; $($idents)*; $($tail)*)
    };
    ($bitset:ident; $(,)?$($idents:block),*; !($($tail:tt)*)) => {
        iter_bitset!($bitset; $($idents)*; $($tail)*)
    };*/
}

/// The join macro makes it very easy to iterate over multiple components of the same `Entity` at
/// once.
///
/// There are two ways to use this macro: With a single component and with multiple.
///
/// When joining over a single component, simply provide the name of the `Components<T>` instance as
/// an immutable or mutable reference. An iterator over the components will be returned. The
/// iterator will be of type `&T` or `&mut T` elements.
///
/// Joining over multiple components offers a complete syntax to decide which components should or
/// should not be matched. Here is an example:
/// ```rust,ignore
/// let iter = join!(&storage1 && &mut storage2 || &mut storage3 && !&storage4);
/// ```
///
/// Here, we first provide a bitset. This is due to a limitation with rust macros where creating
/// variables inside of the macro and returning them is not allowed.
///
/// Then, we tell join to join over all entities that have:
/// - A component in storage1
/// - A component in either storage2 or storage3
/// - No component in storage4
///
/// We also specify that storage2 and storage3 should be accessed mutably.
///
/// Finally, we can iterate:
/// ```rust,ignore
/// iter.for_each(|(component1, mut component2, mut component3, _)| {});
/// ```
/// This iterator will be of type `(Option<&T1>, Option<&mut T2>, ...)`.
#[macro_export]
macro_rules! join {
    (&$st:ident) => {
        $st.iter()
    };
    (&mut $st:ident) => {
        $st.iter_mut()
    };
    ($($complex:tt)*) => {
        {
            use $crate::bitset::BitSet;
            // TODO find a way to avoid having this first vec allocation.
            let mut bitset = std::rc::Rc::new(vec![]);
            $crate::gen_bitset!(bitset; $($complex)*);
            let iter = $crate::iter_bitset!(bitset ; ; $($complex)*);
            iter
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn join_components() {
        #[derive(TypeUuid, Clone)]
        #[uuid = "f92ee887-6d2f-4fea-9c6c-d47754d4873a"]
        struct A;
        #[derive(TypeUuid, Clone)]
        #[uuid = "8fe1f4cb-be2d-4be1-8dbc-2e55da7b101e"]
        struct B;
        let comp1 = ComponentStore::<A>::default();
        let comp2 = ComponentStore::<B>::default();
        join!(&comp1 && &comp2).for_each(|_| {});
    }

    #[test]
    fn join_components_atomic() {
        #[derive(TypeUuid, Clone)]
        #[uuid = "f92ee887-6d2f-4fea-9c6c-d47754d4873a"]
        struct A;
        #[derive(TypeUuid, Clone)]
        #[uuid = "8fe1f4cb-be2d-4be1-8dbc-2e55da7b101e"]
        struct B;
        let comp1 = AtomicComponentStore::<A>::default();
        let comp1 = comp1.borrow_mut();
        let comp2 = AtomicComponentStore::<B>::default();
        let comp2 = comp2.borrow_mut();
        join!(&comp1 && &comp2).for_each(|_| {});
    }

    #[test]
    fn complex_join() {
        #[derive(TypeUuid, Clone)]
        #[uuid = "f92ee887-6d2f-4fea-9c6c-d47754d4873a"]
        struct A;
        #[derive(TypeUuid, Clone)]
        #[uuid = "8fe1f4cb-be2d-4be1-8dbc-2e55da7b101e"]
        struct B;
        #[derive(TypeUuid, Clone)]
        #[uuid = "f227d3fe-d525-48cc-8aa9-c1c43e69b4f9"]
        struct C;
        let mut storage1 = ComponentStore::<A>::default();
        let storage2 = ComponentStore::<B>::default();
        let storage3 = ComponentStore::<C>::default();
        let mut count = 0;
        join!(&mut storage1 && &storage2 || !&storage3).for_each(|(_a, _b, _c)| {
            count += 1;
        });
        assert_eq!(count, 0);
    }

    #[test]
    fn start_with_not() {
        #[derive(TypeUuid, Clone)]
        #[uuid = "f92ee887-6d2f-4fea-9c6c-d47754d4873a"]
        struct A;
        #[derive(TypeUuid, Clone)]
        #[uuid = "8fe1f4cb-be2d-4be1-8dbc-2e55da7b101e"]
        struct B;
        let comp1 = ComponentStore::<A>::default();
        let comp2 = ComponentStore::<B>::default();
        join!(!&comp1 && &comp2).for_each(|_| {});
    }
}
