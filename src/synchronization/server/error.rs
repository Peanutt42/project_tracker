use thiserror::Error;

use crate::synchronization::BaseSynchronizationError;

#[derive(Debug, Clone, Error)]
pub enum ServerSynchronizationError {
	#[error("server disconnected")]
	Disconnected,
	#[error("failed to connect to ws server: {0}")]
	ConnectToWsServer(String),
	#[error("failed to encrypt request: {0}")]
	EncryptRequest(String),
	#[error("failed to parse server response: {0}")]
	ParseServerResponse(String),
}

impl BaseSynchronizationError for ServerSynchronizationError {
	fn label(&self) -> &'static str {
		match self {
			Self::Disconnected => "Disconnected",
			Self::ConnectToWsServer(_) => "Connect to server",
			Self::EncryptRequest(_) => "Encrypt request",
			Self::ParseServerResponse(_) => "Failed parsing response",
		}
	}
}
