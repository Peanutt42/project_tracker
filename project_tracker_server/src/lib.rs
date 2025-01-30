#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

use chrono::{DateTime, Datelike, Local, Timelike, Utc};
use project_tracker_core::{
	get_last_modification_date_time, Database, DatabaseMessage, SerializedDatabase,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};
use tracing::error;

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
		database_messages: Vec<DatabaseMessage>,
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
	pub fn encrypt(request: &Request, password: &str) -> ServerResult<Vec<u8>> {
		let serialized_request = Self(Encrypted::encrypt(request, password)?);
		bincode::serialize(&serialized_request).map_err(|_| ServerError::FailedToSerializeToBinary)
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
		database_messages: Vec<DatabaseMessage>,
	},
	DatabaseUpdated,
	AdminInfos(AdminInfos),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedResponse(pub Result<Encrypted<Response>, ServerError>);

impl SerializedResponse {
	pub fn ok(response: &Response, password: &str) -> ServerResult<Vec<u8>> {
		bincode::serialize(&Self(Ok(Encrypted::encrypt(response, password)?)))
			.map_err(|_| ServerError::FailedToSerializeToBinary)
	}

	pub fn error(error: ServerError) -> ServerResult<Vec<u8>> {
		bincode::serialize(&Self(Err(error))).map_err(|_| ServerError::FailedToSerializeToBinary)
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
		database_messages: Vec<DatabaseMessage>,
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

pub fn load_database_from_file(filepath: PathBuf) -> Option<Database> {
	let last_modified_time = get_last_modification_date_time(&filepath.metadata().ok()?)?;

	let database_file_content = std::fs::read(&filepath).ok()?;

	Database::from_binary(&database_file_content, last_modified_time).ok()
}

pub async fn save_database_to_file(database_filepath: &PathBuf, database_binary: &[u8]) {
	if let Err(e) = tokio::fs::write(database_filepath, database_binary).await {
		error!(
			"cant write database to file: {}, error: {e}",
			database_filepath.display()
		);

		// try to save database to a different filepath that contains the date, in order to not have a file names that could theoretically cause any problems
		let mut tmp_backup_database_filepath = database_filepath.clone();
		let now = Local::now();
		let formatted_date_time = format!(
			"{}_{}_{} - {}_{}_{}",
			now.day(),
			now.month(),
			now.year(),
			now.hour(),
			now.minute(),
			now.second()
		);
		tmp_backup_database_filepath.set_file_name(format!(
			"tmp_backup_database_{formatted_date_time}.project_tracker"
		));
		if let Err(e) = tokio::fs::write(&tmp_backup_database_filepath, database_binary).await {
			error!(
				"failed to write database to tmp backup file ('{}') after already failing to save database to original filepath ('{}'): {e}",
				tmp_backup_database_filepath.display(),
				database_filepath.display()
			);
		}
	}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
	use crate::{
		Request, Response, SerializedRequest, SerializedResponse, ServerError, DEFAULT_PASSWORD,
	};

	const TEST_PASSWORD: &str = DEFAULT_PASSWORD;

	#[test]
	fn test_request_serialization() {
		let request = Request::GetFullDatabase;

		let request_bytes = SerializedRequest::encrypt(&request, TEST_PASSWORD).unwrap();
		assert_eq!(
			SerializedRequest::decrypt(&request_bytes, TEST_PASSWORD).unwrap(),
			request
		);
	}

	#[test]
	fn test_response_serialization() {
		let response = Response::DatabaseUpdated;

		let response_bytes = SerializedResponse::ok(&response, TEST_PASSWORD).unwrap();
		assert_eq!(
			SerializedResponse::decrypt(&response_bytes, TEST_PASSWORD).unwrap(),
			response
		);

		let error_response = ServerError::InvalidPassword;
		let error_response_bytes = SerializedResponse::error(error_response.clone()).unwrap();
		assert_eq!(
			error_response,
			SerializedResponse::decrypt(&error_response_bytes, TEST_PASSWORD).unwrap_err()
		);
	}
}
