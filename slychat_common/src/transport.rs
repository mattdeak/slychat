use crate::types::APICommand;
use futures::prelude::*;
use serde::de::DeserializeOwned;
use serde_json::json;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serde::formats::SymmetricalJson;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

#[derive(Debug, Clone)]
pub enum TransportError {
    WriteError,
    ReadError(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WriteError => write!(f, "Invalid Write Transport Operation"),
            Self::ReadError(message) => write!(f, "Invalid Read Transport Operation. {}", message),
        }
    }
}

impl std::error::Error for TransportError {}

pub async fn send_command<W, C>(writer: &mut W, command: &C) -> Result<(), TransportError>
where
    W: AsyncWriteExt + Send + Unpin + 'static,
    C: APICommand,
{
    let length_delimited = FramedWrite::new(writer, LengthDelimitedCodec::new());
    let mut serialized =
        tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalJson::default());

    serialized.send(json!(command)).await.unwrap();
    Ok(())
}

pub async fn read_command<'de, R, C>(reader: &mut R) -> Result<C, TransportError>
where
    R: AsyncReadExt + Send + Unpin + 'static,
    C: APICommand + DeserializeOwned,
{
    let length_delimited = FramedRead::new(reader, LengthDelimitedCodec::new());

    let mut deserialized = tokio_serde::SymmetricallyFramed::new(
        length_delimited,
        SymmetricalJson::<Value>::default(),
    );

    match deserialized.try_next().await {
        Ok(Some(v)) => Ok(serde_json::from_value::<C>(v)
            .map_err(|_| TransportError::ReadError("Error deserializing".to_string()))?),
        Ok(None) => Err(TransportError::ReadError("No data".to_string())),
        Err(e) => {
            eprintln!("Failed to read command. Got Error: {}", e);
            Err(TransportError::ReadError("Error reading".to_string()))
        }
    }
}
