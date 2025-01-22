use chrono::{DateTime, Utc};
use project_tracker_core::{
	get_last_modification_date_time, Database, DatabaseMessage, SerializedDatabase,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};

mod error;
pub use error::{ServerError, ServerResult};

mod server;
pub use server::{handle_client, run_server};

mod admin_infos;
pub use admin_infos::AdminInfos;

mod cpu_usage;
pub use cpu_usage::{messure_cpu_usage_avg_thread, CpuUsageAverage};

mod encryption;
pub use encryption::{Encrypted, EncryptionError};

mod logs;
pub use logs::get_logs_as_string;

pub const DEFAULT_HOSTNAME: &str = "127.0.0.1";
pub const DEFAULT_PORT: usize = 8080;
pub const DEFAULT_PASSWORD: &str = "1234";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Request {
	CheckUpToDate {
		database_checksum: u64,
	},
	GetFullDatabase,
	UpdateDatabase {
		database_message: DatabaseMessage,
		database_before_update_checksum: u64,
	},
	ImportDatabase {
		database: SerializedDatabase,
	},
	AdminInfos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedRequest(pub Encrypted<Request>);

impl SerializedRequest {
	pub fn encrypt(request: &Request, password: &str) -> Vec<u8> {
		let serialized_request = Self(Encrypted::encrypt(request, password));
		bincode::serialize(&serialized_request).expect("failed to serialize response")
	}

	pub fn decrypt(bytes: &[u8], password: &str) -> ServerResult<Request> {
		let serialized_request: Self =
			bincode::deserialize(bytes).map_err(|_| ServerError::RequestParseError)?;
		serialized_request.0.decrypt(password)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Response {
	DatabaseUpToDate,
	MoreUpToDateDatabase {
		database: SerializedDatabase,
		last_modified_time: DateTime<Utc>,
	},
	DatabaseChanged {
		database_before_update_checksum: u64,
		database_message: DatabaseMessage,
	},
	DatabaseUpdated,
	AdminInfos(AdminInfos),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedResponse(pub Result<Encrypted<Response>, ServerError>);

impl SerializedResponse {
	pub fn ok(response: &Response, password: &str) -> Vec<u8> {
		bincode::serialize(&Self(Ok(Encrypted::encrypt(response, password))))
			.expect("failed to serialize response")
	}

	pub fn error(error: ServerError) -> Vec<u8> {
		bincode::serialize(&Self(Err(error))).expect("failed to serialize response")
	}

	pub fn decrypt(bytes: &[u8], password: &str) -> ServerResult<Response> {
		let serialized_response: Self =
			bincode::deserialize(bytes).map_err(|_| ServerError::ResponseParseError)?;
		let encrypted_response = serialized_response.0?;
		encrypted_response.decrypt(password)
	}
}

#[derive(Debug, Clone)]
pub enum DatabaseUpdateEvent {
	DatabaseMessage {
		database_message: DatabaseMessage,
		before_modification_checksum: u64,
	},
	ImportDatabase,
}

#[derive(Debug, Clone)]
pub struct ModifiedEvent {
	pub modified_database: Database,
	pub database_update_event: DatabaseUpdateEvent,
	pub modified_sender_address: SocketAddr,
}

impl ModifiedEvent {
	pub fn new(
		modified_database: Database,
		database_update_event: DatabaseUpdateEvent,
		sender_addr: SocketAddr,
	) -> Self {
		Self {
			modified_database,
			database_update_event,
			modified_sender_address: sender_addr,
		}
	}
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ConnectedClient {
	NativeGUI(SocketAddr),
	Web(SocketAddr),
}

pub fn load_database_from_file(filepath: PathBuf) -> Database {
	let last_modified_time = get_last_modification_date_time(
		&filepath
			.metadata()
			.expect("Failed to get the last modified metadata of database file"),
	)
	.expect("Failed to get the last modified metadata of database file");

	let database_file_content =
		std::fs::read(&filepath).expect("Failed to read database file at startup!");

	Database::from_binary(&database_file_content, last_modified_time)
		.expect("Failed to parse database file content at startup!")
}

#[cfg(test)]
mod tests {
	use crate::{
		Request, Response, SerializedRequest, SerializedResponse, ServerError, DEFAULT_PASSWORD,
	};

	const TEST_PASSWORD: &str = DEFAULT_PASSWORD;

	#[test]
	fn test_request_serialization() {
		let request = Request::GetFullDatabase;

		let request_bytes = SerializedRequest::encrypt(&request, TEST_PASSWORD);
		assert_eq!(
			SerializedRequest::decrypt(&request_bytes, TEST_PASSWORD).unwrap(),
			request
		);
	}

	#[test]
	fn test_response_serialization() {
		let response = Response::DatabaseUpdated;

		let response_bytes = SerializedResponse::ok(&response, TEST_PASSWORD);
		assert_eq!(
			SerializedResponse::decrypt(&response_bytes, TEST_PASSWORD).unwrap(),
			response
		);

		let error_response = ServerError::InvalidPassword;
		let error_response_bytes = SerializedResponse::error(error_response.clone());
		assert_eq!(
			error_response,
			SerializedResponse::decrypt(&error_response_bytes, TEST_PASSWORD).unwrap_err()
		);
	}
}
