use crate::synchronization::Synchronization;
use crate::{
	components::filepath_widget, project_tracker::Message, synchronization::BaseSynchronization,
};
use iced::{
	widget::{row, text, Space},
	Element,
	Length::Fill,
	Subscription,
};
use project_tracker_core::Database;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemSynchronization {
	filepath: PathBuf,
}

impl From<FilesystemSynchronization> for Synchronization {
	fn from(value: FilesystemSynchronization) -> Self {
		Self::FilesystemSynchronization(value)
	}
}

impl FilesystemSynchronization {
	pub fn new(filepath: PathBuf) -> Self {
		Self { filepath }
	}

	async fn synchronize_file(
		filepath: PathBuf,
		database_binary: Option<Vec<u8>>,
	) -> Result<(), FilesystemSynchronizationError> {
		match database_binary {
			Some(database_binary) => {
				if filepath.exists() {
					tokio::fs::write(&filepath, database_binary)
						.await
						.map_err(
							|io_error| FilesystemSynchronizationError::FailedToWriteToFile {
								filepath,
								io_error,
							},
						)
				} else {
					Err(FilesystemSynchronizationError::FileDoesNotExist { filepath })
				}
			}
			None => Err(FilesystemSynchronizationError::FailedToSerializeToBinary),
		}
	}
}

#[derive(Debug, Error)]
pub enum FilesystemSynchronizationError {
	#[error("failed to serialize to binary")]
	FailedToSerializeToBinary,
	#[error("file does not exist: {}", filepath.display())]
	FileDoesNotExist { filepath: PathBuf },
	#[error("could not write to file '{}', error: {io_error}", filepath.display())]
	FailedToWriteToFile {
		filepath: PathBuf,
		io_error: std::io::Error,
	},
}

impl FilesystemSynchronizationError {
	pub fn label(&self) -> &'static str {
		match self {
			Self::FailedToSerializeToBinary => "Serialization error",
			Self::FailedToWriteToFile { .. } => "File write error",
			Self::FileDoesNotExist { .. } => "File doesn't exist",
		}
	}
}

impl BaseSynchronization for FilesystemSynchronization {
	type Message = ();

	fn synchronize(&mut self, database: &Database) -> iced::Task<Message> {
		info!("synchronizing to {}", self.filepath.display());
		let filepath = self.filepath.clone();
		let database_binary = database.to_binary();
		iced::Task::perform(
			Self::synchronize_file(filepath, database_binary),
			|result| Message::SyncedDatabase(result.map_err(|e| Arc::new(e.into()))),
		)
	}

	fn subscription(&self) -> Subscription<Message> {
		Subscription::none()
	}

	fn view(&self, _show_password: bool) -> Element<Message> {
		row![
			text("File to synchronize:"),
			Space::new(Fill, 0.0),
			filepath_widget(self.filepath.clone())
		]
		.into()
	}
}

pub async fn browse_filesystem_synchronization_filepath_dialog() -> Option<FilesystemSynchronization>
{
	let file_dialog_result = rfd::AsyncFileDialog::new()
		.set_title("Set Filesystem Synchronization filepath")
		.add_filter(
			"Projecttracker Database (.project_tracker)",
			&["project_tracker"],
		)
		.set_file_name(Database::FILE_NAME)
		.pick_file()
		.await;

	file_dialog_result
		.map(|file_handle| FilesystemSynchronization::new(file_handle.path().to_path_buf()))
}
