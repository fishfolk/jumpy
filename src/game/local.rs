//! Node that emulates network for local play
//! This is temporary solution, in the future "LocalNetwork" node should
//! be able to mix local/remote players
//!
//! But, right now, with fixed-delay networking
//! it is nice to run completely local, no-delay game

use macroquad::experimental::scene::{self, Handle, Node, NodeWith, RefMut};

use crate::{
    capabilities::NetworkReplicate,
    collect_input, exit_to_main_menu,
    gui::{self, GameMenuResult},
    quit_to_desktop, GameInputScheme, Player,
};

pub struct LocalGame {
    player1_input: GameInputScheme,
    player1: Handle<Player>,
    player2_input: GameInputScheme,
    player2: Handle<Player>,

    is_menu_open: bool,
}

impl LocalGame {
    pub fn new(
        players_input: Vec<GameInputScheme>,
        player1: Handle<Player>,
        player2: Handle<Player>,
    ) -> LocalGame {
        LocalGame {
            player1,
            player2,
            player1_input: players_input[0],
            player2_input: players_input[1],
            is_menu_open: false,
        }
    }
}

impl Node for LocalGame {
    fn fixed_update(mut node: RefMut<Self>) {
        scene::get_node(node.player1).apply_input(collect_input(node.player1_input));
        scene::get_node(node.player2).apply_input(collect_input(node.player2_input));

        #[cfg(debug_assertions)]
        if macroquad::input::is_key_pressed(macroquad::prelude::KeyCode::U) {
            crate::debug::toggle_debug_draw();
        }

        if macroquad::input::is_key_pressed(macroquad::prelude::KeyCode::Escape) {
            node.is_menu_open = !node.is_menu_open;
        }

        if !node.is_menu_open {
            for NodeWith { node, capability } in scene::find_nodes_with::<NetworkReplicate>() {
                (capability.network_update)(node);
            }
        }
    }

    fn draw(mut node: RefMut<Self>) {
        if node.is_menu_open {
            if let Some(res) = gui::show_game_menu() {
                match res {
                    GameMenuResult::MainMenu => exit_to_main_menu(),
                    GameMenuResult::Quit => quit_to_desktop(),
                    GameMenuResult::Cancel => node.is_menu_open = false,
                }
            }
        }
    }
}
