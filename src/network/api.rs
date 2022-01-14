//! This holds a dummy API for testing purposes

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

use crate::{Error, ErrorKind, Result};

static mut API_INSTANCE: Option<Box<Api>> = None;

pub struct Api {
    auth_state: AuthState,
    accounts: Vec<Account>,
    lobbies: Vec<Lobby>,

    next_account_id: AccountId,
    next_lobby_id: AccountId,
}

impl Api {
    fn new() -> Self {
        let auth_state = AuthState::Unauthenticated;

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

        Api {
            auth_state,
            accounts,
            lobbies: Vec::new(),
            next_account_id: 3,
            next_lobby_id: 1,
        }
    }

    pub fn get_instance() -> &'static mut Self {
        unsafe { API_INSTANCE.get_or_insert_with(|| Box::new(Self::new())) }
    }

    pub fn is_authenticated(&self) -> Result<bool> {
        let res = self.auth_state.is_authenticated();

        Ok(res)
    }

    pub fn sign_up(&mut self, display_name: &str, email: &str, password: &str) -> Result<Account> {
        self.validate_display_name(display_name)?;

        self.validate_email(email)?;

        self.validate_password(password)?;

        let password_hash = self.hash_password(password)?;

        let account = Account {
            id: self.next_account_id,
            display_name: display_name.to_string(),
            email: email.to_string(),
            password_hash: Some(password_hash),
        };

        self.next_account_id += 1;

        self.create_account(account)
    }

    fn generate_token(&self, account_id: AccountId) -> Result<AuthToken> {
        let bytes = format!("test_token for account id {}", account_id).into_bytes();
        let token = AuthToken(bytes);

        Ok(token)
    }

    fn validate_display_name(&self, _display_name: &str) -> Result<()> {
        Ok(())
    }

    fn validate_email(&self, email: &str) -> Result<()> {
        let is_existing = self.list_accounts().iter().any(|acc| acc.email == *email);

        if is_existing {
            return Err(Error::new_const(
                ErrorKind::Api,
                &"Account sign-up: The specified email address is already in use",
            ));
        }

        Ok(())
    }

    fn validate_password(&self, _password: &str) -> Result<()> {
        Ok(())
    }

    fn hash_password(&self, password: &str) -> Result<String> {
        Ok(password.to_string())
    }

    pub fn sign_in(&mut self, email: &str, password: &str) -> Result<Account> {
        let password_hash = self.hash_password(password)?;

        let res = self.list_accounts().iter().find(|&acc| {
            acc.email == *email
                && acc.password_hash.is_some()
                && *acc.password_hash.as_ref().unwrap() == password_hash
        });

        if let Some(account) = res {
            let token = self.generate_token(account.id).unwrap();

            self.auth_state = AuthState::Authenticated {
                account_id: account.id,
                token,
            };

            self.get_own_account()
        } else {
            Err(Error::new_const(ErrorKind::Api, &"Unauthenticated"))
        }
    }

    pub fn sign_out(&mut self) -> Result<()> {
        self.auth_state = AuthState::Unauthenticated;

        Ok(())
    }

    fn list_accounts(&self) -> &[Account] {
        self.accounts.as_ref()
    }

    fn create_account(&mut self, account: Account) -> Result<Account> {
        if account.password_hash.is_none() {
            return Err(Error::new_const(ErrorKind::Api, &"No password hash specified in account parameters. This is required when creating a new account"));
        }

        self.accounts.push(account.clone());

        Ok(account.remove_secrets())
    }

    pub fn get_account(&self, id: AccountId) -> Result<Account> {
        if !self.auth_state.is_authenticated() {
            return Err(Error::new_const(ErrorKind::Api, &"Unauthenticated"));
        }

        let res = self.accounts.iter().find(|account| account.id == id);

        if let Some(account) = res.cloned() {
            Ok(account.remove_secrets())
        } else {
            Err(Error::new_const(ErrorKind::Api, &"Not found"))
        }
    }

    pub fn get_own_account(&self) -> Result<Account> {
        if let AuthState::Authenticated { account_id, .. } = &self.auth_state {
            self.get_account(*account_id)
        } else {
            Err(Error::new_const(ErrorKind::Api, &"Unauthenticated"))
        }
    }

    pub fn get_own_address(&self) -> Result<IpAddr> {
        let ip = Ipv4Addr::new(127, 0, 0, 1).into();
        Ok(ip)
    }

    pub fn is_own_id(&self, id: AccountId) -> Result<bool> {
        let account = self.get_own_account()?;
        Ok(id == account.id)
    }

    pub fn create_lobby(&mut self) -> Result<Lobby> {
        if !self.auth_state.is_authenticated() {
            return Err(Error::new_const(ErrorKind::Api, &"Unauthenticated"));
        }

        let account = self.get_own_account().unwrap();

        let lobby = Lobby {
            id: self.next_lobby_id,
            host: account.id,
            clients: Vec::new(),
        };

        self.next_lobby_id += 1;

        self.lobbies.push(lobby.clone());

        Ok(lobby)
    }
}

pub type AccountId = u64;

struct AuthToken(Vec<u8>);

#[allow(dead_code)]
enum AuthState {
    Unauthenticated,
    Authenticated {
        account_id: AccountId,
        token: AuthToken,
    },
}

impl AuthState {
    pub fn is_authenticated(&self) -> bool {
        if let Self::Authenticated { .. } = self {
            return true;
        }

        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountId,
    pub display_name: String,
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
}

impl Account {
    pub fn remove_secrets(self) -> Self {
        Account {
            password_hash: None,
            ..self
        }
    }
}

pub type LobbyId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lobby {
    pub id: LobbyId,
    pub host: AccountId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clients: Vec<AccountId>,
}
