use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Admin,
    Standard,
}

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String, // on hachera plus tard
    pub role: Role,
}

pub struct AuthManager {
    pub users: Vec<User>,
    pub current_user: Option<User>,
}

impl AuthManager {
    pub fn new() -> Self {
        let mut users = Vec::new();
        // On crée ton compte par défaut
        users.push(User {
            username: String::from("andre"),
            password_hash: String::from("admin123"),
            role: Role::Admin,
        });
        
        AuthManager {
            users,
            current_user: None,
        }
    }

    pub fn login(&mut self, username: &str, password: &str) -> bool {
        for user in &self.users {
            if user.username.to_lowercase() == username.to_lowercase() && user.password_hash == password {
                self.current_user = Some(user.clone());
                return true;
            }
        }
        false
    }

    pub fn logout(&mut self) {
        self.current_user = None;
    }

    pub fn get_current_username(&self) -> String {
        match &self.current_user {
            Some(user) => user.username.clone(),
            None => String::from("Guest"),
        }
    }
}

lazy_static! {
    pub static ref AUTH: Mutex<AuthManager> = Mutex::new(AuthManager::new());
}