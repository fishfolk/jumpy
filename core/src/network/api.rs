use std::collections::HashMap;

use async_trait::async_trait;

use crate::Result;

use super::{Id, Lobby, NetworkEvent, Player, RequestStatus};

static mut API_INSTANCE: Option<Api> = None;

pub struct Api {
    backend: Box<dyn ApiBackend>,
}

impl Api {
    fn try_get_instance() -> Option<&'static mut Api> {
        unsafe { API_INSTANCE.as_mut() }
    }

    fn get_instance() -> &'static mut Api {
        Self::try_get_instance()
            .unwrap_or_else(|| panic!("Api::get_instance was called before Api::init"))
    }

    pub async fn init<T: 'static + ApiBackend + Default>(token: &str) -> Result<()> {
        let backend = Box::new(T::default());

        let mut api = Api { backend };

        api.backend.init(token).await
    }

    pub async fn close() -> Result<()> {
        let api = Self::get_instance();

        api.backend.close().await
    }

    pub async fn get_player(id: &Id) -> Result<Player> {
        let api = Self::get_instance();

        api.backend.get_player(id).await
    }

    pub async fn get_lobby(id: &Id) -> Result<Lobby> {
        let api = Self::get_instance();

        api.backend.get_lobby(id).await
    }

    pub fn poll_events() -> Vec<NetworkEvent> {
        let api = Self::get_instance();

        api.backend.poll_events()
    }
}

/// This trait should be implemented by all backend implementations
#[async_trait]
pub trait ApiBackend {
    /// Init backend
    async fn init(&mut self, token: &str) -> Result<()>;
    /// Close connection
    async fn close(&mut self) -> Result<()>;
    /// Get `Player` with the specified `id`
    async fn get_player(&mut self, id: &Id) -> Result<Player>;
    /// Get `Lobby` with the specified `id`
    async fn get_lobby(&mut self, id: &Id) -> Result<Lobby>;
    /// Get the next event in the event queue
    fn poll_events(&mut self) -> Vec<NetworkEvent>;
}

/// This is used as a placeholder for when no external backend implementation is available.
/// Will be removed once we have a backend that can be freely redistributed (Steam, probably)
#[allow(dead_code)]
pub struct MockApiBackend {
    players: Vec<Player>,
    lobbies: Vec<Lobby>,
    sessions: HashMap<String, Id>,
}

impl MockApiBackend {
    pub fn new() -> Self {
        let players = vec![
            Player::new(&Id::from("1"), "Player One"),
            Player::new(&Id::from("2"), "Player Two"),
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
    async fn init(&mut self, _token: &str) -> Result<()> {
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn get_player(&mut self, id: &Id) -> Result<Player> {
        if let Some(player) = self.players.iter().find(|&player| player.id == *id) {
            Ok(player.clone())
        } else {
            Err(RequestStatus::NotFound.into())
        }
    }

    async fn get_lobby(&mut self, id: &Id) -> Result<Lobby> {
        if let Some(lobby) = self.lobbies.iter().find(|&lobby| lobby.id == *id) {
            Ok(lobby.clone())
        } else {
            Err(RequestStatus::NotFound.into())
        }
    }

    fn poll_events(&mut self) -> Vec<NetworkEvent> {
        Vec::new()
    }
}
