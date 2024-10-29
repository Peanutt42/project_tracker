use std::time::{SystemTime, UNIX_EPOCH};
use std::hash::{DefaultHasher, Hash, Hasher};
use thiserror::Error;
use serde::{Deserialize, Serialize};

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
pub struct Request {
	pub password_hash: String,
	pub request_type: RequestType,
}

pub fn hash_password(password: String) -> String {
	let mut hasher = DefaultHasher::default();
	password.hash(&mut hasher);
	hasher.finish().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
	GetModifiedDate,
	DownloadDatabase,
	UpdateDatabase {
		database_json: String
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
	ModifiedDate(ModifiedDate),
	Database {
		database_json: String,
	},
	InvalidPassword,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModifiedDate {
	pub seconds_since_epoch: u64,
}

impl From<SystemTime> for ModifiedDate {
	fn from(value: SystemTime) -> Self {
		let duration_since_epoch = value.duration_since(UNIX_EPOCH)
			.expect("invalid system time!");
		Self {
			seconds_since_epoch: duration_since_epoch.as_secs(),
		}
	}
}