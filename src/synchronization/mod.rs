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
use project_tracker_core::Database;
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
use crate::synchronization::server::{ServerSynchronizationError, ServerSynchronizationMessage};
pub use server::{ServerConfig, ServerSynchronization};

pub trait BaseSynchronization: Clone + Serialize + DeserializeOwned {
	type Message;

	/// Will update app with 'Message::SyncedDatabase' when synchronized
	fn synchronize(&mut self, database: Option<&Database>) -> iced::Task<Message>;

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
	ServerSynchronizationMessage(ServerSynchronizationMessage),
	FilesystemSynchronizationMessage(FilesystemSynchronizationMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Synchronization {
	FilesystemSynchronization(FilesystemSynchronization),
	ServerSynchronization(ServerSynchronization),
}

impl BaseSynchronization for Synchronization {
	type Message = SynchronizationMessage;

	fn synchronize(&mut self, database: Option<&Database>) -> iced::Task<Message> {
		match self {
			Self::FilesystemSynchronization(filesystem_synchronization) => {
				filesystem_synchronization.synchronize(database)
			}
			Self::ServerSynchronization(server_synchronization) => {
				server_synchronization.synchronize(database)
			}
		}
	}

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
