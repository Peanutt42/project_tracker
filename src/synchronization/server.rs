use crate::components::{hide_password_button, show_password_button};
use crate::modals::SettingsModalMessage;
use crate::project_tracker::Message;
use crate::styles::{text_input_style_default, SPACING_AMOUNT};
use crate::synchronization::{
	BaseSynchronization, BaseSynchronizationError, Synchronization, SynchronizationMessage,
};
use async_tungstenite::tungstenite;
use iced::alignment::Vertical;
use iced::futures::channel::mpsc;
use iced::futures::{self, SinkExt, Stream, StreamExt};
use iced::widget::{column, container, row, text_input};
use iced::{stream, Element, Subscription, Task};
use project_tracker_core::Database;
use project_tracker_server::{
	AdminInfos, EncryptedResponse, Request, Response, DEFAULT_HOSTNAME, DEFAULT_PASSWORD,
	DEFAULT_PORT,
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSynchronization {
	#[serde(flatten)]
	pub config: ServerConfig,
	#[serde(skip)]
	request_sender: Option<ServerSynchronizationRequestSender>,
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
		Self::ServerSynchronization(value)
	}
}

impl BaseSynchronization for ServerSynchronization {
	type Message = ServerSynchronizationMessage;

	fn synchronize(&mut self, database: &Database) -> Task<Message> {
		match &mut self.request_sender {
			Some(request_sender) => {
				self.database_to_sync = Some(database.clone());
				let _ = request_sender.send(Request::GetModifiedDate);
			}
			None => warn!("tried to synchronize but no request sender set yet!"),
		}
		Task::none()
	}

	fn update(&mut self, message: ServerSynchronizationMessage) -> iced::Task<Message> {
		match message {
			Ok(event) => match event {
				ServerSynchronizationEvent::RequestSender(request_sender) => {
					self.request_sender = Some(request_sender);
					Task::none()
				}
				ServerSynchronizationEvent::Connected => {
					info!("ws connected!");
					if let Some(request_sender) = &mut self.request_sender {
						let _ = request_sender.send(Request::GetModifiedDate);
					}
					Task::none()
				}
				ServerSynchronizationEvent::Disconnected => {
					tracing::warn!("ws disconected!");
					Task::done(Message::SyncedDatabase(Err(Arc::new(
						ServerSynchronizationError::Disconnected.into(),
					))))
				}
				ServerSynchronizationEvent::Response { response, password } => match response.0 {
					Ok(encrypted_response) => {
						match EncryptedResponse::decrypt(encrypted_response, &password) {
							Ok(encrypted_response) => {
								self.handle_encrypted_server_response(encrypted_response)
							}
							Err(e) => {
								let e_str = format!("{e}");
								error!("{e_str}");
								Task::done(Message::SyncedDatabase(Err(Arc::new(
									ServerSynchronizationError::EncryptRequest(e_str).into(),
								))))
							}
						}
					}
					Err(e) => {
						let e_str = format!("{e}");
						error!("{e_str}");
						Task::done(Message::SyncedDatabase(Err(Arc::new(
							ServerSynchronizationError::ParseServerResponse(e_str).into(),
						))))
					}
				},
			},
			Err(e) => Task::done(Message::SyncedDatabase(Err(Arc::new(e.clone().into())))),
		}
	}

	fn subscription(&self) -> Subscription<Message> {
		// if the server config changes --> new hash --> new subscription id --> reconnect
		let server_config_hash = {
			let mut hasher = std::hash::DefaultHasher::new();
			self.config.hash(&mut hasher);
			hasher.finish()
		};
		Subscription::run_with_id(server_config_hash, connect_ws(self.config.clone()))
			.map(|message| Message::SynchronizationMessage(message.into()))
	}

	fn view(&self, show_password: bool) -> Element<Message> {
		row![column![
			row![
				container("Hostname: ").width(100.0),
				text_input("ex. 127.0.0.1 or raspberrypi.local", &self.config.hostname)
					.on_input(|hostname| SettingsModalMessage::SetServerHostname(hostname).into())
					.style(text_input_style_default),
			]
			.align_y(Vertical::Center),
			row![
				container("Port: ").width(100.0),
				text_input("ex. 8080", &format!("{}", self.config.port))
					.on_input(|input| {
						let new_port = match usize::from_str(&input) {
							Ok(new_port) => Some(new_port),
							Err(_) => {
								if input.is_empty() {
									Some(8080)
								} else {
									None
								}
							}
						};
						match new_port {
							Some(new_port) => SettingsModalMessage::SetServerPort(new_port).into(),
							None => SettingsModalMessage::InvalidPortInput.into(),
						}
					})
					.style(text_input_style_default)
					.width(55.0),
			]
			.align_y(Vertical::Center),
			row![
				container("Password: ").width(100.0),
				if show_password {
					row![
						text_input(
							format!("default: {}", DEFAULT_PASSWORD).as_str(),
							&self.config.password
						)
						.on_input(
							|password| SettingsModalMessage::SetServerPassword(password).into()
						)
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
		]
		.spacing(SPACING_AMOUNT)]
		.spacing(SPACING_AMOUNT)
		.into()
	}
}

impl ServerSynchronization {
	pub fn request_admin_infos(&mut self) {
		if let Some(request_sender) = &mut self.request_sender {
			let _ = request_sender.send(Request::AdminInfos);
		}
	}

	fn handle_encrypted_server_response(
		&mut self,
		encrypted_response: EncryptedResponse,
	) -> iced::Task<Message> {
		match encrypted_response {
			EncryptedResponse::Database {
				database,
				last_modified_time,
			} => {
				let server_is_more_up_to_date = self
					.database_to_sync
					.as_ref()
					.map(|db| last_modified_time > *db.last_changed_time())
					.unwrap_or(true);

				if server_is_more_up_to_date {
					Task::done(Message::LoadedDatabase(Ok(Database::from_serialized(
						database,
						last_modified_time,
					))))
				} else {
					Task::none()
				}
			}
			EncryptedResponse::DatabaseUpdated => Task::done(Message::SyncedDatabase(Ok(()))),
			EncryptedResponse::ModifiedDate(server_modified_date) => {
				if let Some(request_sender) = &mut self.request_sender {
					let server_is_more_up_to_date = self
						.database_to_sync
						.as_ref()
						.map(|db| server_modified_date > *db.last_changed_time())
						.unwrap_or(true);

					if server_is_more_up_to_date {
						let _ = request_sender.send(Request::DownloadDatabase);
					} else if let Some(database) = self.database_to_sync.clone() {
						let last_modified_time = *database.last_changed_time();
						let _ = request_sender.send(Request::UpdateDatabase {
							database: database.into_serialized(),
							last_modified_time,
						});
					}
				}
				Task::none()
			}
			EncryptedResponse::AdminInfos(admin_infos) => {
				self.latest_admin_infos = Some(admin_infos);
				Task::none()
			}
		}
	}
}

pub type ServerSynchronizationMessage =
	Result<ServerSynchronizationEvent, ServerSynchronizationError>;

impl From<ServerSynchronizationMessage> for SynchronizationMessage {
	fn from(message: ServerSynchronizationMessage) -> Self {
		SynchronizationMessage::ServerSynchronizationMessage(message)
	}
}

fn connect_ws(config: ServerConfig) -> impl Stream<Item = ServerSynchronizationMessage> {
	stream::channel(100, |mut output| async move {
		let mut state = ServerConnectionState::new();
		let mut connection = ServerConnection::Disconnected;

		while state.update(&mut connection, &config, &mut output).await {}
	})
}

#[derive(Debug, Clone)]
pub enum ServerSynchronizationEvent {
	RequestSender(ServerSynchronizationRequestSender),
	Connected,
	Disconnected,
	Response {
		response: Response,
		password: String,
	},
}

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

type WebSocketStream = async_tungstenite::WebSocketStream<async_tungstenite::tokio::ConnectStream>;

#[derive(Debug)]
enum ServerConnection {
	Disconnected,
	Connecting,
	Connected(WebSocketStream),
}

struct ServerConnectionState {
	request_receiver: Option<mpsc::UnboundedReceiver<Request>>,
	request_sender_sent: bool,
}

impl ServerConnectionState {
	fn new() -> Self {
		Self {
			request_receiver: None,
			request_sender_sent: false,
		}
	}

	// returns whether to continue or to quit
	async fn update(
		&mut self,
		connection: &mut ServerConnection,
		config: &ServerConfig,
		output: &mut mpsc::Sender<ServerSynchronizationMessage>,
	) -> bool {
		match connection {
			ServerConnection::Disconnected => {
				if !self.request_sender_sent {
					let (sender, receiver) = mpsc::unbounded();

					self.request_receiver = Some(receiver);

					if output
						.send(Ok(ServerSynchronizationEvent::RequestSender(
							ServerSynchronizationRequestSender(sender),
						)))
						.await
						.is_ok()
					{
						self.request_sender_sent = true;
					} else {
						return false;
					}
				}

				*connection = ServerConnection::Connecting;

				true
			}
			ServerConnection::Connecting => match self.connect(output, connection, config).await {
				Ok(continue_subscription) => continue_subscription,
				Err(e) => {
					error!("failed to connect to ws: {e}");
					if output
						.send(Err(ServerSynchronizationError::ConnectToWsServer(format!(
							"{e}"
						))))
						.await
						.is_err()
					{
						return false;
					}
					if output
						.send(Ok(ServerSynchronizationEvent::Disconnected))
						.await
						.is_err()
					{
						return false;
					}
					tokio::time::sleep(std::time::Duration::from_secs(1)).await;
					true
				}
			},
			ServerConnection::Connected(websocket) => {
				let (continue_subscription, new_connection_state) =
					self.listen(websocket, config.clone(), output).await;
				if let Some(new_connection_state) = new_connection_state {
					*connection = new_connection_state;
				}
				continue_subscription
			}
		}
	}

	async fn connect(
		&mut self,
		output: &mut mpsc::Sender<ServerSynchronizationMessage>,
		connection: &mut ServerConnection,
		config: &ServerConfig,
	) -> Result<bool, async_tungstenite::tungstenite::Error> {
		let address = format!("ws://{}:{}", config.hostname, config.port);

		let (webserver, _) = async_tungstenite::tokio::connect_async(address).await?;

		if output
			.send(Ok(ServerSynchronizationEvent::Connected))
			.await
			.is_err()
		{
			return Ok(false);
		}
		*connection = ServerConnection::Connected(webserver);
		Ok(true)
	}

	async fn listen(
		&mut self,
		websocket: &mut WebSocketStream,
		config: ServerConfig,
		output: &mut mpsc::Sender<ServerSynchronizationMessage>,
	) -> (bool, Option<ServerConnection>) {
		let mut fused_websocket = websocket.by_ref().fuse();

		match &mut self.request_receiver {
			Some(request_receiver) => futures::select! {
				received = fused_websocket.select_next_some() => {
					match received {
						Ok(tungstenite::Message::Binary(binary)) => (
							output.send(Self::parse_server_response(binary, config.password.clone())).await.is_ok(),
							None
						),
						Ok(tungstenite::Message::Close(_)) => (
							output.send(Ok(ServerSynchronizationEvent::Disconnected)).await.is_ok(),
							Some(ServerConnection::Disconnected)
						),
						Err(e) => {
							info!("ws server disconnected: {e}");
							(
								output.send(Ok(ServerSynchronizationEvent::Disconnected)).await.is_ok(),
								Some(ServerConnection::Disconnected)
							)
						}
						_ => (true, None),
					}
				},

				request = request_receiver.select_next_some() => {
					let (opt_event_result, opt_new_connection) = Self::send_request(request, &config.password, websocket).await;
					let continue_subscription = match opt_event_result {
						Some(event_result) => output.send(event_result).await.is_ok(),
						None => true,
					};

					(
						continue_subscription,
						opt_new_connection,
					)
				},
			},
			None => {
				self.request_sender_sent = false;
				(true, Some(ServerConnection::Disconnected))
			}
		}
	}

	fn parse_server_response(
		binary_response: Vec<u8>,
		password: String,
	) -> ServerSynchronizationMessage {
		match Response::deserialize(binary_response) {
			Ok(response) => Ok(ServerSynchronizationEvent::Response { response, password }),
			Err(e) => Err(ServerSynchronizationError::ParseServerResponse(format!(
				"{e}"
			))),
		}
	}

	// returns:
	// 1. 'Option<ServerSynchronizationMessage>': a potential server ws event result to be send to iced app
	// 2. 'Option<ServerConnection>': a potential new server connection state ('ServerConnection::Disconnected')
	async fn send_request(
		request: Request,
		password: &str,
		websocket: &mut WebSocketStream,
	) -> (
		Option<ServerSynchronizationMessage>,
		Option<ServerConnection>,
	) {
		match request.encrypt(password) {
			Ok(request_binary) => {
				if websocket
					.send(tungstenite::Message::Binary(request_binary))
					.await
					.is_ok()
				{
					(None, None)
				} else {
					(
						Some(Ok(ServerSynchronizationEvent::Disconnected)),
						Some(ServerConnection::Disconnected),
					)
				}
			}
			Err(e) => (
				Some(Err(ServerSynchronizationError::EncryptRequest(format!(
					"{e}"
				)))),
				None,
			),
		}
	}
}

impl Default for ServerConnectionState {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone)]
pub struct ServerSynchronizationRequestSender(mpsc::UnboundedSender<Request>);

impl ServerSynchronizationRequestSender {
	fn send(&mut self, request: Request) -> Result<(), mpsc::TrySendError<Request>> {
		self.0.unbounded_send(request)
	}
}
