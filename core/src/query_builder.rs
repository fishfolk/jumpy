use std::marker::PhantomData;

use hecs::{Query, QueryBorrow, QueryMut, World};

pub struct BuildStart();
pub struct Builder1<T>(PhantomData<T>);
pub struct Builder2<A, B>(PhantomData<(A, B)>);
pub struct Builder3<A, B, C>(PhantomData<(A, B, C)>);
pub struct Builder4<A, B, C, D>(PhantomData<(A, B, C, D)>);
pub struct Builder5<A, B, C, D, E>(PhantomData<(A, B, C, D, E)>);
pub struct Builder6<A, B, C, D, E, F>(PhantomData<(A, B, C, D, E, F)>);
pub struct Builder7<A, B, C, D, E, F, G>(PhantomData<(A, B, C, D, E, F, G)>);
pub struct Builder8<A, B, C, D, E, F, G, H>(PhantomData<(A, B, C, D, E, F, G, H)>);
pub struct Builder9<A, B, C, D, E, F, G, H, I>(PhantomData<(A, B, C, D, E, F, G, H, I)>);
pub struct Builder10<A, B, C, D, E, F, G, H, I, J>(PhantomData<(A, B, C, D, E, F, G, H, I, J)>);
pub struct Builder11<A, B, C, D, E, F, G, H, I, J, K>(
    PhantomData<(A, B, C, D, E, F, G, H, I, J, K)>,
);
pub struct Builder12<A, B, C, D, E, F, G, H, I, J, K, L>(
    PhantomData<(A, B, C, D, E, F, G, H, I, J, K, L)>,
);
pub struct Builder13<A, B, C, D, E, F, G, H, I, J, K, L, M>(
    PhantomData<(A, B, C, D, E, F, G, H, I, J, K, L, M)>,
);
pub struct Builder14<A, B, C, D, E, F, G, H, I, J, K, L, M, N>(
    PhantomData<(A, B, C, D, E, F, G, H, I, J, K, L, M, N)>,
);
pub struct Builder15<A, B, C, D, E, F, G, H, I, J, K, L, M, N, P>(
    PhantomData<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, P)>,
);

impl Default for BuildStart {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildStart {
    pub fn new() -> Self {
        Self()
    }
    pub fn query(world: &World) -> QueryBorrow<'_, ()> {
        world.query()
    }
    pub fn query_mut(world: &mut World) -> QueryMut<'_, ()> {
        world.query_mut()
    }
    pub fn with<T>(self) -> Builder1<T> {
        Builder1(PhantomData)
    }
}

impl<T: Query> Builder1<T> {
    pub fn query(self, world: &World) -> QueryBorrow<'_, T> {
        world.query()
    }
    pub fn query_mut(self, world: &mut World) -> QueryMut<'_, T> {
        world.query_mut()
    }
    pub fn with<N>(self) -> Builder2<T, N>
    where
        (T, N): Query,
    {
        Builder2(PhantomData)
    }
}

impl<A, B> Builder2<A, B>
where
    (A, B): Query,
{
    pub fn query(self, world: &World) -> QueryBorrow<'_, (A, B)> {
        world.query()
    }
    pub fn query_mut(self, world: &mut World) -> QueryMut<'_, (A, B)> {
        println!("QUERY!");
        world.query_mut()
    }
    pub fn with<N>(self) -> Builder3<A, B, N>
    where
        (A, B, N): Query,
    {
        Builder3(PhantomData)
    }
}
