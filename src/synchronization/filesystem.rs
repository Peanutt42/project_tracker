use crate::{
	components::{filepath_widget, select_synchronization_filepath_button},
	project_tracker::Message,
	styles::SPACING_AMOUNT,
	synchronization::{
		BaseSynchronization, DelayedSynchronization, Synchronization, SynchronizationError,
		SynchronizationMessage, SynchronizationOutput,
	},
};
use async_watcher::{
	notify::{Event, RecursiveMode},
	AsyncDebouncer,
};
use chrono::{DateTime, Utc};
use iced::{
	alignment::Horizontal,
	futures::{SinkExt, Stream},
	stream,
	widget::{column, container},
};
use iced::{
	widget::{row, text, Space},
	Element,
	Length::Fill,
	Subscription,
};
use project_tracker_core::{get_last_modification_date_time, Database, LoadDatabaseError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::{
	hash::{Hash, Hasher},
	time::Duration,
};
use thiserror::Error;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct FilesystemSynchronization {
	filepath: PathBuf,
	#[serde(skip)]
	last_write_datetime: Option<DateTime<Utc>>,
}

impl From<FilesystemSynchronization> for Synchronization {
	fn from(value: FilesystemSynchronization) -> Self {
		Self::FilesystemSynchronization(value)
	}
}

impl FilesystemSynchronization {
	pub fn new(filepath: PathBuf) -> Self {
		Self {
			filepath,
			last_write_datetime: None,
		}
	}

	async fn synchronize_file(
		filepath: PathBuf,
		database_binary_and_last_write_time: Option<(Vec<u8>, DateTime<Utc>)>,
	) -> Result<SynchronizationOutput, FilesystemSynchronizationError> {
		match database_binary_and_last_write_time {
			Some((database_binary, last_write_time)) => {
				if filepath.exists() {
					let last_file_modification_time = filepath
						.metadata()
						.ok()
						.and_then(|metadata| get_last_modification_date_time(&metadata));

					match last_file_modification_time {
						Some(last_file_modification_time) => {
							if last_file_modification_time > last_write_time {
								match Database::load(filepath).await {
									Ok(database) => {
										Ok(SynchronizationOutput::DatabaseLoaded(database))
									}
									Err(e) => {
										Err(FilesystemSynchronizationError::LoadDatabaseError(e))
									}
								}
							} else {
								tokio::fs::write(&filepath, database_binary)
									.await
									.map_err(|io_error| {
										FilesystemSynchronizationError::FailedToWriteToFile {
											filepath,
											io_error,
										}
									})
									.map(|_| SynchronizationOutput::DatabaseSaved)
							}
						}
						None => Err(FilesystemSynchronizationError::FileDoesNotExist { filepath }),
					}
				} else {
					Err(FilesystemSynchronizationError::FileDoesNotExist { filepath })
				}
			}
			// since we dont have any database --> load any we get
			None => match Database::load(filepath).await {
				Ok(database) => Ok(SynchronizationOutput::DatabaseLoaded(database)),
				Err(e) => Err(FilesystemSynchronizationError::LoadDatabaseError(e)),
			},
		}
	}

	fn get_file_last_modification_time(&self) -> Option<DateTime<Utc>> {
		self.filepath
			.metadata()
			.ok()
			.and_then(|metadata| get_last_modification_date_time(&metadata))
	}
}

#[derive(Debug, Error)]
pub enum FilesystemSynchronizationError {
	#[error("file does not exist: {}", filepath.display())]
	FileDoesNotExist { filepath: PathBuf },
	#[error("could not write to file '{}', error: {io_error}", filepath.display())]
	FailedToWriteToFile {
		filepath: PathBuf,
		io_error: std::io::Error,
	},
	#[error(transparent)]
	FileWatcherError(#[from] FilesystemWatcherError),
	#[error(transparent)]
	LoadDatabaseError(LoadDatabaseError),
}

impl FilesystemSynchronizationError {
	pub fn label(&self) -> &'static str {
		match self {
			Self::FailedToWriteToFile { .. } => "File write error",
			Self::FileDoesNotExist { .. } => "File doesn't exist",
			Self::FileWatcherError(_) => "File watcher error",
			Self::LoadDatabaseError(_) => "Load Database error",
		}
	}
}

impl DelayedSynchronization for FilesystemSynchronization {
	fn synchronize(&mut self, database: Option<&Database>) -> iced::Task<Message> {
		info!("synchronizing to {}", self.filepath.display());
		let filepath = self.filepath.clone();
		let database_binary_and_last_write_time = database.and_then(|db| {
			db.to_binary()
				.map(|binary| (binary, *db.last_changed_time()))
		});
		iced::Task::perform(
			Self::synchronize_file(filepath, database_binary_and_last_write_time),
			|result| {
				Message::SynchronizationMessage(
					FilesystemSynchronizationMessage::Synced(
						result.map_err(|e| Arc::new(e.into())),
					)
					.into(),
				)
			},
		)
	}
}

impl BaseSynchronization for FilesystemSynchronization {
	type Message = FilesystemSynchronizationMessage;

	fn update(&mut self, message: Self::Message) -> iced::Task<Message> {
		match message {
			Self::Message::Synced(result) => {
				if result.is_ok() {
					match self.get_file_last_modification_time() {
						Some(last_write_datetime) => {
							self.last_write_datetime = Some(last_write_datetime);
						}
						None => error!(
							"failed to get last modification time of file: {}",
							self.filepath.display()
						),
					}
				}
				iced::Task::done(Message::SyncedDatabase(result))
			}
			Self::Message::Event(event) => {
				if event.kind.is_modify() {
					if let Some(file_last_modification_time) =
						self.get_file_last_modification_time()
					{
						if self.last_write_datetime == Some(file_last_modification_time) {
							// this means that we were the one writing to the file
							// --> no action needed
							// For some reason this never gets called tho... (no event gets send)
						} else {
							info!("synchronization file changed --> synchronizing...");
							return iced::Task::perform(
								Database::load(self.filepath.clone()),
								|result| match result {
									Ok(database) => Message::SyncedDatabase(Ok(
										SynchronizationOutput::DatabaseLoaded(database),
									)),
									Err(e) => Message::SyncedDatabase(Err(Arc::new(
										FilesystemSynchronizationError::LoadDatabaseError(e).into(),
									))),
								},
							);
						}
					}
				}

				iced::Task::none()
			}
			Self::Message::FilesystemWatcherError(error) => {
				iced::Task::done(Message::SyncedDatabase(Err(Arc::new(
					FilesystemSynchronizationError::FileWatcherError(error).into(),
				))))
			}
		}
	}

	fn subscription(&self) -> Subscription<Message> {
		let config_hash = {
			let mut hasher = std::hash::DefaultHasher::default();
			self.hash(&mut hasher);
			hasher.finish()
		};
		Subscription::run_with_id(
			config_hash,
			watch_file_modification_subscription(self.filepath.clone()),
		)
		.map(|msg| Message::SynchronizationMessage(msg.into()))
	}

	fn view(&self, _show_password: bool) -> Element<Message> {
		column![
			row![
				text("File to synchronize:"),
				Space::new(Fill, 0.0),
				filepath_widget(self.filepath.clone()),
			]
			.spacing(SPACING_AMOUNT),
			container(select_synchronization_filepath_button(),)
				.width(Fill)
				.align_x(Horizontal::Right)
		]
		.spacing(SPACING_AMOUNT)
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

#[derive(Debug, Clone)]
pub enum FilesystemSynchronizationMessage {
	Synced(Result<SynchronizationOutput, Arc<SynchronizationError>>),
	Event(Event),
	FilesystemWatcherError(FilesystemWatcherError),
}

impl From<FilesystemSynchronizationMessage> for SynchronizationMessage {
	fn from(message: FilesystemSynchronizationMessage) -> Self {
		SynchronizationMessage::FilesystemSynchronizationMessage(message)
	}
}

#[derive(Debug, Error, Clone)]
pub enum FilesystemWatcherError {
	#[error("failed to create file watcher")]
	FailedToCreate,
	#[error("failed to start file watcher")]
	FailedToStart,
	#[error("file watcher quit unexpectedly")]
	QuitUnexpectedly,
}

fn watch_file_modification_subscription(
	filepath: PathBuf,
) -> impl Stream<Item = FilesystemSynchronizationMessage> {
	stream::channel(100, |mut output| async move {
		let result = AsyncDebouncer::new_with_channel(
			Duration::from_millis(200),
			Some(Duration::from_millis(200)),
		)
		.await;

		match result {
			Ok((mut debouncer, mut file_events)) => {
				if let Err(e) = debouncer
					.watcher()
					.watch(filepath.as_path(), RecursiveMode::NonRecursive)
				{
					error!("failed to start watching file: {e}");
					let _ = output
						.send(FilesystemSynchronizationMessage::FilesystemWatcherError(
							FilesystemWatcherError::FailedToStart,
						))
						.await;
					return;
				}
				while let Some(Ok(events)) = file_events.recv().await {
					for event in events {
						if output
							.send(FilesystemSynchronizationMessage::Event(event.event))
							.await
							.is_err()
						{
							return;
						}
					}
				}
				let error = FilesystemWatcherError::QuitUnexpectedly;
				error!("{error}");
				let _ = output
					.send(FilesystemSynchronizationMessage::FilesystemWatcherError(
						error,
					))
					.await;
			}
			Err(e) => {
				error!("failed to create file watcher: {e}");
				let _ = output
					.send(FilesystemSynchronizationMessage::FilesystemWatcherError(
						FilesystemWatcherError::FailedToCreate,
					))
					.await
					.is_err();
			}
		}
	})
}
