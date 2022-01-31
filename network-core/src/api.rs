use std::collections::HashMap;

use async_trait::async_trait;

use crate::{Lobby, LobbyId, Player, Id, Result, PlayerId};

static mut API_INSTANCE: Option<Api> = None;

pub struct Api {
    backend: Box<dyn ApiBackend>,
    own_player: Option<Player>,
}

impl Api {
    fn try_get_instance() -> Option<&'static mut Api> {
        unsafe { API_INSTANCE.as_mut() }
    }

    fn get_instance() -> &'static mut Api {
        Self::try_get_instance()
            .unwrap_or_else(|| panic!("Api::get_instance was called before Api::init"))
    }

    pub async fn init<T: 'static + ApiBackend + Default>(token: &str) -> Result<Player> {
        let backend = Box::new(T::default());

        let mut api = Api {
            backend,
            own_player: None,
        };

        let res = api.backend.init(token).await;

        api.own_player = res
            .as_ref()
            .ok()
            .cloned();

        res
    }

    pub fn is_own_id(id: &PlayerId) -> Result<bool> {
        let api = Self::get_instance();

        if let Some(player) = &api.own_player {
            Ok(player.id == *id)
        } else {
            Err("unauthenticated")
        }
    }

    pub fn get_own_player() -> Option<Player> {
        let api = Self::get_instance();

        api.own_player.clone()
    }

    pub async fn get_player(id: &PlayerId) -> Result<Player> {
        let api = Self::get_instance();

        api.backend.get_player(id).await
    }

    pub async fn list_lobbies() -> Result<&'static [Lobby]> {
        let api = Self::get_instance();

        api.backend.list_lobbies().await
    }

    pub async fn get_lobby(id: &LobbyId) -> Result<Lobby> {
        let api = Self::get_instance();

        api.backend.get_lobby(id).await
    }
}

/// This trait should be implemented by all backend implementations
#[async_trait]
pub trait ApiBackend {
    /// Init session and return authenticated `Player`
    async fn init(&mut self, token: &str) -> Result<Player>;
    /// Get `Player` with the specified `id`
    async fn get_player(&self, id: &Id) -> Result<Player>;
    /// List all available lobbies
    async fn list_lobbies(&self) -> Result<&[Lobby]>;
    /// Get `Lobby` with the specified `id`
    async fn get_lobby(&self, id: &LobbyId) -> Result<Lobby>;
}

/// This is used as a placeholder for when no external backend implementation is available.
/// Will be removed once we have a backend that can be freely redistributed (Steam, probably)
#[allow(dead_code)]
pub struct MockApiBackend {
    players: Vec<Player>,
    lobbies: Vec<Lobby>,
    sessions: HashMap<String, PlayerId>,
}

impl MockApiBackend {
    pub fn new() -> Self {
        let players = vec![
            Player::new(&PlayerId::from("1"), "oasf"),
            Player::new(&PlayerId::from("2"), "other player"),
        ];

        let mut sessions = HashMap::new();

        sessions.insert("player_one_token".to_string(), players[0].id.clone());
        sessions.insert("player_two_token".to_string(), players[1].id.clone());

        MockApiBackend {
            players,
            lobbies: Vec::new(),
            sessions,
        }
    }
}

impl Default for MockApiBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ApiBackend for MockApiBackend {
    async fn init(&mut self, token: &str) -> Result<Player> {
        if let Some(player_id) = self.sessions.get(token) {
            self.get_player(player_id).await
        } else {
            Err("Unauthenticated")
        }
    }

    async fn get_player(&self, id: &PlayerId) -> Result<Player> {
        if let Some(player) = self.players.iter().find(|&player| player.id == *id) {
            Ok(player.clone())
        } else {
            Err("not found")
        }
    }

    async fn list_lobbies(&self) -> Result<&[Lobby]> {
        Ok(self.lobbies.as_slice())
    }

    async fn get_lobby(&self, id: &LobbyId) -> Result<Lobby> {
        if let Some(lobby) = self.lobbies.iter().find(|&lobby| lobby.id == *id) {
            Ok(lobby.clone())
        } else {
            Err("not found")
        }
    }
}
