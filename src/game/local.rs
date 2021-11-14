//! Node that emulates network for local play
//! This is temporary solution, in the future "LocalNetwork" node should
//! be able to mix local/remote players
//!
//! But, right now, with fixed-delay networking
//! it is nice to run completely local, no-delay game

use fishsticks::{Button, GamepadContext};

use macroquad::{
    experimental::{
        scene::{self, Handle, Node, NodeWith, RefMut},
        collections::storage,
    },
    ui::root_ui,
};

use crate::{
    capabilities::NetworkReplicate,
    collect_input,
    exit_to_main_menu,
    gui::{self, GAME_MENU_RESULT_MAIN_MENU, GAME_MENU_RESULT_QUIT},
    quit_to_desktop,
    GameInputScheme, Player,
    is_gamepad_btn_pressed,
};

pub struct LocalGame {
    player1_input: GameInputScheme,
    player1: Handle<Player>,
    player2_input: GameInputScheme,
    player2: Handle<Player>,
}

impl LocalGame {
    pub fn new(
        player_input: Vec<GameInputScheme>,
        player1: Handle<Player>,
        player2: Handle<Player>,
    ) -> LocalGame {
        LocalGame {
            player1,
            player2,
            player1_input: player_input[0],
            player2_input: player_input[1],
        }
    }
}

impl Node for LocalGame {
    fn fixed_update(mut node: RefMut<Self>) {
        #[cfg(debug_assertions)]
        if macroquad::input::is_key_pressed(macroquad::prelude::KeyCode::U) {
            crate::debug::toggle_debug_draw();
        }

        {
            let gamepad_context = storage::get::<GamepadContext>();
            if macroquad::input::is_key_pressed(macroquad::prelude::KeyCode::Escape)
                || is_gamepad_btn_pressed(&gamepad_context, Button::Start) {
                gui::toggle_game_menu();
            }
        }

        if !gui::is_game_menu_open() {
            scene::get_node(node.player1).apply_input(collect_input(node.player1_input));
            scene::get_node(node.player2).apply_input(collect_input(node.player2_input));

            for NodeWith { node, capability } in scene::find_nodes_with::<NetworkReplicate>() {
                (capability.network_update)(node);
            }
        }
    }

    fn draw(mut node: RefMut<Self>) {
        if gui::is_game_menu_open() {
            if let Some(res) = gui::draw_game_menu(&mut *root_ui()) {
                match res.into_usize() {
                    GAME_MENU_RESULT_MAIN_MENU => exit_to_main_menu(),
                    GAME_MENU_RESULT_QUIT => quit_to_desktop(),
                    _ => {},
                }
            }
        }
    }
}
