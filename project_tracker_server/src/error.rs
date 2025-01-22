use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq)]
pub enum ServerError {
	#[error("invalid password")]
	InvalidPassword,
	#[error("invalid database binary format")]
	InvalidDatabaseBinary,
	#[error("failed to parse request")]
	RequestParseError,
	#[error("failed to parse response")]
	ResponseParseError,
	#[error("failed to parse encrypted content")]
	EncryptedContentParseError,
	#[error("failed to serialize to binary")]
	FailedToSerializeToBinary,
	#[error("failed to encrypt binary")]
	FailedToEncryptBinary,
}

pub type ServerResult<T> = Result<T, ServerError>;
