use async_tungstenite::tungstenite;
use iced::{
	futures::{self, channel::mpsc, SinkExt, Stream, StreamExt},
	stream,
	widget::{row, text, Space},
	Alignment, Element,
};
use project_tracker_server::{Request, Response, DEFAULT_HOSTNAME, DEFAULT_PASSWORD, DEFAULT_PORT};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, info};

use crate::{
	components::{retry_connecting_to_server_button, show_error_popup_button},
	project_tracker::Message,
	styles::{danger_text_style, SPACING_AMOUNT},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

pub fn connect_ws() -> impl Stream<Item = Result<ServerWsEvent, ServerWsError>> {
	stream::channel(100, |mut output| async move {
		let mut state = WsServerConnectionState::new();
		let mut connection = WsServerConnection::Disconnected;

		while state.update(&mut connection, &mut output).await {}
	})
}

pub struct WsServerConnectionState {
	message_receiver: Option<mpsc::UnboundedReceiver<ServerWsMessage>>,
	message_sender_sent: bool,
}

impl WsServerConnectionState {
	pub fn new() -> Self {
		Self {
			message_receiver: None,
			message_sender_sent: false,
		}
	}

	// returns whether to continue or to quit
	pub async fn update(
		&mut self,
		connection: &mut WsServerConnection,
		output: &mut mpsc::Sender<Result<ServerWsEvent, ServerWsError>>,
	) -> bool {
		match connection {
			WsServerConnection::Disconnected => {
				if !self.message_sender_sent {
					let (sender, receiver) = mpsc::unbounded();

					self.message_receiver = Some(receiver);

					if output
						.send(Ok(ServerWsEvent::MessageSender(ServerWsMessageSender(
							sender,
						))))
						.await
						.is_ok()
					{
						self.message_sender_sent = true;
					} else {
						return false;
					}
				}

				if let Some(message_receiver) = &mut self.message_receiver {
					if let Some(ServerWsMessage::Connect(server_config)) =
						message_receiver.next().await
					{
						*connection = WsServerConnection::Connecting(server_config);
					}
				}
				true
			}
			WsServerConnection::Connecting(server_config) => {
				let server_config = server_config.clone();
				match self.connect(output, connection, server_config).await {
					Ok(continue_subscription) => continue_subscription,
					Err(e) => {
						error!("failed to connect to ws: {e}");
						if output
							.send(Err(ServerWsError::ConnectToWsServer(format!("{e}"))))
							.await
							.is_err()
						{
							return false;
						}
						if output.send(Ok(ServerWsEvent::Disconnected)).await.is_err() {
							return false;
						}
						tokio::time::sleep(std::time::Duration::from_secs(1)).await;
						true
					}
				}
			}
			WsServerConnection::Connected(websocket, server_config) => {
				let (continue_subscription, new_connection_state) =
					self.listen(websocket, server_config.clone(), output).await;
				if let Some(new_connection_state) = new_connection_state {
					*connection = new_connection_state;
				}
				continue_subscription
			}
		}
	}

	async fn connect(
		&mut self,
		output: &mut mpsc::Sender<Result<ServerWsEvent, ServerWsError>>,
		connection: &mut WsServerConnection,
		mut server_config: ServerConfig,
	) -> Result<bool, async_tungstenite::tungstenite::Error> {
		match &mut self.message_receiver {
			Some(message_receiver) => {
				if let WsServerConnection::Connecting(current_server_config) = connection {
					if let Ok(Some(ServerWsMessage::Connect(new_server_config))) =
						message_receiver.try_next()
					{
						*current_server_config = new_server_config.clone();
						server_config = new_server_config;
					}
				}

				let address = format!("ws://{}:{}", server_config.hostname, server_config.port);

				let (webserver, _) = async_tungstenite::tokio::connect_async(address).await?;

				if output.send(Ok(ServerWsEvent::Connected)).await.is_err() {
					return Ok(false);
				}
				*connection = WsServerConnection::Connected(webserver, server_config.clone());
			}
			None => {
				self.message_sender_sent = false;
				*connection = WsServerConnection::Disconnected;
			}
		}
		Ok(true)
	}

	async fn listen(
		&mut self,
		websocket: &mut WebSocketStream,
		server_config: ServerConfig,
		output: &mut mpsc::Sender<Result<ServerWsEvent, ServerWsError>>,
	) -> (bool, Option<WsServerConnection>) {
		let mut fused_websocket = websocket.by_ref().fuse();

		match &mut self.message_receiver {
			Some(message_receiver) => futures::select! {
				received = fused_websocket.select_next_some() => {
					match received {
						Ok(tungstenite::Message::Binary(binary)) => (
							output.send(Self::parse_server_response(binary, server_config.password.clone())).await.is_ok(),
							None
						),
						Ok(tungstenite::Message::Close(_)) => (
							output.send(Ok(ServerWsEvent::Disconnected)).await.is_ok(),
							Some(WsServerConnection::Disconnected)
						),
						Err(e) => {
							info!("ws server disconnected: {e}");
							(
								output.send(Ok(ServerWsEvent::Disconnected)).await.is_ok(),
								Some(WsServerConnection::Disconnected)
							)
						}
						_ => (true, None),
					}
				},

				message = message_receiver.select_next_some() => {
					match message {
						ServerWsMessage::Connect(server_config) => (
							true,
							Some(WsServerConnection::Connecting(server_config))
						),
						ServerWsMessage::CloseSubscription => {
							let _  = websocket.close(None).await;
							(
								false,
								Some(WsServerConnection::Disconnected)
							)
						},
						ServerWsMessage::Request(request) => {
							let (opt_event_result, opt_new_connection) = Self::send_request(request, &server_config.password, websocket).await;
							let continue_subscription = match opt_event_result {
								Some(event_result) => output.send(event_result).await.is_ok(),
								None => true,
							};

							(
								continue_subscription,
								opt_new_connection,
							)
						}
					}
				},
			},
			None => {
				self.message_sender_sent = false;
				(true, Some(WsServerConnection::Disconnected))
			}
		}
	}

	fn parse_server_response(
		binary_response: Vec<u8>,
		password: String,
	) -> Result<ServerWsEvent, ServerWsError> {
		match Response::deserialize(binary_response) {
			Ok(response) => Ok(ServerWsEvent::Response { response, password }),
			Err(e) => Err(ServerWsError::ParseServerResponse(format!("{e}"))),
		}
	}

	// returns:
	// 1. 'Option<Result<ServerWsEvent, ServerWsError>>': a potential server ws event result to be send to iced app
	// 2. 'Option<WsServerConnection>': a potential new server connection state ('WsServerConnection::Disconnected')
	async fn send_request(
		request: Request,
		password: &str,
		websocket: &mut WebSocketStream,
	) -> (
		Option<Result<ServerWsEvent, ServerWsError>>,
		Option<WsServerConnection>,
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
						Some(Ok(ServerWsEvent::Disconnected)),
						Some(WsServerConnection::Disconnected),
					)
				}
			}
			Err(e) => (
				Some(Err(ServerWsError::EncryptRequest(format!("{e}")))),
				None,
			),
		}
	}
}

impl Default for WsServerConnectionState {
	fn default() -> Self {
		Self::new()
	}
}

type WebSocketStream = async_tungstenite::WebSocketStream<async_tungstenite::tokio::ConnectStream>;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum WsServerConnection {
	Disconnected,
	Connecting(ServerConfig),
	Connected(WebSocketStream, ServerConfig),
}

#[derive(Debug, Clone)]
pub enum ServerWsEvent {
	MessageSender(ServerWsMessageSender),
	Connected,
	Disconnected,
	Response {
		response: Response,
		password: String,
	},
}

#[derive(Debug, Clone, Error)]
pub enum ServerWsError {
	#[error("failed to connect to ws server: {0}")]
	ConnectToWsServer(String),
	#[error("failed to encrypt request: {0}")]
	EncryptRequest(String),
	#[error("failed to parse server response: {0}")]
	ParseServerResponse(String),
}

#[derive(Debug, Clone)]
pub struct ServerWsMessageSender(mpsc::UnboundedSender<ServerWsMessage>);

impl ServerWsMessageSender {
	pub fn send(
		&mut self,
		message: ServerWsMessage,
	) -> Result<(), mpsc::TrySendError<ServerWsMessage>> {
		self.0.unbounded_send(message)
	}
}

#[derive(Debug, Clone)]
pub enum ServerWsMessage {
	Connect(ServerConfig),
	Request(Request),
	CloseSubscription, // closes listen thread, only used by tests
}

#[derive(Debug, Clone)]
pub enum ServerConnectionStatus {
	Disconected,
	Connected,
	Connecting,
	Error(String),
}

impl ServerConnectionStatus {
	pub fn view(&self) -> Element<Message> {
		match self {
			ServerConnectionStatus::Error(error_msg) => row![
				text("Server Error").style(danger_text_style),
				show_error_popup_button(error_msg.clone()),
				retry_connecting_to_server_button()
			]
			.spacing(SPACING_AMOUNT)
			.align_y(Alignment::Center)
			.into(),
			ServerConnectionStatus::Disconected => {
				row![text("Disconnected"), retry_connecting_to_server_button()]
					.spacing(SPACING_AMOUNT)
					.align_y(Alignment::Center)
					.into()
			}
			ServerConnectionStatus::Connecting => text("Connecting...").into(),
			ServerConnectionStatus::Connected => Element::new(Space::new(0.0, 0.0)),
		}
	}
}
