use crate::{
	components::{retry_synchronization_button, show_error_popup_button},
	project_tracker::Message,
	styles::{danger_text_style, SPACING_AMOUNT},
};
use filesystem::{FilesystemSynchronizationError, FilesystemSynchronizationMessage};
use iced::{
	widget::{container, row, text},
	Alignment, Element,
	Length::Fill,
	Subscription,
};
use project_tracker_core::{Database, DatabaseMessage};
use project_tracker_server::AdminInfos;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Debug;
use thiserror::Error;

mod filesystem;
pub use filesystem::{
	browse_filesystem_synchronization_filepath_dialog, FilesystemSynchronization,
};

mod server;
use crate::synchronization::server::{ServerSubscriptionMessage, ServerSynchronizationError};
pub use server::{ServerConfig, ServerSynchronization};

pub enum DatabaseUpdateEvent {
	DatabaseMessage(DatabaseMessage),
	ImportDatabase(Database),
}

/// gets called immediatly before every change
pub trait OnUpdateSynchronization {
	/// used to preset the database to sync, if the synchronization requires to know it
	/// before any db updates are done
	fn preset_database_to_sync(&mut self, database: &Database);

	/// Will update app with 'Message::SyncedDatabase' when synchronized
	fn before_database_update(
		&mut self,
		database: &Database,
		database_update_event: DatabaseUpdateEvent,
	) -> iced::Task<Message>;
}

/// synchronize is delayed and rate limited to 1 times per second
pub trait DelayedSynchronization {
	fn synchronize(&mut self, database: Option<&Database>) -> iced::Task<Message>;
}

pub trait BaseSynchronization: Clone + Serialize + DeserializeOwned {
	type Message;

	fn update(&mut self, _message: Self::Message) -> iced::Task<Message> {
		iced::Task::none()
	}

	fn subscription(&self) -> Subscription<Message>;

	fn view(&self, show_password: bool) -> Element<Message>;
}

pub trait BaseSynchronizationError: Debug + Error {
	fn label(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub enum SynchronizationMessage {
	ServerSynchronizationMessage(ServerSubscriptionMessage),
	FilesystemSynchronizationMessage(FilesystemSynchronizationMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Synchronization {
	FilesystemSynchronization(FilesystemSynchronization),
	ServerSynchronization(Box<ServerSynchronization>),
}

impl BaseSynchronization for Synchronization {
	type Message = SynchronizationMessage;

	fn update(&mut self, message: Self::Message) -> iced::Task<Message> {
		match message {
			SynchronizationMessage::ServerSynchronizationMessage(msg) => match self {
				Self::ServerSynchronization(server_synchronization) => {
					server_synchronization.update(msg)
				}
				_ => iced::Task::none(),
			},
			SynchronizationMessage::FilesystemSynchronizationMessage(msg) => match self {
				Self::FilesystemSynchronization(filesystem_synchronization) => {
					filesystem_synchronization.update(msg)
				}
				_ => iced::Task::none(),
			},
		}
	}

	fn subscription(&self) -> Subscription<Message> {
		match self {
			Self::FilesystemSynchronization(filesystem_synchronization) => {
				filesystem_synchronization.subscription()
			}
			Self::ServerSynchronization(server_synchronization) => {
				server_synchronization.subscription()
			}
		}
	}

	fn view(&self, show_password: bool) -> Element<Message> {
		match self {
			Synchronization::FilesystemSynchronization(filesystem_synchronization) => {
				filesystem_synchronization.view(show_password)
			}
			Synchronization::ServerSynchronization(server_synchronization) => {
				server_synchronization.view(show_password)
			}
		}
	}
}

impl OnUpdateSynchronization for Synchronization {
	fn preset_database_to_sync(&mut self, database: &Database) {
		match self {
			Self::FilesystemSynchronization(_filesystem_synchronization) => (),
			Self::ServerSynchronization(server_synchronization) => {
				server_synchronization.preset_database_to_sync(database);
			}
		}
	}

	fn before_database_update(
		&mut self,
		database: &Database,
		database_update_event: DatabaseUpdateEvent,
	) -> iced::Task<Message> {
		match self {
			Self::FilesystemSynchronization(_filesystem_synchronization) => iced::Task::none(),
			Self::ServerSynchronization(server_synchronization) => {
				server_synchronization.before_database_update(database, database_update_event)
			}
		}
	}
}

impl DelayedSynchronization for Synchronization {
	fn synchronize(&mut self, database: Option<&Database>) -> iced::Task<Message> {
		match self {
			Self::FilesystemSynchronization(filesystem_synchronization) => {
				filesystem_synchronization.synchronize(database)
			}
			Self::ServerSynchronization(_server_synchronization) => iced::Task::none(),
		}
	}
}

impl Synchronization {
	pub fn is_filesystem(&self) -> bool {
		matches!(self, Synchronization::FilesystemSynchronization(_))
	}
	pub fn is_server(&self) -> bool {
		matches!(self, Synchronization::ServerSynchronization(_))
	}
	pub fn latest_admin_infos(&self) -> Option<&AdminInfos> {
		match self {
			Synchronization::ServerSynchronization(server_synchronization) => {
				server_synchronization.latest_admin_infos.as_ref()
			}
			_ => None,
		}
	}
}

#[derive(Debug, Clone)]
pub enum SynchronizationOutput {
	DatabaseSaved,
	DatabaseLoaded(Database),
	DatabaseUpToDate,
}

#[derive(Debug, Error)]
pub enum SynchronizationError {
	#[error(transparent)]
	FilesystemSynchronizationError(#[from] FilesystemSynchronizationError),
	#[error(transparent)]
	ServerSynchronizationError(#[from] ServerSynchronizationError),
}

impl SynchronizationError {
	pub fn view(&self) -> Element<Message> {
		container(
			row![
				text(self.label()).style(danger_text_style),
				show_error_popup_button(format!("{self}")),
				retry_synchronization_button()
			]
			.align_y(Alignment::Center)
			.spacing(SPACING_AMOUNT),
		)
		.center_x(Fill)
		.into()
	}
}

impl BaseSynchronizationError for SynchronizationError {
	fn label(&self) -> &'static str {
		match self {
			Self::FilesystemSynchronizationError(e) => e.label(),
			Self::ServerSynchronizationError(e) => e.label(),
		}
	}
}
