use std::borrow::BorrowMut;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;
use std::time::{Duration, Instant};

use glutin::event::{StartCause, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{Fullscreen, WindowBuilder};
use hecs::World;

use crate::math::Vec2;

use crate::audio::{apply_audio_config, stop_music};

use crate::color::colors;
use crate::event::{Event, EventHandler};
use crate::gl::create_gl_context;
use crate::input::{
    apply_input_config, is_key_pressed, is_key_released, mouse_movement, mouse_position,
    update_gamepad_context, KeyCode,
};
use crate::math::Size;
use crate::physics::{fixed_delta_time, physics_world};
use crate::prelude::{input_event_handler, DefaultEventHandler};
use crate::rendering::{clear_screen, end_frame};
use crate::window::{apply_window_config, create_window, WindowMode, DEFAULT_WINDOW_TITLE};
use crate::{Config, Result};

use crate::state::{GameState, GameStateBuilderFn};

pub struct Game<E: 'static + Debug> {
    window_title: String,
    config: Config,
    state: Rc<RefCell<dyn GameState>>,
    event_loop: Option<EventLoop<Event<E>>>,
    event_handler: Option<Box<dyn EventHandler<E>>>,
    fixed_draw_delta_time: Option<Duration>,
    last_update: Instant,
    last_draw: Instant,
    fixed_update_accumulator: f32,
}

impl<E: 'static + Debug> Game<E> {
    pub fn new<S: 'static + GameState>(state: S) -> Self {
        let config = Config::default();

        let fixed_draw_delta_time = config
            .video
            .max_fps
            .map(|max_fps| Duration::from_secs_f32(1.0 / max_fps as f32));

        Game {
            window_title: DEFAULT_WINDOW_TITLE.to_string(),
            config,
            state: Rc::new(RefCell::new(state)),
            event_loop: None,
            event_handler: None,
            fixed_draw_delta_time,
            last_update: Instant::now(),
            last_draw: Instant::now(),
            fixed_update_accumulator: 0.0,
        }
    }

    pub fn with_window_title(self, window_title: &str) -> Self {
        Game {
            window_title: window_title.to_string(),
            ..self
        }
    }

    pub fn with_config(self, config: Config) -> Self {
        let mut res = self;
        res.apply_config(&config);
        res
    }

    pub fn with_event_loop(self, event_loop: EventLoop<Event<E>>) -> Self {
        Game {
            event_loop: Some(event_loop),
            ..self
        }
    }

    pub fn with_event_handler<H: 'static + EventHandler<E>>(self, event_handler: H) -> Self {
        Game {
            event_handler: Some(Box::new(event_handler)),
            ..self
        }
    }

    pub fn change_state(&mut self, state: Rc<RefCell<dyn GameState>>) -> Result<()> {
        stop_music();

        physics_world().clear();

        let world = self.get_state().end()?;

        self.state = state;

        self.get_state().begin(world)?;

        Ok(())
    }

    pub fn apply_config(&mut self, config: &Config) {
        self.config = config.clone();

        self.fixed_draw_delta_time = config
            .video
            .max_fps
            .map(|max_fps| Duration::from_secs_f32(1.0 / max_fps as f32));

        apply_window_config(&config.window);

        apply_audio_config(&config.audio);

        apply_input_config(&config.input);
    }

    pub fn try_get_state(&mut self) -> Option<&mut (dyn GameState + 'static)> {
        Rc::get_mut(&mut self.state).map(|rc| rc.get_mut())
    }

    pub fn get_state(&mut self) -> &mut (dyn GameState + 'static) {
        self.try_get_state().unwrap()
    }

    pub async fn run(self) -> Result<()> {
        let mut game = self;

        let event_loop = game
            .event_loop
            .take()
            .unwrap_or_else(|| EventLoop::<Event<E>>::with_user_event());

        let mut event_handler = game
            .event_handler
            .take()
            .unwrap_or_else(|| Box::new(DefaultEventHandler));

        {
            let window = create_window(&game.window_title, &event_loop, &game.config)?;
            let _ = create_gl_context(window);
        }

        event_loop.run(move |event, _, control_flow| {
            event_handler.handle(&event, control_flow);

            match &event {
                glutin::event::Event::NewEvents(cause) => {
                    match cause {
                        StartCause::Init => {
                            game.get_state().begin(None);
                        }
                        _ => {}
                    }

                    update_gamepad_context()
                        .unwrap_or_else(|err| panic!("Error in gamepad context update: {}", err));
                }
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                },
                glutin::event::Event::RedrawRequested(..) => {
                    clear_screen(colors::BLACK);

                    end_frame().unwrap_or_else(|err| panic!("ERROR: Rendering error: {}", err));
                }
                glutin::event::Event::UserEvent(event) => match event {
                    Event::Custom(event) => event_handler.handle_custom(event, control_flow),
                    Event::ConfigChanged(config) => game.apply_config(config),
                    Event::StateTransition(state) => game
                        .change_state(state.clone())
                        .unwrap_or_else(|err| panic!("Error when changing state: {}", err)),
                    Event::Quit => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                },
                _ => {}
            }

            if input_event_handler(&event) {
                if is_key_pressed(KeyCode::A) {
                    println!("The 'A' key was pressed on the keyboard");
                }

                if is_key_released(KeyCode::Q) {
                    *control_flow = ControlFlow::Exit;
                }

                // query the change in mouse this update
                let mouse_diff = mouse_movement();
                if mouse_diff != Vec2::ZERO {
                    println!("The mouse diff is: {:?}", mouse_diff);
                    println!("The mouse position is: {:?}", mouse_position());
                }

                if *control_flow == ControlFlow::Exit {
                    let now = Instant::now();

                    let delta_time = now.duration_since(game.last_update);

                    game.get_state()
                        .update(delta_time.as_secs_f32())
                        .unwrap_or_else(|err| panic!("Error in game state update: {}", err));

                    game.last_update = now;

                    game.fixed_update_accumulator += delta_time.as_secs_f32();

                    let fixed_delta_time = fixed_delta_time().as_secs_f32();

                    while game.fixed_update_accumulator >= fixed_delta_time {
                        game.fixed_update_accumulator -= fixed_delta_time;

                        let integration_factor =
                            if game.fixed_update_accumulator >= fixed_delta_time {
                                1.0
                            } else {
                                game.fixed_update_accumulator / fixed_delta_time
                            };

                        game.get_state()
                            .fixed_update(fixed_delta_time, integration_factor)
                            .unwrap_or_else(|err| {
                                panic!("Error in game state fixed update: {}", err)
                            });
                    }

                    {
                        let fixed_draw_delta_time =
                            game.fixed_draw_delta_time.unwrap_or(Duration::ZERO);

                        let draw_delta_time = now.duration_since(game.last_draw);

                        if draw_delta_time >= fixed_draw_delta_time {
                            game.get_state()
                                .draw(draw_delta_time.as_secs_f32())
                                .unwrap_or_else(|err| {
                                    panic!("Error in game state fixed draw: {}", err)
                                });

                            game.last_draw = now;
                        }
                    }
                } else {
                    stop_music();

                    game.get_state().end();

                    return;
                }
            }
        });

        Ok(())
    }
}
