use hecs::{Entity, World};

pub type SystemFn = fn(&mut World);

/// This is used as a component to signify ownership
pub struct Owner(pub Entity);

/// Placeholder until we implement threading
#[derive(Default)]
pub struct SchedulerBuilder {
    steps: Vec<SystemFn>,
}

impl SchedulerBuilder {
    #[must_use]
    pub fn add_system(self, system: SystemFn) -> Self {
        let mut steps = self.steps;
        steps.push(system);

        SchedulerBuilder { steps }
    }

    #[must_use]
    pub fn add_thread_local(self, system: SystemFn) -> Self {
        let mut steps = self.steps;
        steps.push(system);

        SchedulerBuilder { steps }
    }

    pub fn build(self) -> Scheduler {
        Scheduler { steps: self.steps }
    }
}

/// Placeholder until we implement threading
pub struct Scheduler {
    steps: Vec<SystemFn>,
}

impl Scheduler {
    pub fn builder() -> SchedulerBuilder {
        SchedulerBuilder::default()
    }

    pub fn execute(&mut self, world: &mut World) {
        for f in &mut self.steps {
            f(world);
        }
    }
}
