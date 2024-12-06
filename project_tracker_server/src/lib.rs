use std::{net::TcpStream, io::{Read, Write}};
use chrono::{DateTime, Utc};
use thiserror::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::tcp::{OwnedReadHalf, OwnedWriteHalf}};

mod server;
pub use server::run_server;

mod encryption;
pub use encryption::{encrypt, decrypt, SALT_LENGTH, NONCE_LENGTH};

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
	#[error("invalid database binary format")]
	InvalidDatabaseBinaryFormat,
}

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
	GetModifiedDate,
	DownloadDatabase,
	UpdateDatabase {
		database_binary: Vec<u8>,
		last_modified_time: DateTime<Utc>,
	}
}

impl Request {
	pub fn send(&self, stream: &mut TcpStream, password: &str) -> ServerResult<()> {
		send_message(stream, self, password)
	}
	pub fn read(stream: &mut TcpStream, password: &str) -> ServerResult<Self> {
		read_message(stream, password)
	}
	pub async fn send_async(&self, stream: &mut OwnedWriteHalf, password: &str) -> ServerResult<()> {
		send_message_async(stream, self, password).await
	}
	pub async fn read_async(stream: &mut OwnedReadHalf, password: &str) -> ServerResult<Self> {
		read_message_async(stream, password).await
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
	ModifiedDate(DateTime<Utc>),
	Database {
		database_binary: Vec<u8>,
	},
	DatabaseUpdated,
	InvalidPassword,
	InvalidDatabaseBinary,
}
impl Response {
	pub fn send(&self, stream: &mut TcpStream, password: &str) -> ServerResult<()> {
		send_message(stream, self, password)
	}
	pub fn read(stream: &mut TcpStream, password: &str) -> ServerResult<Self> {
		read_message(stream, password)
	}
	pub async fn send_async(&self, stream: &mut OwnedWriteHalf, password: &str) -> ServerResult<()> {
		send_message_async(stream, self, password).await
	}
	pub async fn read_async(stream: &mut OwnedReadHalf, password: &str) -> ServerResult<Self> {
		read_message_async(stream, password).await
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedMessage {
	encrypted_message: Vec<u8>,
	salt: [u8; SALT_LENGTH],
	nonce: [u8; NONCE_LENGTH],
}

impl EncryptedMessage {
	fn new(plaintext_message: &[u8], password: &str) -> ServerResult<Self> {
		let (encrypted_message, salt, nonce) = encrypt(plaintext_message, password)
			.map_err(|_| ServerError::InvalidPassword)?;

		Ok(Self {
			encrypted_message,
			salt,
			nonce,
		})
	}

	fn decrypt(&self, password: &str) -> ServerResult<Vec<u8>> {
		decrypt(&self.encrypted_message, password, &self.salt, &self.nonce)
			.map_err(|_| ServerError::InvalidPassword)
	}
}

fn send_message<T: Serialize>(stream: &mut TcpStream, message: &T, password: &str) -> ServerResult<()> {
	let message_json = serde_json::to_string(message)?;
	let encrypted_message = EncryptedMessage::new(message_json.as_bytes(), password)?;
	let encrypted_message_json = serde_json::to_string(&encrypted_message)?;
	let encrypted_message_bytes = encrypted_message_json.as_bytes();
	let encrypted_message_len = encrypted_message_bytes.len();
	let encrypted_message_len_bytes = (encrypted_message_len as u32).to_be_bytes();

	stream.write_all(&encrypted_message_len_bytes)?;
	stream.write_all(encrypted_message_bytes)?;

	Ok(())
}

fn read_message<T: DeserializeOwned>(stream: &mut TcpStream, password: &str) -> ServerResult<T> {
	let mut encrypted_message_len_bytes = [0u8; 4];
	stream.read_exact(&mut encrypted_message_len_bytes)?;
	let encrypted_message_len = u32::from_be_bytes(encrypted_message_len_bytes) as usize;

	let mut encrypted_message_bytes = vec![0u8; encrypted_message_len];
	stream.read_exact(&mut encrypted_message_bytes)?;

	let encrypted_message_json = String::from_utf8(encrypted_message_bytes)
		.map_err(|_| ServerError::InvalidResponse)?;
	let encrypted_message: EncryptedMessage = serde_json::from_str(&encrypted_message_json)
		.map_err(ServerError::ParseError)?;

	let message_bytes = encrypted_message.decrypt(password)?;

	let message_json = String::from_utf8(message_bytes).map_err(|_| ServerError::InvalidResponse)?;
	serde_json::from_str(&message_json)
		.map_err(ServerError::ParseError)
}


async fn send_message_async<T: Serialize>(stream: &mut OwnedWriteHalf, message: &T, password: &str) -> ServerResult<()> {
	let message_json = serde_json::to_string(message)?;
	let encrypted_message = EncryptedMessage::new(message_json.as_bytes(), password)?;
	let encrypted_message_json = serde_json::to_string(&encrypted_message)?;
	let encrypted_message_bytes = encrypted_message_json.as_bytes();
	let encrypted_message_len = encrypted_message_bytes.len();
	let encrypted_message_len_bytes = (encrypted_message_len as u32).to_be_bytes();

	stream.write_all(&encrypted_message_len_bytes).await?;
	stream.write_all(encrypted_message_bytes).await?;

	Ok(())
}

async fn read_message_async<T: DeserializeOwned>(stream: &mut OwnedReadHalf, password: &str) -> ServerResult<T> {
	let mut encrypted_message_len_bytes = [0u8; 4];
	stream.read_exact(&mut encrypted_message_len_bytes).await?;
	let encrypted_message_len = u32::from_be_bytes(encrypted_message_len_bytes) as usize;

	let mut encrypted_message_bytes = vec![0u8; encrypted_message_len];
	stream.read_exact(&mut encrypted_message_bytes).await?;

	let encrypted_message_json = String::from_utf8(encrypted_message_bytes)
		.map_err(|_| ServerError::InvalidResponse)?;
	let encrypted_message: EncryptedMessage = serde_json::from_str(&encrypted_message_json)
		.map_err(ServerError::ParseError)?;

	let message_bytes = encrypted_message.decrypt(password)?;

	let message_json = String::from_utf8(message_bytes).map_err(|_| ServerError::InvalidResponse)?;
	serde_json::from_str(&message_json)
		.map_err(ServerError::ParseError)
}