//! [`System`] executor for [`World`]s.

use crate::prelude::*;

/// A builder that accumulates systems to be inserted into a [`Dispatcher`].
#[derive(Default)]
pub struct DispatcherBuilder {
    /// The current systems in this builder
    pub systems: Vec<System>,
}

impl DispatcherBuilder {
    /// Creates a new `DispatcherBuilder`.
    pub fn new() -> Self {
        Self {
            systems: Vec::default(),
        }
    }

    /// Adds a function implementing `IntoSystem` to the system pool.
    #[allow(clippy::should_implement_trait)]
    pub fn add<R, F: IntoSystem<R>>(mut self, into_system: F) -> Self {
        self.systems.push(into_system.system());
        self
    }

    /// Builds a `Dispatcher` from the accumulated set of `System`.
    /// This preserves the order from the inserted systems.
    pub fn build(self, world: &mut World) -> Dispatcher {
        for sys in self.systems.iter() {
            (sys.initialize)(world);
        }

        // TODO: Right now we only care about single-threaded execution, so everything goes into the
        // same stage.
        let stages: Vec<Vec<System>> = vec![self.systems];

        Dispatcher { stages }
    }
}

/// A dispatcher is used to execute a collection of `System` in order.
///
/// Eventually, parallel execution could be implemented.
pub struct Dispatcher {
    stages: Vec<Vec<System>>,
}
impl Dispatcher {
    /// Create a builder for [`Dispatcher`].
    pub fn builder() -> DispatcherBuilder {
        DispatcherBuilder::default()
    }

    /// Returns an iterator of all stages.
    ///
    /// Until parallel execution is implemented, there will only ever be one stage.
    ///
    /// This is not needed for regular use, but can be useful for debugging or for implementing
    /// custom executors.
    pub fn iter_stages(&self) -> impl Iterator<Item = &Vec<System>> {
        self.stages.iter()
    }

    /// Runs the systems one after the other, one at a time.
    pub fn run_seq(&mut self, world: &World) -> Result<(), EcsError> {
        for stage in &mut self.stages {
            let errors = stage
                .iter_mut()
                .map(|s| s.run(world))
                .flat_map(|r| r.err())
                .collect::<Vec<_>>();
            if !errors.is_empty() {
                return Err(EcsError::DispatcherExecutionFailed(errors));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use type_uuid::TypeUuid;

    #[test]
    fn simple_dispatcher() {
        #[derive(Default, TypeUuid, Clone)]
        #[uuid = "89062d2d-221a-4178-ab98-d22c3a94103f"]
        pub struct A;
        let mut world = World::default();
        let sys = (|_comps: Res<A>| Ok(())).system();
        let mut dispatch = DispatcherBuilder::new().add(sys).build(&mut world);
        dispatch.run_seq(&world).unwrap();
        dispatch.run_seq(&world).unwrap();
        dispatch.run_seq(&world).unwrap();
        assert!(world.resources.try_get::<A>().is_some());
    }

    #[test]
    fn generic_simple_dispatcher() {
        #[derive(Default, TypeUuid, Clone)]
        #[uuid = "89062d2d-221a-4178-ab98-d22c3a94103f"]
        pub struct A;
        let mut world = World::default();
        let mut dispatch = DispatcherBuilder::new()
            .add(|_: Comp<A>| ())
            .add((|_: CompMut<A>| ()).system())
            .build(&mut world);
        dispatch.run_seq(&world).unwrap();
        dispatch.run_seq(&world).unwrap();
        dispatch.run_seq(&world).unwrap();
        assert!(world.components.try_get::<A>().is_ok());
    }
}
