use bevy_tasks::{ComputeTaskPool, TaskPool, ThreadExecutor};
use bones_utils::parking_lot::Mutex;
use piccolo::{
    AnyCallback, AnyUserData, CallbackReturn, Closure, Context, Fuel, Lua, ProtoCompileError,
    StaticClosure, StaticTable, Table, Thread, ThreadMode,
};
use send_wrapper::SendWrapper;

use crate::prelude::*;

#[macro_use]
mod freeze;
use freeze::*;

mod asset;
pub use asset::*;

/// Install the scripting plugin.
pub fn game_plugin(game: &mut Game) {
    // Initialize the lua engine resource.
    game.init_shared_resource::<LuaEngine>();
}

/// A frozen reference to the ECS [`World`].
///
// This type can be converted into lua userdata for accessing the world from lua.
#[derive(Deref, DerefMut)]
pub struct WorldRef(Frozen<Freeze![&'freeze World]>);

impl WorldRef {
    /// Convert this [`WorldRef`] into a Lua userdata.
    pub fn into_userdata<'gc>(
        self,
        ctx: Context<'gc>,
        world_metatable: Table<'gc>,
    ) -> AnyUserData<'gc> {
        let data = AnyUserData::new_static(&ctx, self);
        data.set_metatable(&ctx, Some(world_metatable));
        data
    }
}

// SOUND: WorldRef is 'static and cannot hold garbage collector references.
unsafe impl gc_arena::Collect for WorldRef {
    fn needs_trace() -> bool
    where
        Self: Sized,
    {
        false
    }

    fn trace(&self, _cc: &gc_arena::Collection) {}
}

/// Resource used to access the lua scripting engine.
#[derive(HasSchema, Clone)]
#[schema(no_default)]
pub struct LuaEngine {
    /// The thread-local task executor that is used to spawn any tasks that need access to the
    /// lua engine which can only be accessed on it's own thread.
    executor: Arc<ThreadExecutor<'static>>,
    /// The lua engine state container.
    state: Arc<SendWrapper<EngineState>>,
}

/// Internal state for [`LuaEngine`]
struct EngineState {
    /// The Lua engine.
    lua: Mutex<Lua>,
    /// Persisted lua data we need stored in Rust, such as the environment table, world
    // metatable, etc.
    data: LuaData,
}

impl Default for EngineState {
    fn default() -> Self {
        // Initialize an empty lua engine and our lua data.
        let mut lua = Lua::empty();
        let callbacks = lua.run(LuaData::new);
        let lua = Mutex::new(lua);
        Self {
            lua,
            data: callbacks,
        }
    }
}

impl Default for LuaEngine {
    /// Initialize the Lua engine.
    fn default() -> Self {
        // Make sure the compute task pool is initialized
        ComputeTaskPool::init(TaskPool::new);

        #[cfg(not(target_arch = "wasm32"))]
        let executor = {
            let (send, recv) = async_channel::bounded(1);

            // Spawn the executor task that will be used for the lua engine.
            let pool = ComputeTaskPool::get();
            pool.spawn_local(async move {
                let executor = Arc::new(ThreadExecutor::new());
                send.try_send(executor.clone()).unwrap();

                let ticker = executor.ticker().unwrap();
                loop {
                    ticker.tick().await;
                }
            })
            .detach();
            pool.with_local_executor(|local| while local.try_tick() {});

            recv.try_recv().unwrap()
        };

        #[cfg(target_arch = "wasm32")]
        let executor = Arc::new(ThreadExecutor::new());

        LuaEngine {
            executor,
            state: Arc::new(SendWrapper::new(default())),
        }
    }
}

impl LuaEngine {
    /// Access the lua engine to run code on it.
    pub fn exec<'a, F: FnOnce(&mut Lua) + Send + 'a>(&self, f: F) {
        let pool = ComputeTaskPool::get();

        // Create a new scope spawned on the lua engine thread.
        pool.scope_with_executor(false, Some(&self.executor), |scope| {
            scope.spawn_on_external(async {
                f(&mut self.state.lua.lock());
            });
        });
    }

    /// Run a lua script as a system on the given world.
    pub fn run_script_system(&self, world: &World, script: Handle<LuaScript>) {
        self.exec(|lua| {
            Frozen::<Freeze![&'freeze World]>::in_scope(world, |world| {
                let worldref = WorldRef(world);
                // Insert the world ref into the global scope.
                lua.try_run(|ctx| {
                    // Create a thread
                    let thread = Thread::new(&ctx);

                    // Fetch the env table
                    let env = ctx.state.registry.fetch(&self.state.data.env);

                    // Compile the script
                    let closure = worldref.with(|world| {
                        let asset_server = world.resource::<AssetServer>();
                        let cid = *asset_server
                            .store
                            .asset_ids
                            .get(&script.untyped())
                            .expect("Script asset not loaded");

                        let mut compiled_scripts = self.state.data.compiled_scripts.lock();
                        let closure = compiled_scripts.get(&cid);

                        Ok::<_, ProtoCompileError>(match closure {
                            Some(closure) => ctx.state.registry.fetch(closure),
                            None => {
                                let asset = asset_server.store.assets.get(&cid).unwrap();
                                let source = &asset.data.cast_ref::<LuaScript>().source;
                                let closure = Closure::load_with_env(ctx, source.as_bytes(), env)?;
                                compiled_scripts
                                    .insert(cid, ctx.state.registry.stash(&ctx, closure));

                                closure
                            }
                        })
                    })?;

                    // Insert the world ref into the global scope
                    let world = worldref.into_userdata(
                        ctx,
                        ctx.state.registry.fetch(&self.state.data.world_metatable),
                    );
                    env.set(ctx, "world", world)?;

                    // Start the thread
                    thread.start(ctx, closure.into(), ())?;

                    // Run the thread to completion
                    let mut fuel = Fuel::with_fuel(i32::MAX);
                    loop {
                        // If the thread is ready
                        if matches!(thread.mode(), ThreadMode::Normal) {
                            // Step it
                            thread.step(ctx, &mut fuel)?;
                        } else {
                            break;
                        }

                        // Handle fuel interruptions
                        if fuel.is_interrupted() {
                            break;
                        }
                    }

                    // Take the thread result and print any errors
                    let result = thread.take_return::<()>(ctx)?;
                    if let Err(e) = result {
                        error!("{e}");
                    }

                    Ok(())
                })
                .unwrap();
            });
        });
    }
}

/// Persisted lua data that we store handles to in Rust.
pub struct LuaData {
    /// The _ENV table used when running scripts.
    pub env: StaticTable,
    /// The metatable the [`WorldRef`] type.
    pub world_metatable: StaticTable,
    /// Cache of the content IDs of loaded scripts, and their compiled lua closures.
    pub compiled_scripts: Mutex<HashMap<Cid, StaticClosure>>,
}

impl LuaData {
    // Initialize the [`LuaData`] in the given context.
    pub fn new(ctx: Context) -> Self {
        Self {
            env: ctx.state.registry.stash(&ctx, Self::gen_env(ctx)),
            world_metatable: ctx.state.registry.stash(&ctx, Self::world_metatable(ctx)),
            compiled_scripts: Mutex::new(HashMap::default()),
        }
    }

    /// Build the world metatable.
    fn world_metatable(ctx: Context) -> Table {
        let metatable = Table::new(&ctx);
        metatable
            .set(
                ctx,
                "__index",
                AnyCallback::from_fn(&ctx, |_ctx, _fuel, stack| {
                    let _this = stack.pop_front();
                    let _key = stack.pop_front();

                    // TODO: Add world bindings to lua.

                    Ok(CallbackReturn::Return)
                }),
            )
            .unwrap();

        metatable
    }

    /// Generate the environment table for executing scripts under.
    fn gen_env(ctx: Context) -> Table {
        let env = Table::new(&ctx);

        macro_rules! add_log_fn {
            ($level:ident) => {
                env.set(
                    ctx,
                    stringify!($level),
                    AnyCallback::from_fn(&ctx, |_ctx, _fuel, stack| {
                        let value = stack.pop_front();

                        tracing::$level!("{value}");

                        Ok(CallbackReturn::Return)
                    }),
                )
                .unwrap();
            };
        }

        // Register logging callbacks
        add_log_fn!(trace);
        add_log_fn!(debug);
        add_log_fn!(info);
        add_log_fn!(warn);
        add_log_fn!(error);

        // Prevent creating new items in the global scope, by overrideing the __newindex metamethod
        // on the _ENV metatable.
        let metatable = Table::new(&ctx);
        metatable
            .set(
                ctx,
                "__newindex",
                AnyCallback::from_fn(&ctx, |_ctx, _fuel, _stack| {
                    Err(anyhow::format_err!(
                        "Cannot set global variables, you must use `world` \
            to persist any state across frames."
                    )
                    .into())
                }),
            )
            .unwrap();
        env.set_metatable(&ctx, Some(metatable));

        env
    }
}
