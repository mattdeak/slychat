use slychat_common::{APICommand, UserKey};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

use crate::{
    chatroom::ChatRoom,
    server::{self, Server},
    user::ChatUser,
};

pub async fn initialize_connection<G: ChatRoom>(socket: TcpStream, server: &mut Server<G>) {
    let (mut reader, mut writer) = tokio::io::split(socket);

    // Handle greeting from socket
    let greeting = wait_for_greeting(&mut reader, &writer).await;
    match greeting {
        Ok(key) => {
            // Register the user
            if register_user(key, writer, server).await.is_err() {
                eprintln!("Registration Failed");
                return;
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    // Start main loop
    let mut buffer: Vec<u8> = vec![0; 1024];
    loop {
        buffer.clear();
        let bytes_read = match reader.read(&mut buffer).await {
            Ok(n) if n > 0 => n,
            _ => {
                eprintln!("Socket Died");
                break;
            }
        };

        let r = serde_json::from_reader(reader);
        let result = serde_json::from_slice(&buffer[..bytes_read]);
        match result {
            Ok(APICommand::RefreshRoomKeysRequest) => {
                todo!();
            }
            _ => todo!(),
        }
    }
}

async fn wait_for_greeting(
    reader: &mut ReadHalf<TcpStream>,
    writer: &WriteHalf<TcpStream>,
) -> Result<UserKey, Box<dyn std::error::Error>> {
    let mut buffer: Vec<u8> = vec![0; 1024];

    match reader.read(&mut buffer).await {
        Ok(n) if n > 0 => {
            let result = serde_json::from_slice(&buffer)?;

            match &result {
                APICommand::Greet(user_key) => Ok(user_key.clone()),
                APICommand => Err("Expected Greet, got something else".into()),
                _ => Err("Got unparseable result".into()),
            }
        }
        _ => Err("Error Reading Socket".into()),
    }
}

async fn register_user<G: ChatRoom>(
    key: UserKey,
    writer: WriteHalf<TcpStream>,
    server: &mut Server<G>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut chat_user = ChatUser::build(key, writer);
    let user = server.create_user(chat_user)?;

    user.socket
        .write_all(serde_json::to_vec(&APICommand::GreetAck)?.as_ref())
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {}
