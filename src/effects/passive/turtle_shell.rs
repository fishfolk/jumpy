use macroquad::{
    experimental::{
        scene::{
            Handle,
        },
        coroutines::{
            Coroutine,
            start_coroutine,
        }
    }
};

use crate::player::{
    Player,
    PlayerEventParams,
};

pub const COROUTINE_ID: &str = "turtle_shell";

pub fn coroutine (
    _instance_id: &str,
    _player_handle: Handle<Player>,
    _event_params: PlayerEventParams,
) -> Coroutine {
    let coroutine = async move {
        println!("EFFECT COROUTINE");
    };

    start_coroutine(coroutine)
}