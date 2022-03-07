use std::borrow::BorrowMut;
use hecs::World;

use crate::ecs::{DrawSystemFn, FixedUpdateSystemFn, UpdateSystemFn};

pub trait GameState {
    fn begin(&mut self, _world: Option<World>) {}

    fn update(&mut self, _delta_time: f32) {}

    fn fixed_update(&mut self, _delta_time: f32, _integration_factor: f32) {}

    fn draw(&mut self) {}

    fn end(&mut self) -> Option<World> {
        None
    }
}

pub type GameStateConstructorFn = fn(&mut World);

pub type GameStateDestructorFn = fn(World) -> Option<World>;

pub struct DefaultGameState {
    world: Option<World>,
    constructor: GameStateConstructorFn,
    updates: Vec<UpdateSystemFn>,
    fixed_updates: Vec<FixedUpdateSystemFn>,
    draws: Vec<DrawSystemFn>,
    destructor: GameStateDestructorFn,
}

impl DefaultGameState {
    pub fn builder() -> GameStateBuilder {
        GameStateBuilder::new()
    }
}

impl GameState for DefaultGameState {
    fn begin(&mut self, world: Option<World>) {
        self.world = world
            .unwrap_or_else(World::new)
            .into();

        (self.constructor)(self.world.as_mut().unwrap());
    }

    fn update(&mut self, delta_time: f32) {
        for f in &mut self.updates {
            f(self.world.as_mut().unwrap(), delta_time)
        }
    }

    fn fixed_update(&mut self, delta_time: f32, integration_factor: f32) {
        for f in &mut self.fixed_updates {
            f(self.world.as_mut().unwrap(), delta_time, integration_factor)
        }
    }

    fn draw(&mut self) {
        for f in &mut self.draws {
            f(self.world.as_mut().unwrap())
        }
    }

    fn end(&mut self) -> Option<World> {
        let world = self.world
            .take()
            .unwrap();

        (self.destructor)(world)
    }
}

pub struct GameStateBuilder {
    constructor: GameStateConstructorFn,
    updates: Vec<UpdateSystemFn>,
    fixed_updates: Vec<FixedUpdateSystemFn>,
    draws: Vec<DrawSystemFn>,
    destructor: GameStateDestructorFn,
}

impl GameStateBuilder {
    pub fn new() -> Self {
        GameStateBuilder {
            constructor: |_: &mut World| {},
            updates: Vec::new(),
            fixed_updates: Vec::new(),
            draws: Vec::new(),
            destructor: |world: World| { Some(world) },
        }
    }

    pub fn add_update(&mut self, f: UpdateSystemFn) -> &mut Self {
        self.updates.push(f);
        self
    }

    pub fn with_update(self, f: UpdateSystemFn) -> Self {
        let mut builder = self;
        builder.add_update(f);
        builder
    }

    pub fn add_fixed_update(&mut self, f: FixedUpdateSystemFn) -> &mut Self {
        self.fixed_updates.push(f);
        self
    }

    pub fn with_fixed_update(self, f: FixedUpdateSystemFn) -> Self {
        let mut builder = self;
        builder.add_fixed_update(f);
        builder
    }

    pub fn add_draw(&mut self, f: DrawSystemFn) -> &mut Self {
        self.draws.push(f);
        self
    }

    pub fn with_draw(self, f: DrawSystemFn) -> Self {
        let mut builder = self;
        builder.add_draw(f);
        builder
    }

    pub fn with_constructor(self, f: GameStateConstructorFn) -> Self {
        GameStateBuilder {
            constructor: f,
            ..self
        }
    }

    pub fn with_destructor(self, f: GameStateDestructorFn) -> Self {
        GameStateBuilder {
            destructor: f,
            ..self
        }
    }

    pub fn build(self) -> DefaultGameState {
        DefaultGameState {
            world: None,
            constructor: self.constructor,
            updates: self.updates,
            fixed_updates: self.fixed_updates,
            draws: self.draws,
            destructor: self.destructor,
        }
    }
}

impl Default for GameStateBuilder {
    fn default() -> Self {
        GameStateBuilder::new()
    }
}