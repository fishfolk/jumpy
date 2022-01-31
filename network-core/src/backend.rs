use crate::{Lobby, Player, PlayerId, Result};

pub trait Backend<'a> {
    fn get_player(id: &PlayerId) -> Result<&'a Player>;
    fn get_self() -> Result<&'a Player>;
    fn is_own_id(id: &PlayerId) -> Result<bool>;
}

static mut BACKEND_INSTANCE: Option<Box<MockBackend>> = None;

/// This is used as a placeholder for when no external backend implementation is available.
/// Will be removed once we have a backend that can be freely redistributed (Steam, probably)
#[allow(dead_code)]
pub struct MockBackend {
    players: Vec<Player>,
    lobbies: Vec<Lobby>,

    own_player_id: PlayerId,
}

impl MockBackend {
    pub fn new() -> Self {
        let own_player_id = PlayerId::from("1");

        let players = vec![
            Player::new(&own_player_id, "oasf"),
            Player::new(&PlayerId::from("2"), "other player"),
        ];

        MockBackend {
            players,
            lobbies: Vec::new(),
            own_player_id,
        }
    }

    fn get_instance() -> &'static mut Self {
        unsafe { BACKEND_INSTANCE.get_or_insert_with(|| Box::new(Self::new())) }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Backend<'a> for MockBackend {
    fn get_player(id: &PlayerId) -> Result<&'a Player> {
        let instance = Self::get_instance();
        if let Some(player) = instance.players.iter().find(|&account| account.id == *id) {
            Ok(player)
        } else {
            Err("not found")
        }
    }

    fn get_self() -> Result<&'a Player> {
        let instance = Self::get_instance();
        Self::get_player(&instance.own_player_id)
    }

    fn is_own_id(id: &PlayerId) -> Result<bool> {
        let player = Self::get_self()?;
        Ok(*id == player.id)
    }
}
