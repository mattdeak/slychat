use slychat_common::APICommand;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::{
    chatroom::ChatRoom,
    server::{self, Server},
    user::ChatUser,
};

async fn initialize_connection<G: ChatRoom>(socket: TcpStream, server: Server<G>) {
    let (reader, mut writer) = tokio::io::split(socket);
    let mut buffer: Vec<u8> = vec![0; 1024];
    loop {
        buffer.clear();
        match socket.read(&mut buffer).await {
            Ok(n) if n > 0 => _,
            _ => {
                eprintln!("Socket Died");
                break;
            }
        }

        let result = serde_json::from_slice(&buffer);
        match result {
            Ok(APICommand::Greet(user_key)) => {
                let chat_user = ChatUser::build(user_key, writer);
                server.create_user(chat_user);
                // TODO: Refresh roomkeys
            }
            Ok(APICommand::RefreshRoomKeysRequest) => {
                todo!();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {}
