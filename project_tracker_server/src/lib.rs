use chrono::{DateTime, Utc};
use humantime::format_duration;
use project_tracker_core::{
	get_last_modification_date_time, Database, DatabaseMessage, SerializedDatabase,
};
use serde::{Deserialize, Serialize};
use std::{
	collections::HashSet,
	net::SocketAddr,
	path::PathBuf,
	sync::{
		atomic::{AtomicU32, Ordering},
		Arc, RwLock,
	},
};
use systemstat::{saturating_sub_bytes, Platform, System};
use thiserror::Error;

mod server;
pub use server::{handle_client, run_server};

mod encryption;
pub use encryption::{decrypt, encrypt, NONCE_LENGTH, SALT_LENGTH};

mod logs;
pub use logs::get_logs_as_string;

pub const DEFAULT_HOSTNAME: &str = "127.0.0.1";
pub const DEFAULT_PORT: usize = 8080;
pub const DEFAULT_PASSWORD: &str = "1234";

#[derive(Debug, Error)]
pub enum ServerError {
	#[error("connection failed with server: {0}")]
	ConnectionError(#[from] std::io::Error),
	#[error("invalid response from server")]
	InvalidResponse,
	#[error("{0}")]
	ResponseError(#[from] ResponseError),
}

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Request {
	pub fn decrypt(binary: Vec<u8>, password: &str) -> ServerResult<Self> {
		let encrypted_message: EncryptedMessage =
			bincode::deserialize(&binary).map_err(|_| ResponseError::ParseError)?;
		let request_binary = encrypted_message.decrypt(password)?;
		Ok(bincode::deserialize(&request_binary).map_err(|_| ResponseError::ParseError)?)
	}

	pub fn encrypt(&self, password: &str) -> ServerResult<Vec<u8>> {
		let request_binary = bincode::serialize(self).map_err(|_| ResponseError::ParseError)?;

		Ok(
			bincode::serialize(&EncryptedMessage::new(&request_binary, password)?)
				.map_err(|_| ResponseError::ParseError)?,
		)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptedResponse {
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
impl EncryptedResponse {
	pub fn decrypt(encrypted: EncryptedMessage, password: &str) -> ServerResult<Self> {
		let response_binary = encrypted.decrypt(password)?;
		Ok(bincode::deserialize(&response_binary).map_err(|_| ResponseError::ParseError)?)
	}

	pub fn encrypt(&self, password: &str) -> ServerResult<EncryptedMessage> {
		let response_binary = bincode::serialize(self).map_err(|_| ResponseError::ParseError)?;

		EncryptedMessage::new(&response_binary, password)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum ResponseError {
	#[error("invalid password")]
	InvalidPassword,
	#[error("invalid database binary format")]
	InvalidDatabaseBinary,
	#[error("failed to parse request")]
	ParseError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response(pub Result<EncryptedMessage, ResponseError>);

impl Response {
	pub fn serialize(&self) -> ServerResult<Vec<u8>> {
		Ok(bincode::serialize(&self.0).map_err(|_| ResponseError::ParseError)?)
	}

	pub fn deserialize(binary: Vec<u8>) -> ServerResult<Self> {
		let result = bincode::deserialize(&binary).map_err(|_| ResponseError::ParseError)?;
		Ok(Self(result))
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
	encrypted_message: Vec<u8>,
	salt: [u8; SALT_LENGTH],
	nonce: [u8; NONCE_LENGTH],
}

impl EncryptedMessage {
	pub fn new(plaintext_message: &[u8], password: &str) -> ServerResult<Self> {
		let (encrypted_message, salt, nonce) =
			encrypt(plaintext_message, password).map_err(|_| ResponseError::InvalidPassword)?;

		Ok(Self {
			encrypted_message,
			salt,
			nonce,
		})
	}

	pub fn decrypt(&self, password: &str) -> ServerResult<Vec<u8>> {
		let bytes = decrypt(&self.encrypted_message, password, &self.salt, &self.nonce)
			.map_err(|_| ResponseError::InvalidPassword)?;
		Ok(bytes)
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminInfos {
	pub connected_native_gui_clients: Vec<SocketAddr>,
	pub connected_web_clients: Vec<SocketAddr>,
	pub cpu_usage: f32,
	pub cpu_temp: Option<f32>,
	pub ram_info: String,
	pub uptime: String,
	pub latest_logs_of_the_day: String,
}

impl AdminInfos {
	pub fn generate(
		connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
		cpu_usage_avg: &CpuUsageAverage,
		log_filepath: &PathBuf,
	) -> Self {
		let connected_clients = connected_clients.read().unwrap().clone();

		let mut connected_native_gui_clients = Vec::new();
		let mut connected_web_clients = Vec::new();

		for connected_client in connected_clients {
			match connected_client {
				ConnectedClient::NativeGUI(addr) => connected_native_gui_clients.push(addr),
				ConnectedClient::Web(addr) => connected_web_clients.push(addr),
			}
		}

		let sys = System::new();

		let cpu_temp = sys.cpu_temp().ok();

		let ram_info = match sys.memory() {
			Ok(mem) => format!(
				"{} / {}",
				saturating_sub_bytes(mem.total, mem.free),
				mem.total
			),
			Err(_) => "failed to get ram info".to_string(),
		};

		let uptime = match sys.uptime() {
			Ok(uptime) => format_duration(uptime).to_string(),
			Err(_) => "failed to get uptime".to_string(),
		};

		let latest_logs_of_the_day = match get_logs_as_string(log_filepath) {
			Ok(logs) => logs,
			Err(error_str) => error_str,
		};

		AdminInfos {
			connected_native_gui_clients,
			connected_web_clients,
			cpu_usage: cpu_usage_avg.load(),
			cpu_temp,
			ram_info,
			uptime,
			latest_logs_of_the_day,
		}
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

/// number is directly the usage percent number
///
/// `AtomicU32` is used to atomically store a f32 (`f32::as_bits`, `f32::to_bits`)
#[derive(Debug, Default)]
pub struct CpuUsageAverage(pub AtomicU32);

impl CpuUsageAverage {
	pub fn new() -> CpuUsageAverage {
		Self(AtomicU32::new(0))
	}

	pub fn load(&self) -> f32 {
		f32::from_bits(self.0.load(Ordering::Relaxed))
	}

	fn store(&self, percentage: f32) {
		self.0.store(percentage.to_bits(), Ordering::Relaxed);
	}
}

pub async fn messure_cpu_usage_avg_thread(cpu_usage_avg: Arc<CpuUsageAverage>) {
	let sys = System::new();
	loop {
		if let Ok(cpu_load) = sys.cpu_load_aggregate() {
			tokio::time::sleep(std::time::Duration::from_secs(2)).await;
			if let Ok(cpu_load) = cpu_load.done() {
				cpu_usage_avg.store(1.0 - cpu_load.idle);
			}
		}
	}
}
