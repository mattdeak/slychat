use log::info;
use slychat_common::types::UserKey;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub enum ChatRoomError {
    RegistrationFailure(Option<&'static str>),
    UserAlreadyExists(String),
    MessageError(Option<&'static str>),
}

impl Display for ChatRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatRoomError::RegistrationFailure(message) => {
                let err_msg = message.unwrap_or("");
                write!(f, "Registration Error{}", err_msg)
            }
            ChatRoomError::MessageError(message) => {
                let err_msg = message.unwrap_or("");
                write!(f, "Message Error{}", err_msg)
            }
            ChatRoomError::UserAlreadyExists(message) => {
                write!(f, "User {} Already Exists", message)
            }
        }
    }
}

impl Error for ChatRoomError {}

pub trait ChatRoom {
    fn build(id: String, capacity: usize) -> Self;
    fn register_user(&mut self, username: &String, key: Vec<u8>) -> Result<(), ChatRoomError>;
    fn unregister_user(&mut self, username: &String) -> Result<(), ChatRoomError>;

    fn get_roomkeys(&self) -> Result<Vec<(&String, &Vec<u8>)>, ChatRoomError>;
    fn publish_message(&self, message: &'static str) -> Result<(), ChatRoomError>;
}

pub struct SimpleChatRoom {
    pub id: String,
    pub capacity: usize,
    pub current_size: usize,

    pub registered_users: HashMap<String, Vec<u8>>,
    // A vec rather than a HashMap because we need to iterate through it a lot
    // user_collection: Arc<Mutex<UserRouter>>,
}

impl ChatRoom for SimpleChatRoom {
    fn build(id: String, capacity: usize) -> Self {
        Self {
            id,
            capacity,
            current_size: 0,
            registered_users: HashMap::new(),
        }
    }

    fn register_user(&mut self, username: &String, key: Vec<u8>) -> Result<(), ChatRoomError> {
        info!("Registering user {} into {}", username, self.id);
        // Create a channel for sending the user messages
        let (sender, receiver) = tokio::sync::mpsc::channel::<String>(1024);
        // let username = &user.user_data.user;

        // Check if user already exists in chatroom
        if self.registered_users.contains_key(username) {
            Err(ChatRoomError::UserAlreadyExists(username.into()))
        } else {
            self.registered_users.insert(username.into(), key);
            Ok(())
        }
    }

    fn unregister_user(&mut self, username: &String) -> Result<(), ChatRoomError> {
        if self.registered_users.contains_key(username) {
            self.registered_users.remove(username);
            Ok(())
        } else {
            Err(ChatRoomError::RegistrationFailure(Some(
                "User does not exist in chatroom.",
            )))
        }
    }

    fn publish_message(&self, message: &'static str) -> Result<(), ChatRoomError> {
        todo!()
    }

    fn get_roomkeys(&self) -> Result<Vec<(&String, &Vec<u8>)>, ChatRoomError> {
        let roomkeys: Vec<(&String, &Vec<u8>)> = self.registered_users.iter().collect();
        Ok(roomkeys)
    }
}
