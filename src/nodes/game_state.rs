use macroquad::{
    experimental::{
        coroutines::{start_coroutine, wait_seconds},
        scene::{self, RefMut},
    },
    prelude::*,
    window::miniquad::*,
};

use crate::gui::pause_menu;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Starting,
    Paused,
    InProgress,
}

/// Mediator of a game
/// Do pauses, manage little custscenes like game start/game finish etc
/// idk the worst name for a node, but it really is about a current state of a game
pub struct GameState {
    pub game_paused: bool,
    pub want_quit: bool,

    material: Material,
    state: State,
    label: String,
}

impl GameState {
    pub fn new() -> GameState {
        let material = load_material(
            VERTEX,
            FRAGMENT,
            MaterialParams {
                uniforms: vec![("test_color".to_string(), UniformType::Float4)],
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();

        GameState {
            material,
            game_paused: true,
            want_quit: false,
            state: State::Starting,
            label: "".to_string(),
        }
    }
}

impl scene::Node for GameState {
    fn ready(mut node: RefMut<Self>) {
        let handle = node.handle();

        node.state = State::Starting;

        start_coroutine(async move {
            {
                let mut node = scene::get_node(handle);
                node.label = "FISH".to_string();
            }
            wait_seconds(0.11).await;
            {
                let mut node = scene::get_node(handle);
                node.label = "FIGHT".to_string();
            }
            wait_seconds(0.11).await;
            {
                let mut node = scene::get_node(handle);
                node.label = "KILL".to_string();
            }
            wait_seconds(0.11).await;
            {
                let mut node = scene::get_node(handle);
                node.label = "FISH".to_string();
            }
            wait_seconds(0.11).await;

            let mut node = scene::get_node(handle);
            node.state = State::InProgress;
            node.game_paused = false;
        });
    }

    fn draw(mut node: RefMut<Self>) {
        if let State::Starting = node.state {
            push_camera_state();
            set_default_camera();

            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0., 0., 0., 0.8),
            );

            let text_size = measure_text(&node.label, None, 16, 1.);

            let pos = vec2(
                screen_width() / 2. - text_size.width / 2. * 20.,
                screen_height() / 2. + text_size.height / 2. * 20.,
            );

            gl_use_material(node.material);
            draw_text_ex(
                &node.label,
                pos.x,
                pos.y,
                TextParams {
                    font_size: 16,
                    font_scale: 20.,
                    ..Default::default()
                },
            );
            gl_use_default_material();

            pop_camera_state();
        }
        if node.state == State::Paused {
            match pause_menu::gui() {
                pause_menu::PauseResult::Quit => {
                    node.want_quit = true;
                }
                pause_menu::PauseResult::Close => {
                    node.state = State::InProgress;
                    node.game_paused = true;
                }
                pause_menu::PauseResult::Nothing => {}
            }
        }
    }

    fn update(mut node: RefMut<Self>) {
        if is_key_pressed(KeyCode::Escape) {
            if node.state == State::InProgress {
                node.state = State::Paused;
                node.game_paused = true;
            } else if node.state == State::Paused {
                node.state = State::InProgress;
                node.game_paused = false;
            }
        }
    }
}

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;

const FRAGMENT: &str = r#"#version 100
precision mediump float;

varying lowp vec2 uv;
uniform sampler2D Texture;
uniform vec4 _Time;

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453) * 2.0 - 1.0;
}

float offset_x(float blocks, vec2 uv) {
	return rand(vec2(_Time.y, floor(uv.y * blocks)));
}

float offset_y(float blocks, vec2 uv) {
	return rand(vec2(_Time.y + 1., floor(uv.x * blocks)));
}

void main() {
    vec4 res = texture2D(Texture, uv);
    
    res.a = texture2D(Texture, 
               uv + vec2(offset_x(8.0, uv) * 0.0015, offset_y(8.0, uv) * 0.0015)).a;

    gl_FragColor = res;
}"#;
