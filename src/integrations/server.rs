use chrono::{DateTime, Utc};
use project_tracker_server::{hash_password, Request, RequestType, Response, ServerError, ServerResult, DEFAULT_HOSTNAME, DEFAULT_PASSWORD, DEFAULT_PORT};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use crate::core::Database;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerConfig {
	pub hostname: String,
	pub port: usize,
	pub password: String,
}

impl Default for ServerConfig {
	fn default() -> Self {
		Self {
			hostname: DEFAULT_HOSTNAME.to_string(),
			port: DEFAULT_PORT,
			password: DEFAULT_PASSWORD.to_string(),
		}
	}
}

pub enum SyncServerDatabaseResponse {
	UploadedDatabase,
	DownloadedDatabase(Database),
}

pub async fn sync_database_from_server(config: ServerConfig, database_last_modified_date: DateTime<Utc>, database: Database) -> ServerResult<SyncServerDatabaseResponse> {
	let stream = TcpStream::connect(format!("{}:{}", config.hostname, config.port)).await?;
	let (mut read_half, mut write_half) = stream.into_split();
	let password_hash = hash_password(config.password);

	Request {
		password_hash: password_hash.clone(),
		request_type: RequestType::GetModifiedDate,
	}
	.send_async(&mut write_half)
	.await?;

	let server_modified_date = match Response::read_async(&mut read_half).await? {
		Response::ModifiedDate(date) => date,
		// should not send database when only asked for the modified date
		Response::Database { .. } => return Err(ServerError::InvalidResponse),
		Response::InvalidPassword => return Err(ServerError::InvalidPassword),
	};

	if server_modified_date > database_last_modified_date {
		// download database
		Request {
			password_hash: password_hash.clone(),
			request_type: RequestType::DownloadDatabase,
		}
		.send_async(&mut write_half)
		.await?;

		match Response::read_async(&mut read_half).await? {
			Response::Database{ database_json } => match serde_json::from_str(&database_json) {
				Ok(database) => Ok(SyncServerDatabaseResponse::DownloadedDatabase(database)),
				Err(_) => Err(ServerError::InvalidResponse),
			},
			// should not send database when only asked for the modified date
			Response::ModifiedDate(_) => Err(ServerError::InvalidResponse),
			Response::InvalidPassword => Err(ServerError::InvalidPassword),
		}
	}
	else {
		// upload database
		Request {
			password_hash,
			request_type: RequestType::UpdateDatabase { database_json: database.to_json() },
		}
		.send_async(&mut write_half)
		.await
		.map(|_| SyncServerDatabaseResponse::UploadedDatabase)
	}
}