use std::hash::{DefaultHasher, Hash, Hasher};
use chrono::{DateTime, Utc};
use filetime::FileTime;
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
	ModifiedDate(DateTime<Utc>),
	Database {
		database_json: String,
	},
	InvalidPassword,
}

pub fn get_last_modification_date_time(metadata: &std::fs::Metadata) -> DateTime<Utc> {
	let modified = FileTime::from_last_modification_time(metadata);

	let unix_timestamp = modified.unix_seconds();
	let nanos = modified.nanoseconds();

	DateTime::from_timestamp(unix_timestamp, nanos)
		.expect("invalid file modification date timestamp")
}