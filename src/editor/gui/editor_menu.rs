use crate::editor::EditorContext;
use macroquad::{
    prelude::*,
    ui::{hash, Ui},
};

use crate::gui::{Menu, MenuEntry, MenuResult};

const MENU_WIDTH: f32 = 200.0;

pub const EDITOR_MENU_RESULT_NEW: usize = 0;
pub const EDITOR_MENU_RESULT_OPEN: usize = 1;
pub const EDITOR_MENU_RESULT_SAVE: usize = 2;
pub const EDITOR_MENU_RESULT_SAVE_AS: usize = 3;
pub const EDITOR_MENU_RESULT_MAIN_MENU: usize = 4;
pub const EDITOR_MENU_RESULT_QUIT: usize = 5;

static mut EDITOR_MENU_INSTANCE: Option<Menu> = None;

pub fn open_editor_menu(ctx: &EditorContext) {
    unsafe {
        if EDITOR_MENU_INSTANCE.is_none() {
            let menu = Menu::new(
                hash!(),
                MENU_WIDTH,
                &[
                    MenuEntry {
                        index: EDITOR_MENU_RESULT_NEW,
                        title: "New".to_string(),
                        ..Default::default()
                    },
                    MenuEntry {
                        index: EDITOR_MENU_RESULT_OPEN,
                        title: "Open".to_string(),
                        ..Default::default()
                    },
                    MenuEntry {
                        index: EDITOR_MENU_RESULT_SAVE,
                        title: "Save".to_string(),
                        is_pulled_down: false,
                        is_disabled: !ctx.is_user_map,
                    },
                    MenuEntry {
                        index: EDITOR_MENU_RESULT_SAVE_AS,
                        title: "Save As".to_string(),
                        ..Default::default()
                    },
                    MenuEntry {
                        index: EDITOR_MENU_RESULT_MAIN_MENU,
                        title: "Main Menu".to_string(),
                        is_pulled_down: false,
                        is_disabled: false,
                    },
                    MenuEntry {
                        index: EDITOR_MENU_RESULT_QUIT,
                        title: "Quit".to_string(),
                        is_pulled_down: false,
                        is_disabled: false,
                    },
                ],
            );

            EDITOR_MENU_INSTANCE = Some(menu);
        }
    }
}

pub fn close_editor_menu() {
    unsafe { EDITOR_MENU_INSTANCE = None };
}

pub fn draw_editor_menu(ui: &mut Ui, ctx: &EditorContext) -> Option<MenuResult> {
    let menu = unsafe {
        if EDITOR_MENU_INSTANCE.is_none() {
            open_editor_menu(ctx);
        }

        EDITOR_MENU_INSTANCE.as_mut().unwrap()
    };

    let res = menu.ui(ui);

    if res.is_some() {
        close_editor_menu();
    }

    res
}

pub fn is_editor_menu_open() -> bool {
    unsafe { EDITOR_MENU_INSTANCE.is_some() }
}

/// Toggle editor menu and return state after toggle
pub fn toggle_editor_menu(ctx: &EditorContext) -> bool {
    if is_editor_menu_open() {
        close_editor_menu();
        false
    } else {
        open_editor_menu(ctx);
        true
    }
}
