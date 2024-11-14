use std::{net::TcpStream, io::{Read, Write}};
use chrono::{DateTime, Utc};
use filetime::FileTime;
use thiserror::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
#[cfg(feature = "async_tokio")]
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::tcp::{OwnedReadHalf, OwnedWriteHalf}};

mod password_hash;
pub use password_hash::PasswordHash;

pub const DEFAULT_HOSTNAME: &str = "127.0.0.1";
pub const DEFAULT_PORT: usize = 8080;
pub const DEFAULT_PASSWORD: &str = "1234";

#[derive(Debug, Error)]
pub enum ServerError {
	#[error("connection failed with server: {0}")]
	ConnectionError(#[from] std::io::Error),
	#[error("failed to parse server response: {0}")]
	ParseError(#[from] serde_json::Error),
	#[error("invalid response from server")]
	InvalidResponse,
	#[error("invalid password")]
	InvalidPassword,
}

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
	GetModifiedDate,
	DownloadDatabase,
	UpdateDatabase {
		database_json: String
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
	pub password_hash: PasswordHash,
	pub request_type: RequestType,
}

impl Request {
	pub fn new(password_hash: PasswordHash, request_type: RequestType) -> Self {
		Self {
			password_hash,
			request_type,
		}
	}

	pub fn send(&self, stream: &mut TcpStream) -> ServerResult<()> {
		send_message(stream, self)
	}
	pub fn read(stream: &mut TcpStream) -> ServerResult<Self> {
		read_message(stream)
	}
	#[cfg(feature = "async_tokio")]
	pub async fn send_async(&self, stream: &mut OwnedWriteHalf) -> ServerResult<()> {
		send_message_async(stream, self).await
	}
	#[cfg(feature = "async_tokio")]
	pub async fn read_async(stream: &mut OwnedReadHalf) -> ServerResult<Self> {
		read_message_async(stream).await
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
	ModifiedDate(DateTime<Utc>),
	Database {
		database_json: String,
	},
	InvalidPassword,
}
impl Response {
	pub fn send(&self, stream: &mut TcpStream) -> ServerResult<()> {
		send_message(stream, self)
	}
	pub fn read(stream: &mut TcpStream) -> ServerResult<Self> {
		read_message(stream)
	}
	#[cfg(feature = "async_tokio")]
	pub async fn send_async(&self, stream: &mut OwnedWriteHalf) -> ServerResult<()> {
		send_message_async(stream, self).await
	}
	#[cfg(feature = "async_tokio")]
	pub async fn read_async(stream: &mut OwnedReadHalf) -> ServerResult<Self> {
		read_message_async(stream).await
	}
}

pub fn get_last_modification_date_time(metadata: &std::fs::Metadata) -> DateTime<Utc> {
	let modified = FileTime::from_last_modification_time(metadata);

	let unix_timestamp = modified.unix_seconds();
	let nanos = modified.nanoseconds();

	DateTime::from_timestamp(unix_timestamp, nanos)
		.expect("invalid file modification date timestamp")
}


fn send_message<T: Serialize>(stream: &mut TcpStream, message: &T) -> ServerResult<()> {
	let message_json = serde_json::to_string(message)?;
	let message_bytes = message_json.as_bytes();
	let message_len = message_bytes.len();
	let message_len_bytes = (message_len as u32).to_be_bytes();

	stream.write_all(&message_len_bytes)?;
	stream.write_all(message_bytes)?;

	Ok(())
}

fn read_message<T: DeserializeOwned>(stream: &mut TcpStream) -> ServerResult<T> {
	let mut message_len_bytes = [0u8; 4];
	stream.read_exact(&mut message_len_bytes)?;
	let message_len = u32::from_be_bytes(message_len_bytes) as usize;

	let mut message_bytes = vec![0u8; message_len];
	stream.read_exact(&mut message_bytes)?;

	let message_json = String::from_utf8_lossy(&message_bytes);
	serde_json::from_str(&message_json)
		.map_err(ServerError::ParseError)
}


#[cfg(feature = "async_tokio")]
async fn send_message_async<T: Serialize>(stream: &mut OwnedWriteHalf, message: &T) -> ServerResult<()> {
	let message_json = serde_json::to_string(message)?;
	let message_bytes = message_json.as_bytes();
	let message_len = message_bytes.len();
	let message_len_bytes = (message_len as u32).to_be_bytes();

	stream.write_all(&message_len_bytes).await?;
	stream.write_all(message_bytes).await?;

	Ok(())
}

#[cfg(feature = "async_tokio")]
async fn read_message_async<T: DeserializeOwned>(stream: &mut OwnedReadHalf) -> ServerResult<T> {
	let mut message_len_bytes = [0u8; 4];
	stream.read_exact(&mut message_len_bytes).await?;
	let message_len = u32::from_be_bytes(message_len_bytes) as usize;

	let mut message_bytes = vec![0u8; message_len];
	stream.read_exact(&mut message_bytes).await?;

	let message_json = String::from_utf8_lossy(&message_bytes);
	serde_json::from_str(&message_json)
		.map_err(ServerError::ParseError)
}