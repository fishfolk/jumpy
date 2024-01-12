//! Global, deterministic random resource.

use crate::prelude::*;

use bones_framework::{
    prelude::bindings::EcsRef,
    scripting::lua::{
        bindings::SchemaLuaEcsRefMetatable,
        piccolo::{self as lua, Callback},
    },
};
pub use turborand::prelude::*;

pub fn plugin(session: &mut Session) {
    session.world.init_resource::<GlobalRng>();
}

/// Resource that can produce deterministic, pseudo-random numbers.
///
/// Access in a system with [`Res<GlobalRng>`].
#[derive(Clone, HasSchema, Deref, DerefMut)]
#[type_data(SchemaLuaEcsRefMetatable(lua_metatable))]
pub struct GlobalRng(AtomicRng);

impl Default for GlobalRng {
    fn default() -> Self {
        Self(AtomicRng::with_seed(7))
    }
}

fn lua_metatable(ctx: lua::Context) -> lua::Table {
    let metatable = lua::Table::new(&ctx);

    let f32_fn = ctx.registry().stash(
        &ctx,
        Callback::from_fn(&ctx, |ctx, _fuel, mut stack| {
            let this: &EcsRef = stack.consume(ctx)?;
            let mut b = this.borrow_mut();
            let global_rng = b.schema_ref_mut()?.cast_into_mut::<GlobalRng>();
            let n = global_rng.0.f32();
            stack.replace(ctx, n);
            Ok(lua::CallbackReturn::Return)
        }),
    );
    metatable
        .set(
            ctx,
            "__index",
            Callback::from_fn(&ctx, move |ctx, _fuel, mut stack| {
                let (_this, key): (lua::Value, lua::String) = stack.consume(ctx)?;

                #[allow(clippy::single_match)]
                match key.as_bytes() {
                    b"f32" => {
                        stack.push_front(ctx.registry().fetch(&f32_fn).into());
                    }
                    _ => (),
                }
                Ok(lua::CallbackReturn::Return)
            }),
        )
        .unwrap();

    metatable
}
