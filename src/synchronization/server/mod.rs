mod subscription;
pub use subscription::{ws_subscription, ServerSubscriptionMessage, ServerSynchronizationEvent};

mod error;
pub use error::ServerSynchronizationError;

use crate::components::{hide_password_button, show_password_button};
use crate::modals::settings_modal;
use crate::project_tracker::Message;
use crate::styles::{text_input_style_default, SPACING_AMOUNT};
use crate::synchronization::{
	BaseSynchronization, DatabaseUpdateEvent, OnUpdateSynchronization, Synchronization,
	SynchronizationOutput,
};
use flume::Sender;
use iced::alignment::Vertical;
use iced::widget::{column, container, row, text_input, toggler};
use iced::{Element, Subscription, Task};
use project_tracker_core::Database;
use project_tracker_server::{AdminInfos, Request, Response, DEFAULT_HOSTNAME, DEFAULT_PASSWORD};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::sync::Arc;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct ServerConfig {
	pub hostname: String,
	pub password: String,
	pub self_signed_certificate: bool,
}

impl Default for ServerConfig {
	fn default() -> Self {
		Self {
			hostname: DEFAULT_HOSTNAME.to_string(),
			password: DEFAULT_PASSWORD.to_string(),
			self_signed_certificate: true,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSynchronization {
	#[serde(flatten)]
	pub config: ServerConfig,
	#[serde(skip)]
	request_sender: Option<Sender<Request>>,
	#[serde(skip)]
	database_to_sync: Option<Database>,
	#[serde(skip)]
	pub latest_admin_infos: Option<AdminInfos>,
}

impl ServerSynchronization {
	pub fn new(config: ServerConfig) -> ServerSynchronization {
		Self {
			config,
			request_sender: None,
			database_to_sync: None,
			latest_admin_infos: None,
		}
	}
}

impl Default for ServerSynchronization {
	fn default() -> Self {
		Self::new(ServerConfig::default())
	}
}

impl From<ServerSynchronization> for Synchronization {
	fn from(value: ServerSynchronization) -> Self {
		Self::ServerSynchronization(Box::new(value))
	}
}

impl OnUpdateSynchronization for ServerSynchronization {
	fn before_database_update(
		&mut self,
		database: &Database,
		database_update_event: DatabaseUpdateEvent,
	) -> iced::Task<Message> {
		self.database_to_sync = Some(database.clone());

		let request = match database_update_event {
			DatabaseUpdateEvent::DatabaseMessage(database_message) => {
				let database_before_update_checksum = database.checksum();
				Request::UpdateDatabase {
					database_messages: vec![database_message],
					database_before_update_checksum,
				}
			}
			DatabaseUpdateEvent::ImportDatabase(database) => Request::ImportDatabase {
				database: database.into_serialized(),
			},
		};

		self.send_request(request);
		Task::none()
	}
}

impl BaseSynchronization for ServerSynchronization {
	type Message = ServerSubscriptionMessage;

	fn update(&mut self, message: ServerSubscriptionMessage) -> iced::Task<Message> {
		match message {
			Ok(event) => match event {
				ServerSynchronizationEvent::RequestSender(request_sender) => {
					self.request_sender = Some(request_sender);
					Task::none()
				}
				ServerSynchronizationEvent::Connected => {
					info!("ws connected!");
					self.send_request(match &self.database_to_sync {
						Some(database) => Request::CheckUpToDate {
							database_checksum: database.checksum(),
						},
						None => {
							warn!("no database set to sync -> requesting full db from server");
							Request::GetFullDatabase
						}
					});
					Task::none()
				}
				ServerSynchronizationEvent::Disconnected => {
					warn!("ws disconected!");
					Task::done(Message::SyncedDatabase(Err(Arc::new(
						ServerSynchronizationError::Disconnected.into(),
					))))
				}
				ServerSynchronizationEvent::Response(response) => {
					self.handle_server_response(response)
				}
			},
			Err(e) => Task::done(Message::SyncedDatabase(Err(Arc::new(e.clone().into())))),
		}
	}

	fn subscription(&self) -> Subscription<Message> {
		ws_subscription(self.config.clone())
			.map(|message| Message::SynchronizationMessage(message.into()))
	}

	fn view(&self, show_password: bool) -> Element<Message> {
		row![column![
			row![
				container("Hostname: ").width(200.0),
				text_input("ex. 127.0.0.1 or raspberrypi.local", &self.config.hostname)
					.on_input(
						|hostname| settings_modal::Message::SetServerHostname(hostname).into()
					)
					.width(250)
					.style(text_input_style_default),
			]
			.align_y(Vertical::Center),
			row![
				container("Password: ").width(200.0),
				if show_password {
					row![
						text_input(
							format!("default: {}", DEFAULT_PASSWORD).as_str(),
							&self.config.password
						)
						.on_input(
							|password| settings_modal::Message::SetServerPassword(password).into()
						)
						.width(250)
						.style(text_input_style_default),
						hide_password_button(),
					]
					.align_y(Vertical::Center)
					.spacing(SPACING_AMOUNT)
					.into()
				} else {
					show_password_button()
				},
			]
			.align_y(Vertical::Center),
			row![
				container("Self Signed Certificate: ").width(200.0),
				toggler(self.config.self_signed_certificate)
					.on_toggle(|enabled| {
						settings_modal::Message::SetServerSelfSignedCertificate(enabled).into()
					})
					.size(27.5),
			]
			.align_y(Vertical::Center),
		]
		.spacing(SPACING_AMOUNT)]
		.spacing(SPACING_AMOUNT)
		.into()
	}
}

impl ServerSynchronization {
	pub fn send_request(&mut self, request: Request) {
		match &mut self.request_sender {
			Some(request_sender) => {
				let _ = request_sender.send(request);
			}
			None => {
				error!("cant send request since request sender is not set\nrequest: {request:#?}")
			}
		}
	}

	fn handle_server_response(&mut self, response: Response) -> iced::Task<Message> {
		match response {
			Response::MoreUpToDateDatabase {
				database,
				last_modified_time,
			} => Task::done(Message::SyncedDatabase(Ok(
				SynchronizationOutput::DatabaseLoaded(Database::from_serialized(
					database,
					last_modified_time,
				)),
			))),
			Response::DatabaseChanged {
				database_before_update_checksum,
				database_messages,
			} => match &mut self.database_to_sync {
				Some(database) if database.checksum() == database_before_update_checksum => {
					for database_message in database_messages {
						database.update(database_message);
					}
					Task::done(Message::SyncedDatabase(Ok(
						SynchronizationOutput::DatabaseLoaded(database.clone()),
					)))
				}
				_ => {
					warn!("db changed on server and we are not up to date -> requesting full db");
					self.send_request(Request::GetFullDatabase);
					Task::none()
				}
			},
			Response::DatabaseUpdated => Task::done(Message::SyncedDatabase(Ok(
				SynchronizationOutput::DatabaseSaved,
			))),
			Response::DatabaseUpToDate => Task::done(Message::SyncedDatabase(Ok(
				SynchronizationOutput::DatabaseUpToDate,
			))),
			Response::AdminInfos(admin_infos) => {
				self.latest_admin_infos = Some(admin_infos);
				Task::none()
			}
		}
	}
}
