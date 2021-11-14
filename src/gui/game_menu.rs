use macroquad::{
    prelude::*,
    ui::{hash, Ui},
};

use super::{Menu, MenuEntry, MenuResult};

const MENU_WIDTH: f32 = 200.0;

pub const GAME_MENU_RESULT_MAIN_MENU: usize = 0;
pub const GAME_MENU_RESULT_QUIT: usize = 1;

static mut GAME_MENU_INSTANCE: Option<Menu> = None;

pub fn open_game_menu() {
    unsafe {
        if GAME_MENU_INSTANCE.is_none() {
            let menu = Menu::new(
                hash!(),
                MENU_WIDTH,
                &[
                    MenuEntry {
                        index: GAME_MENU_RESULT_MAIN_MENU,
                        title: "Main Menu".to_string(),
                        is_pulled_down: false,
                    },
                    MenuEntry {
                        index: GAME_MENU_RESULT_QUIT,
                        title: "Quit".to_string(),
                        is_pulled_down: false,
                    },
                ],
            );

            GAME_MENU_INSTANCE = Some(menu);
        }
    }
}

pub fn close_game_menu() {
    unsafe { GAME_MENU_INSTANCE = None };
}

pub fn draw_game_menu(ui: &mut Ui) -> Option<MenuResult> {
    let menu = unsafe {
        if GAME_MENU_INSTANCE.is_none() {
            open_game_menu();
        }

        GAME_MENU_INSTANCE.as_mut().unwrap()
    };

    let res = menu.ui(ui);

    if res.is_some() {
        close_game_menu();
    }

    res
}

pub fn is_game_menu_open() -> bool {
    unsafe { GAME_MENU_INSTANCE.is_some() }
}

/// Toggle game menu and return state after toggle
pub fn toggle_game_menu() -> bool {
    if is_game_menu_open() {
        close_game_menu();
        false
    } else {
        open_game_menu();
        true
    }
}
