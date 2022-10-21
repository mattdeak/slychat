use slychat_common::types::UserKey;
use std::collections::{HashMap, HashSet};
use tokio::io::AsyncWriteExt;
use tokio::io::WriteHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct ChatUser {
    pub user_data: UserKey,
    pub socket: WriteHalf<TcpStream>,
    // Hashmap<ChatroomKey, ChatroomReceiver>
    pub receivers: HashMap<String, Receiver<String>>,
}

impl ChatUser {
    pub fn build(user_data: UserKey, socket: WriteHalf<TcpStream>) -> Self {
        Self {
            user_data,
            socket,
            receivers: HashMap::new(),
        }
    }
}

// #[cfg(test)]
// mod tests {

//     use super::{ChatUser, UserCollection};
//     use slychat_common::UserKey;

//     struct TcpStream {}

//     // struct User {
//     //     socket: Option<TcpStream>
//     // };

//     #[test]
//     fn add_user() {
//         let mut collection = UserCollection::new();

//         let user = ChatUser::build(
//             UserKey {
//                 user: String::from("test_user"),
//                 public: Vec::new(),
//             },
//             TcpStream {},
//         );

//         collection.insert(&user);
//         assert!(collection.contains_user(&user.user_data.user));
//         collection.remove(&user.user_data.user);
//         assert!(!collection.contains_user(&user.user_data.user));
//     }
// }
