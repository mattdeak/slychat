use super::user::ChatUser;
use std::error::Error;
use std::fmt::Display;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub enum ChatRoomError {
    RegistrationFailure(Option<&'static str>),
    UserAlreadyExists(&'static str),
    MessageError(Option<&'static str>),
}

impl Display for ChatRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
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
    fn register_user(&mut self, user: &'static mut ChatUser) -> Result<(), ChatRoomError>;
    fn unregister_user(&mut self, username: String) -> Result<(), ChatRoomError>;
    fn publish_message(&self, message: &'static str) -> Result<(), ChatRoomError>;
}

type ChannelInfo = (String, Sender<String>);
pub struct SimpleChatRoom {
    pub id: String,
    pub capacity: usize,
    pub current_size: usize,

    // A vector of user_id, sender pairs.
    pub channels: Vec<ChannelInfo>,
    // A vec rather than a HashMap because we need to iterate through it a lot
    // user_collection: Arc<Mutex<UserRouter>>,
}

impl SimpleChatRoom {
    fn has_user(&self, username: &String) -> bool {
        self.channels.iter().any(|(u, _)| username == u)
    }

    fn remove_user(&mut self, username: &String) {
        for i in 0..self.channels.len() {
            let u = &self.channels[i].0.clone();
            if u == username {
                self.channels.swap_remove(i);
                break;
            }
        }
    }
}

impl ChatRoom for SimpleChatRoom {
    fn build(id: String, capacity: usize) -> Self {
        Self {
            id,
            capacity,
            current_size: 0,
            channels: Vec::new(),
        }
    }

    fn register_user(&mut self, user: &'static mut ChatUser) -> Result<(), ChatRoomError> {
        // Create a channel for sending the user messages
        let (sender, receiver) = tokio::sync::mpsc::channel::<String>(1024);
        let username = &user.user_data.user;

        // Check if user already exists in chatroom
        if self.channels.iter().any(|(u, _)| u == username) {
            return Err(ChatRoomError::UserAlreadyExists(username));
        }

        user.receivers.insert(self.id.clone(), receiver);
        self.channels.push((user.user_data.user.clone(), sender));

        Ok(())
    }

    fn unregister_user(&mut self, username: String) -> Result<(), ChatRoomError> {
        if self.has_user(&username) {
            self.remove_user(&username);
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
}
