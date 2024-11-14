use chrono::{DateTime, Utc};
use project_tracker_server::{encrypt, decrypt, Request, Response, ServerError, ServerResult, DEFAULT_HOSTNAME, DEFAULT_PASSWORD, DEFAULT_PORT};
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

	Request::GetModifiedDate.send_async(&mut write_half)
	.await?;

	let server_modified_date = match Response::read_async(&mut read_half).await? {
		Response::ModifiedDate(date) => date,
		// should not send database when only asked for the modified date
		Response::Database { .. } | Response::DatabaseUpdated => return Err(ServerError::InvalidResponse),
		Response::InvalidPassword => return Err(ServerError::InvalidPassword),
	};

	if server_modified_date > database_last_modified_date {
		// download database
		Request::DownloadDatabase
			.send_async(&mut write_half)
			.await?;

		match Response::read_async(&mut read_half).await? {
			Response::Database{ encrypted_database_json, salt, nonce } => {
				let database_json_bytes = decrypt(&encrypted_database_json, &config.password, &salt, &nonce)
        			.map_err(|_| ServerError::InvalidPassword)?;

				let database_json = String::from_utf8(database_json_bytes).map_err(|_| ServerError::InvalidResponse)?;
				match serde_json::from_str(&database_json) {
					Ok(database) => Ok(SyncServerDatabaseResponse::DownloadedDatabase(database)),
					Err(_) => Err(ServerError::InvalidResponse),
				}
			},
			// should not send database when only asked for the modified date
			Response::ModifiedDate(_) | Response::DatabaseUpdated => Err(ServerError::InvalidResponse),
			Response::InvalidPassword => Err(ServerError::InvalidPassword),
		}
	}
	else {
		// upload database
		let database_json = database.to_json();
		let (encrypted_database_json, salt, nonce) = encrypt(database_json.as_bytes(), &config.password)
			.map_err(|_| ServerError::InvalidPassword)?;

		Request::UpdateDatabase { encrypted_database_json, salt, nonce }
			.send_async(&mut write_half)
			.await?;

		match Response::read_async(&mut read_half).await? {
			Response::DatabaseUpdated => Ok(SyncServerDatabaseResponse::UploadedDatabase),
			Response::InvalidPassword => Err(ServerError::InvalidPassword),
			_ => Err(ServerError::InvalidResponse),
		}
	}
}