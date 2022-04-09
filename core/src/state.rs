use cfg_if::cfg_if;
use fishsticks::Button;
use hecs::World;
use std::borrow::BorrowMut;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ecs::{DrawSystemFn, FixedUpdateSystemFn, UpdateSystemFn};
use crate::events::GameEvent;
use crate::input::{is_gamepad_button_pressed, is_key_pressed, update_gamepad_context, KeyCode};
use crate::math::*;
use crate::Result;

#[cfg(feature = "macroquad-backend")]
use crate::gui::{Menu, MenuResult};

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

    fn draw(&mut self) -> Result<()> {
        Ok(())
    }

    fn end(&mut self) -> Result<Option<World>> {
        Ok(None)
    }
}

pub type GameStateConstructorFn<P> = fn(&mut World, Option<&mut P>) -> Result<()>;

pub type GameStateDestructorFn<P> = fn(World, Option<P>) -> Result<Option<World>>;

#[cfg(feature = "macroquad-backend")]
const MENU_FLAG: &str = "__SHOW_MENU__";

pub struct DefaultGameState<P> {
    id: String,
    world: Option<World>,
    constructor: GameStateConstructorFn<P>,
    updates: Vec<UpdateSystemFn>,
    fixed_updates: Vec<FixedUpdateSystemFn>,
    draws: Vec<DrawSystemFn>,
    destructor: GameStateDestructorFn<P>,
    flags: HashMap<String, ()>,
    payload: Option<P>,
    #[cfg(feature = "macroquad-backend")]
    menu: Option<Menu>,
}

impl<P> DefaultGameState<P> {
    pub fn builder(id: &str) -> GameStateBuilder<P> {
        GameStateBuilder::new(id)
    }

    pub fn set_flag(&mut self, flag: &str) {
        self.flags.insert(flag.to_string(), ());
    }

    /// Returns `true` if the flag was actually set before unset
    pub fn unset_flag(&mut self, flag: &str) -> bool {
        self.flags.remove(flag).is_some()
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains_key(flag)
    }

    /// Returns the new state of the flag
    pub fn toggle_flag(&mut self, flag: &str) -> bool {
        if self.flags.get(flag).is_some() {
            self.flags.remove(flag);
            true
        } else {
            self.flags.insert(flag.to_string(), ());
            false
        }
    }
}

impl<P> GameState for DefaultGameState<P> {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn begin(&mut self, world: Option<World>) -> Result<()> {
        self.world = world.unwrap_or_else(World::new).into();

        (self.constructor)(self.world.as_mut().unwrap(), self.payload.as_mut())
    }

    fn update(&mut self, delta_time: f32) -> Result<()> {
        #[cfg(feature = "macroquad-backend")]
        if self.menu.is_some()
            && (is_key_pressed(KeyCode::Escape) || is_gamepad_button_pressed(Button::Start))
        {
            self.toggle_flag(MENU_FLAG);
        }

        for f in &mut self.updates {
            f(self.world.as_mut().unwrap(), delta_time)?;
        }

        Ok(())
    }

    fn fixed_update(&mut self, delta_time: f32, integration_factor: f32) -> Result<()> {
        for f in &mut self.fixed_updates {
            f(self.world.as_mut().unwrap(), delta_time, integration_factor)?;
        }

        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        for f in &mut self.draws {
            f(self.world.as_mut().unwrap())?;
        }

        #[cfg(feature = "macroquad-backend")]
        if self.has_flag(MENU_FLAG) {
            if let Some(menu) = &mut self.menu {
                use macroquad::ui::root_ui;

                if let Some(res) = menu.ui(&mut *root_ui()) {}
            }
        }

        Ok(())
    }

    fn end(&mut self) -> Result<Option<World>> {
        let world = self.world.take().unwrap();

        (self.destructor)(world, self.payload.take())?;

        Ok(None)
    }
}

pub struct GameStateBuilder<P> {
    id: String,
    constructor: GameStateConstructorFn<P>,
    updates: Vec<UpdateSystemFn>,
    fixed_updates: Vec<FixedUpdateSystemFn>,
    draws: Vec<DrawSystemFn>,
    destructor: GameStateDestructorFn<P>,
    flags: HashMap<String, ()>,
    payload: Option<P>,
    #[cfg(feature = "macroquad-backend")]
    menu: Option<Menu>,
}

impl<P> GameStateBuilder<P> {
    pub fn new(id: &str) -> Self {
        GameStateBuilder {
            id: id.to_string(),
            constructor: |_: &mut World, _: Option<&mut P>| Ok(()),
            updates: Vec::new(),
            fixed_updates: Vec::new(),
            draws: Vec::new(),
            destructor: |world: World, _: Option<P>| Ok(Some(world)),
            flags: HashMap::new(),
            payload: None,
            #[cfg(feature = "macroquad-backend")]
            menu: None,
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

    pub fn with_constructor(self, f: GameStateConstructorFn<P>) -> Self {
        GameStateBuilder {
            constructor: f,
            ..self
        }
    }

    pub fn with_destructor(self, f: GameStateDestructorFn<P>) -> Self {
        GameStateBuilder {
            destructor: f,
            ..self
        }
    }

    pub fn with_flag(self, flag: &str) -> Self {
        let mut flags = self.flags;
        flags.insert(flag.to_string(), ());

        GameStateBuilder { flags, ..self }
    }

    pub fn with_payload(self, payload: P) -> Self {
        GameStateBuilder {
            payload: Some(payload),
            ..self
        }
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

    pub fn build(&self) -> DefaultGameState<P>
    where
        P: Clone,
    {
        DefaultGameState {
            id: self.id.clone(),
            world: None,
            constructor: self.constructor,
            updates: self.updates.clone(),
            fixed_updates: self.fixed_updates.clone(),
            draws: self.draws.clone(),
            destructor: self.destructor,
            flags: self.flags.clone(),
            payload: self.payload.clone(),
            #[cfg(feature = "macroquad-backend")]
            menu: self.menu.clone(),
        }
    }

    pub fn build_to(self) -> DefaultGameState<P> {
        DefaultGameState {
            id: self.id,
            world: None,
            constructor: self.constructor,
            updates: self.updates,
            fixed_updates: self.fixed_updates,
            draws: self.draws,
            destructor: self.destructor,
            flags: self.flags,
            payload: self.payload,
            #[cfg(feature = "macroquad-backend")]
            menu: self.menu,
        }
    }
}
