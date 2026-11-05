use crate::prelude::*;

pub fn game_plugin(game: &mut Game) {
    game.systems.add_before_system(update_fullscreen);
}

fn update_fullscreen(game: &mut Game) {
    #[cfg(target_arch = "wasm32")]
    let _ = game;

    #[cfg(not(target_arch = "wasm32"))]
    {
        let storage = game.shared_resource_cell::<Storage>().unwrap();
        let mut storage = storage.borrow_mut().unwrap();
        let window = game.shared_resource_cell::<Window>().unwrap();
        let mut window = window.borrow_mut().unwrap();
        let keyboard = game.shared_resource::<KeyboardInputs>();

        let f11_pressed = keyboard
            .key_events
            .iter()
            .any(|x| x.key_code == Set(KeyCode::F11) && x.button_state.pressed());

        let settings = storage.get_mut::<Settings>().unwrap();
        if f11_pressed {
            settings.fullscreen = !settings.fullscreen;
        }
        window.fullscreen = settings.fullscreen;
    }
}
