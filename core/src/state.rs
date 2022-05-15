use cfg_if::cfg_if;
use hecs::World;

use std::any::{Any, TypeId};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::process::Output;
use std::rc::Rc;
use std::slice::{Iter, IterMut};

use serde::{Deserialize, Serialize};

use crate::camera::camera_position;
use crate::drawables::{debug_draw_drawables, draw_drawables, update_animated_sprites};

use crate::ecs::{DrawFn, FixedUpdateFn, UpdateFn};
use crate::event::Event;
use crate::input::{
    is_gamepad_button_pressed, is_key_pressed, update_gamepad_context, Button, KeyCode,
};
use crate::math::*;
use crate::result::Result;

#[cfg(feature = "macroquad-backend")]
use crate::gui::{Menu, MenuResult};
use crate::map::{draw_map, Map};
use crate::particles::{draw_particles, update_particle_emitters};
use crate::physics::{
    debug_draw_physics_bodies, debug_draw_rigid_bodies, fixed_update_physics_bodies,
    fixed_update_rigid_bodies,
};
use crate::timer::update_timers;

pub trait GameState {
    fn id(&self) -> String;

    fn begin(&mut self, _world: Option<World>) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, _delta_time: f32) -> Result<()> {
        Ok(())
    }

    fn fixed_update(&mut self, _delta_time: f32, _integration_factor: f32) -> Result<()> {
        Ok(())
    }

    fn draw(&mut self, _delta_time: f32) -> Result<()> {
        Ok(())
    }

    fn end(&mut self) -> Result<Option<World>> {
        Ok(None)
    }
}

pub type GameStateConstructorFn<P: Clone> =
    fn(Option<&mut World>, Option<&Map>, Option<&P>) -> Result<()>;

pub type GameStateDestructorFn<P: Clone> =
    fn(Option<&mut World>, Option<&Map>, Option<&P>) -> Result<()>;

pub type GameStateBuilderFn = fn() -> Box<dyn GameState>;

pub struct DefaultGameState<P: Clone> {
    id: String,
    world: Option<World>,
    constructor: GameStateConstructorFn<P>,
    updates: Vec<UpdateFn>,
    fixed_updates: Vec<FixedUpdateFn>,
    draws: Vec<DrawFn>,
    destructor: GameStateDestructorFn<P>,
    map: Option<Map>,
    payload: Option<P>,
    #[cfg(feature = "macroquad-backend")]
    menu: Option<Menu>,
    should_draw_menu: bool,
    is_active: bool,
}

impl<P: Clone> DefaultGameState<P> {
    pub fn builder(id: &str) -> DefaultGameStateBuilder<P> {
        DefaultGameStateBuilder::new(id)
    }
}

impl<P: Clone> GameState for DefaultGameState<P> {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn begin(&mut self, world: Option<World>) -> Result<()> {
        if !self.is_active {
            if let Some(world) = world {
                self.world = Some(world);
            }

            (self.constructor)(
                self.world.as_mut(),
                self.map.as_ref(),
                self.payload.as_ref(),
            )?;

            if let Some(world) = self.world.as_mut() {
                if let Some(map) = self.map.take() {
                    let entity = world.spawn(());
                    world.insert_one(entity, map).unwrap();
                }
            }

            self.is_active = true;
        }

        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> Result<()> {
        if self.is_active {
            #[cfg(feature = "macroquad")]
            if self.menu.is_some()
                && (is_key_pressed(KeyCode::Escape)
                    || is_gamepad_button_pressed(None, Button::Start))
            {
                self.should_draw_menu = !self.should_draw_menu;
            }

            for f in &mut self.updates {
                f(self.world.as_mut().unwrap(), delta_time)?;
            }
        }

        Ok(())
    }

    fn fixed_update(&mut self, delta_time: f32, integration_factor: f32) -> Result<()> {
        if self.is_active {
            for f in &mut self.fixed_updates {
                f(self.world.as_mut().unwrap(), delta_time, integration_factor)?;
            }
        }

        Ok(())
    }

    fn draw(&mut self, delta_time: f32) -> Result<()> {
        if self.is_active {
            for f in &mut self.draws {
                f(self.world.as_mut().unwrap(), delta_time)?;
            }

            #[cfg(feature = "macroquad-backend")]
            if self.should_draw_menu {
                if let Some(menu) = &mut self.menu {
                    use macroquad::ui::root_ui;

                    if let Some(res) = menu.ui(&mut *root_ui()) {}
                }
            }
        }

        Ok(())
    }

    fn end(&mut self) -> Result<Option<World>> {
        if self.is_active {
            (self.destructor)(
                self.world.as_mut(),
                self.map.as_ref(),
                self.payload.as_ref(),
            )?;

            self.is_active = false;
        }

        Ok(self.world.take())
    }
}

#[derive(Clone)]
pub struct DefaultGameStateBuilder<P: Clone> {
    id: String,
    constructor: GameStateConstructorFn<P>,
    updates: Vec<UpdateFn>,
    fixed_updates: Vec<FixedUpdateFn>,
    draws: Vec<DrawFn>,
    destructor: GameStateDestructorFn<P>,
    map: Option<Map>,
    has_world: bool,
    payload: Option<P>,
    #[cfg(feature = "macroquad-backend")]
    menu: Option<Menu>,
}

impl<P: Clone> DefaultGameStateBuilder<P> {
    pub fn new(id: &str) -> Self {
        DefaultGameStateBuilder {
            id: id.to_string(),
            constructor: |_: Option<&mut World>, _: Option<&Map>, _: Option<&P>| Ok(()),
            updates: Vec::new(),
            fixed_updates: Vec::new(),
            draws: Vec::new(),
            destructor: |_: Option<&mut World>, _: Option<&Map>, _: Option<&P>| Ok(()),
            map: None,
            has_world: false,
            payload: None,
            #[cfg(feature = "macroquad-backend")]
            menu: None,
        }
    }

    pub fn add_default_systems(&mut self) -> &mut Self {
        self.add_update(update_timers)
            .add_update(update_animated_sprites)
            .add_update(update_particle_emitters);

        self.add_fixed_update(fixed_update_physics_bodies)
            .add_fixed_update(fixed_update_rigid_bodies);

        self.add_draw(draw_map)
            .add_draw(draw_drawables)
            .add_draw(draw_particles);

        #[cfg(debug_assertions)]
        self.add_draw(debug_draw_drawables)
            .add_draw(debug_draw_physics_bodies)
            .add_draw(debug_draw_rigid_bodies);

        self
    }

    pub fn with_default_systems(self) -> Self {
        let mut builder = self;
        builder.add_default_systems();
        builder
    }

    pub fn add_update(&mut self, f: UpdateFn) -> &mut Self {
        self.updates.push(f);
        self
    }

    pub fn with_update(self, f: UpdateFn) -> Self {
        let mut builder = self;
        builder.add_update(f);
        builder
    }

    pub fn add_fixed_update(&mut self, f: FixedUpdateFn) -> &mut Self {
        self.fixed_updates.push(f);
        self
    }

    pub fn with_fixed_update(self, f: FixedUpdateFn) -> Self {
        let mut builder = self;
        builder.add_fixed_update(f);
        builder
    }

    pub fn add_draw(&mut self, f: DrawFn) -> &mut Self {
        self.draws.push(f);
        self
    }

    pub fn with_draw(self, f: DrawFn) -> Self {
        let mut builder = self;
        builder.add_draw(f);
        builder
    }

    pub fn with_constructor(self, f: GameStateConstructorFn<P>) -> Self {
        DefaultGameStateBuilder {
            constructor: f,
            ..self
        }
    }

    pub fn with_destructor(self, f: GameStateDestructorFn<P>) -> Self {
        DefaultGameStateBuilder {
            destructor: f,
            ..self
        }
    }

    pub fn with_map(self, map: Map) -> Self {
        let mut moved = self;
        moved.add_map(map);
        moved
    }

    pub fn add_map(&mut self, map: Map) -> &mut Self {
        self.map = Some(map);
        self
    }

    pub fn with_empty_world(self) -> Self {
        let mut moved = self;
        moved.has_world = true;
        moved
    }

    pub fn add_empty_world(&mut self) -> &mut Self {
        self.has_world = true;
        self
    }

    pub fn with_payload(self, payload: P) -> Self {
        let mut moved = self;
        moved.add_payload(payload);
        moved
    }

    pub fn add_payload(&mut self, payload: P) -> &mut Self {
        self.payload = Some(payload);
        self
    }

    #[cfg(feature = "macroquad-backend")]
    pub fn with_menu(self, menu: Menu) -> Self {
        let mut moved = self;
        moved.add_menu(menu);
        moved
    }

    #[cfg(feature = "macroquad-backend")]
    pub fn add_menu(&mut self, menu: Menu) -> &mut Self {
        self.menu = Some(menu);
        self
    }

    pub fn build(self) -> DefaultGameState<P> {
        let world = if self.has_world {
            Some(World::new())
        } else {
            None
        };

        DefaultGameState {
            id: self.id.clone(),
            world,
            constructor: self.constructor,
            updates: self.updates.clone(),
            fixed_updates: self.fixed_updates.clone(),
            draws: self.draws.clone(),
            destructor: self.destructor,
            map: self.map.clone(),
            payload: self.payload.clone(),
            #[cfg(feature = "macroquad-backend")]
            menu: self.menu.clone(),
            should_draw_menu: false,
            is_active: false,
        }
    }
}
