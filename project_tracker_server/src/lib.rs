use std::{path::PathBuf, sync::{Arc, RwLock}};
use chrono::{DateTime, Utc};
use project_tracker_core::{get_last_modification_date_time, Database};
use thiserror::Error;
use serde::{Deserialize, Serialize};

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
	ParseError(#[from] bincode::Error),
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
	pub fn decrypt(binary: Vec<u8>, password: &str) -> ServerResult<Self> {
		let encrypted_message: EncryptedMessage = bincode::deserialize(&binary)?;
		let request_binary = encrypted_message.decrypt(password)?;
		Ok(bincode::deserialize(&request_binary)?)
	}

	pub fn encrypt(&self, password: &str) -> ServerResult<Vec<u8>> {
		let request_binary = bincode::serialize(self)?;

		Ok(
			bincode::serialize(&EncryptedMessage::new(
				&request_binary,
				password
			)?)?
		)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
	ModifiedDate(DateTime<Utc>),
	Database {
		database_binary: Vec<u8>,
		last_modified_time: DateTime<Utc>,
	},
	DatabaseUpdated,
	InvalidPassword,
	InvalidDatabaseBinary,
}
impl Response {
	pub fn decrypt(binary: Vec<u8>, password: &str) -> ServerResult<Self> {
		let encrypted_message: EncryptedMessage = bincode::deserialize(&binary)?;
		let response_binary = encrypted_message.decrypt(password)?;
		Ok(bincode::deserialize(&response_binary)?)
	}

	pub fn encrypt(&self, password: &str) -> ServerResult<Vec<u8>> {
		let response_binary = bincode::serialize(self)?;

		Ok(
			bincode::serialize(&EncryptedMessage::new(
				&response_binary,
				password
			)?)?
		)
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


#[derive(Debug, Clone)]
pub struct SharedServerData {
	pub database: Database,
	pub last_modified_time: DateTime<Utc>,
}

impl SharedServerData {
	pub fn new(filepath: PathBuf) -> Arc<RwLock<Self>> {
		let last_modified_time = get_last_modification_date_time(
			&filepath.metadata().expect("Failed to get the last modified metadata of database file")
		)
		.expect("Failed to get the last modified metadata of database file");

		let database_file_content = std::fs::read(&filepath)
			.expect("Failed to read database file at startup!");

		let database = Database::from_binary(&database_file_content, last_modified_time)
			.expect("Failed to parse database file content at startup!");

		let shared_data = SharedServerData {
			database,
			last_modified_time,
		};

		Arc::new(RwLock::new(shared_data))
	}
}