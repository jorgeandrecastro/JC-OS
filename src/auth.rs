use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Admin,
    Standard,
}

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    pub uid: u32,
}

pub struct AuthManager {
    pub users: Vec<User>,
    pub current_user: Option<User>,
    next_uid: u32, // New: to track the next available ID
}

impl AuthManager {
    pub fn new() -> Self {
        let mut users = Vec::new();
        // Default admin account for Andre
        users.push(User {
            username: String::from("andre"),
            password_hash: String::from("admin123"),
            role: Role::Admin,
            uid: 0,
        });
        
        AuthManager {
            users,
            current_user: None,
            next_uid: 1000, // Standard users start at 1000
        }
    }

    /// ADDITION: Dynamically creates a new user
    pub fn add_user(&mut self, username: &str, password: &str) -> Result<u32, &'static str> {
        // Check if the username already exists
        if self.users.iter().any(|u| u.username == username) {
            return Err("This user already exists!!");
        }

        let new_uid = self.next_uid;
        self.users.push(User {
            username: String::from(username),
            password_hash: String::from(password),
            role: Role::Standard,
            uid: new_uid,
        });

        self.next_uid += 1; // Prepare the ID for the next user
        Ok(new_uid)
    }

    pub fn login(&mut self, username: &str, password: &str) -> bool {
        for user in &self.users {
            if user.username == username && user.password_hash == password {
                self.current_user = Some(user.clone());
                return true;
            }
        }
        false
    }

    #[allow(dead_code)]
    pub fn logout(&mut self) {
        self.current_user = None;
    }

    pub fn get_current_username(&self) -> String {
        match &self.current_user {
            Some(user) => user.username.clone(),
            None => String::from("Guest"),
        }
    }

    pub fn get_current_uid(&self) -> u32 {
        self.current_user.as_ref().map(|u| u.uid).unwrap_or(1000)
    }

    pub fn delete_user(&mut self, username: &str) -> Result<(), &'static str> {
        // Prevent deleting the admin or the currently logged-in user
        if username == "andre" {
            return Err("Cannot delete the primary administrator");
        }
        
        if let Some(ref current) = self.current_user {
            if current.username == username {
                return Err("Cannot delete the currently logged-in user");
            }
        }

        // Find index and remove
        if let Some(pos) = self.users.iter().position(|u| u.username == username) {
            self.users.remove(pos);
            Ok(())
        } else {
            Err("User not found")
        }
    }

    

}

lazy_static! {
    pub static ref AUTH: Mutex<AuthManager> = Mutex::new(AuthManager::new());
}