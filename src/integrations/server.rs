use project_tracker_core::Database;
use project_tracker_server::{Request, Response, ServerError, ServerResult, DEFAULT_HOSTNAME, DEFAULT_PASSWORD, DEFAULT_PORT};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

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

pub async fn sync_database_from_server(config: ServerConfig, database: Database) -> ServerResult<SyncServerDatabaseResponse> {
	let stream = TcpStream::connect(format!("{}:{}", config.hostname, config.port)).await?;
	let (mut read_half, mut write_half) = stream.into_split();

	Request::GetModifiedDate.send_async(&mut write_half, &config.password)
	.await?;

	let server_modified_date = match Response::read_async(&mut read_half, &config.password).await? {
		Response::ModifiedDate(date) => date,
		// should not send database when only asked for the modified date
		Response::Database { .. } | Response::DatabaseUpdated => return Err(ServerError::InvalidResponse),
		Response::InvalidPassword => return Err(ServerError::InvalidPassword),
		Response::InvalidDatabaseBinary => return Err(ServerError::InvalidDatabaseBinaryFormat),
	};

	let database_last_modified_date = *database.last_changed_time();

	if server_modified_date > database_last_modified_date {
		// download database
		Request::DownloadDatabase
			.send_async(&mut write_half, &config.password)
			.await?;

		match Response::read_async(&mut read_half, &config.password).await? {
			Response::Database{ database_binary, last_modified_time } => match Database::from_binary(&database_binary, last_modified_time) {
				Ok(database) => Ok(SyncServerDatabaseResponse::DownloadedDatabase(database)),
				Err(_) => Err(ServerError::InvalidResponse),
			},
			// should not send database when only asked for the modified date
			Response::ModifiedDate(_) | Response::DatabaseUpdated => Err(ServerError::InvalidResponse),
			Response::InvalidPassword => Err(ServerError::InvalidPassword),
			Response::InvalidDatabaseBinary => Err(ServerError::InvalidDatabaseBinaryFormat),
		}
	}
	else {
		// upload database
		match database.to_binary() {
			Some(database_binary) => {
				Request::UpdateDatabase { database_binary, last_modified_time: database_last_modified_date }
					.send_async(&mut write_half, &config.password)
					.await?;

				match Response::read_async(&mut read_half, &config.password).await? {
					Response::DatabaseUpdated => Ok(SyncServerDatabaseResponse::UploadedDatabase),
					Response::InvalidPassword => Err(ServerError::InvalidPassword),
					_ => Err(ServerError::InvalidResponse),
				}
			},
			None => {
				eprintln!("failed to serialize database to upload it to the server, sending InvalidResponse error");
				Err(ServerError::InvalidResponse)
			},
		}
	}
}