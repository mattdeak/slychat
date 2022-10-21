use std::fmt::Display;

use log::info;
use slychat_common::transport::{read_command, send_command, TransportError};
use slychat_common::types::{APICommand, APIRequest, APIResponse, Response, UserKey};
use tokio::select;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

use crate::user;
use crate::{
    chatroom::ChatRoom,
    server::{self, Server},
    user::ChatUser,
};

#[derive(Debug, Clone)]
enum ListenerError {
    Transport(TransportError),
    Error(&'static str),
}

impl Display for ListenerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "{}", e),
            Self::Error(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for ListenerError {}

pub async fn initialize_connection<G: ChatRoom>(socket: TcpStream, server: &mut Server<G>) {
    let (mut reader, mut writer) = tokio::io::split(socket);

    // Handle greeting from socket
    let greeting = wait_for_greeting(&mut reader, &writer).await;
    match greeting {
        Ok(key) => {
            // Register the user
            if register_user(key, writer, server).await.is_err() {
                eprintln!("Registration Failed");
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    };

    // Start main loop
    let mut buffer: Vec<u8> = vec![0; 1024];
    loop {
        buffer.clear();

        match read_command(&mut reader).await {
            Ok(command) => match command {
                APIRequest::LoginRequest(_) => todo!(),
                APIRequest::Logout => todo!(),
                APIRequest::RefreshRoomKeysRequest => todo!(),
                APIRequest::SendMessageRequest(_, _) => todo!(),
                APIRequest::ListRoomsRequest => todo!(),
                APIRequest::JoinRoomRequest(_) => todo!(),
                APIRequest::LeaveRoom => todo!(),
            },
            Err(e) => {}
        }
    }
}

async fn wait_for_greeting(
    reader: &mut ReadHalf<TcpStream>,
    writer: &WriteHalf<TcpStream>,
) -> Result<UserKey, ListenerError> {
    match read_command(reader).await {
        Ok(command) => match command {
            APIRequest::LoginRequest(user_key) => {
                println!("Found User: {}", &user_key.user);
                Ok(user_key)
            }
            _ => Err(ListenerError::Error(
                "Expected greeting, got different command",
            )),
        },
        Err(e) => Err(ListenerError::Transport(e)),
    }
}

async fn register_user<G: ChatRoom>(
    key: UserKey,
    writer: WriteHalf<TcpStream>,
    server: &mut Server<G>,
) -> Result<(), Box<dyn std::error::Error>> {
    let chat_user = ChatUser::build(key, writer);
    let user = server.create_user(chat_user)?;

    match send_command(
        &mut user.socket,
        APIResponse::LoginResponse(Response::Success(())),
    )
    .await
    {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(e))?,
    }
}

#[cfg(test)]
mod tests {}
