use crate::{Account, AccountId, Lobby, Result};

pub trait Backend {
    fn get_account(id: AccountId) -> Result<Account>;
    fn get_own_account() -> Result<Account>;
    fn is_own_id(id: AccountId) -> Result<bool>;
}

static mut BACKEND_INSTANCE: Option<Box<MockBackend>> = None;

/// This is used as a placeholder for when no external backend implementation is available.
/// Will be removed once we have a backend that can be freely redistributed (Steam, probably)
#[allow(dead_code)]
pub struct MockBackend {
    accounts: Vec<Account>,
    lobbies: Vec<Lobby>,

    own_account_id: AccountId,
}

impl MockBackend {
    pub fn new() -> Self {
        let accounts = vec![
            Account {
                id: 1,
                display_name: "oasf".to_string(),
                email: "oasf@polygo.no".to_string(),
                password_hash: Some("secretsauce".to_string()),
            },
            Account {
                id: 2,
                display_name: "other_user".to_string(),
                email: "other@polygo.no".to_string(),
                password_hash: Some("secretsauce".to_string()),
            },
        ];

        MockBackend {
            accounts,
            lobbies: Vec::new(),
            own_account_id: 1,
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

impl Backend for MockBackend {
    fn get_account(id: AccountId) -> Result<Account> {
        let instance = Self::get_instance();
        let res = instance.accounts.iter().find(|account| account.id == id);

        if let Some(account) = res.cloned() {
            Ok(account.remove_secrets())
        } else {
            Err("Not found")
        }
    }

    fn get_own_account() -> Result<Account> {
        let instance = Self::get_instance();
        Self::get_account(instance.own_account_id)
    }

    fn is_own_id(id: AccountId) -> Result<bool> {
        let account = Self::get_own_account()?;
        Ok(id == account.id)
    }
}
