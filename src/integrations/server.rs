use project_tracker_server::{ModifiedDate, Request, Response, ServerError, ServerResult, DEFAULT_HOSTNAME, DEFAULT_PORT};
use serde::{Deserialize, Serialize};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpStream}};
use crate::core::Database;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerConfig {
	pub hostname: String,
	pub port: usize,
}

impl Default for ServerConfig {
	fn default() -> Self {
		Self {
			hostname: DEFAULT_HOSTNAME.to_string(),
			port: DEFAULT_PORT,
		}
	}
}

pub enum SyncServerDatabaseResponse {
	UploadDatabase,
	DownloadDatabase,
}


async fn send_server_request(mut write_half: OwnedWriteHalf, request: &Request) -> ServerResult<()> {
	let json = serde_json::to_string(request)?;
	write_half.write_all(json.as_bytes()).await?;
	Ok(())
}

async fn read_server_response(mut read_half: OwnedReadHalf) -> ServerResult<Response> {
	let mut json = String::new();
	read_half.read_to_string(&mut json).await?;
	let response: Response = serde_json::from_str(&json)?;
	Ok(response)
}

pub async fn sync_database_from_server(config: ServerConfig, database_last_modified_date: ModifiedDate) -> ServerResult<SyncServerDatabaseResponse> {
	let stream = TcpStream::connect(format!("{}:{}", config.hostname, config.port)).await?;
	let (read_half, write_half) = stream.into_split();

	send_server_request(write_half, &Request::GetModifiedDate).await?;

	let server_modified_date = match read_server_response(read_half).await? {
		Response::ModifiedDate(date) => date,
		// should not send database when only asked for the modified date
		Response::Database { .. } => return Err(ServerError::InvalidResponse),
	};

	if server_modified_date > database_last_modified_date {
		Ok(SyncServerDatabaseResponse::DownloadDatabase)
	}
	else {
		Ok(SyncServerDatabaseResponse::UploadDatabase)
	}
}

pub async fn download_database_from_server(config: ServerConfig) -> ServerResult<Database> {
	let stream = TcpStream::connect(format!("{}:{}", config.hostname, config.port)).await?;
	let (read_half, write_half) = stream.into_split();

	send_server_request(write_half, &Request::DownloadDatabase).await?;

	let response = read_server_response(read_half).await?;
	match response {
		Response::Database{ database_json } => match serde_json::from_str(&database_json) {
			Ok(database) => Ok(database),
			Err(_) => Err(ServerError::InvalidResponse),
		},
		// should not send database when only asked for the modified date
		Response::ModifiedDate(_) => Err(ServerError::InvalidResponse),
	}
}

pub async fn upload_database_to_server(config: ServerConfig, database_json: String) -> ServerResult<()> {
	let stream = TcpStream::connect(format!("{}:{}", config.hostname, config.port)).await?;
	let (_read_half, write_half) = stream.into_split();
	send_server_request(write_half, &Request::UpdateDatabase { database_json }).await?;
	Ok(())
}