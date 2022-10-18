use std::{collections::HashMap, error::Error, fmt::Display};

use slychat_common::UserKey;
use tokio::io::AsyncWriteExt;

use crate::{
    chatroom::{ChatRoom, ChatRoomError},
    user::ChatUser,
};

#[derive(Debug, Clone)]
enum ServerError {
    UserError(String),
    ChatRoomError(ChatRoomError),
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ServerError::UserError(s) => write!(f, "{}", s),
            ChatRoomError => ChatRoomError.fmt(f),
        }
    }
}

impl Error for ServerError {}

const DEFAULT_CAPACITY: usize = 64;
const WAITING_ROOM: String = "waiting_room".to_string();
pub struct Server<G: ChatRoom> {
    // I think this will need to be cleverly synchronized
    pub users: Vec<ChatUser>,
    // chat_rooms: HashMap<String, ChatRoom<'static>>,
    pub chat_rooms: HashMap<String, G>,
}

impl<G: ChatRoom> Server<G> {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            chat_rooms: HashMap::new(),
        }
    }

    pub fn create_user(&mut self, user: ChatUser) -> Result<(), ServerError> {
        // Should do a membership check first
        if !self
            .users
            .iter()
            .any(|cu| user.user_data.user == cu.user_data.user)
        {
            self.users.push(user);
            self.chat_rooms[&WAITING_ROOM].register_user(&mut user);
            Ok(())
        } else {
            Err(ServerError::UserError("User already exists.".to_string()))
        }
    }

    pub fn create_chatroom(&mut self, chatroom_name: String, capacity: usize) -> Result<&G, &str> {
        if self.chat_rooms.contains_key(&chatroom_name) {
            return Err("Chatroom could not be created.");
        }

        let chatroom = ChatRoom::build(chatroom_name.clone(), capacity);
        self.chat_rooms.insert(chatroom_name.clone(), chatroom);

        // Probably inefficient
        Ok(&self.chat_rooms[&chatroom_name])
    }

    pub fn delete_chatroom(&mut self, chatroom_name: String) -> Result<(), &str> {
        if self.chat_rooms.contains_key(&chatroom_name) {
            self.chat_rooms.remove(&chatroom_name);
            return Ok(());
        } else {
            return Err("Chatroom not found.");
        }
    }

    pub async fn refresh_roomkeys(&self, username: &String) {
        if let Some(user) = self.users.iter().find(|u| &u.user_data.user == username) {
            self.send_roomkeys(&user.socket).await;
        }
    }

    async fn send_roomkeys<W: AsyncWriteExt + Unpin + Send + 'static>(&self, socket: &W) {}
}
