use std::hash::{Hash, Hasher};

use crate::synchronization::server::ServerSynchronizationError;
use crate::synchronization::{ServerConfig, SynchronizationMessage};
use async_tungstenite::tungstenite;
use iced::futures::{self, channel::mpsc, SinkExt, StreamExt};
use iced::{stream, Subscription};
use project_tracker_server::{Request, Response, SerializedRequest, SerializedResponse};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub enum ServerSynchronizationEvent {
	RequestSender(mpsc::UnboundedSender<Request>),
	Connected,
	Disconnected,
	Response(Response),
}

pub type ServerSubscriptionMessage = Result<ServerSynchronizationEvent, ServerSynchronizationError>;

impl From<ServerSubscriptionMessage> for SynchronizationMessage {
	fn from(message: ServerSubscriptionMessage) -> Self {
		SynchronizationMessage::ServerSynchronizationMessage(message)
	}
}

pub fn ws_subscription(config: ServerConfig) -> Subscription<ServerSubscriptionMessage> {
	// if the server config changes --> new hash --> new subscription id --> reconnect
	let server_config_hash = {
		let mut hasher = std::hash::DefaultHasher::new();
		config.hash(&mut hasher);
		hasher.finish()
	};

	let ws_stream = stream::channel(100, |mut output| async move {
		let mut state = ServerConnectionState::new();
		let mut connection = ServerConnection::Disconnected;

		while state.update(&mut connection, &config, &mut output).await {}
	});

	Subscription::run_with_id(server_config_hash, ws_stream)
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
		output: &mut mpsc::Sender<ServerSubscriptionMessage>,
	) -> bool {
		match connection {
			ServerConnection::Disconnected => {
				if !self.request_sender_sent {
					let (sender, receiver) = mpsc::unbounded();

					self.request_receiver = Some(receiver);

					if output
						.send(Ok(ServerSynchronizationEvent::RequestSender(sender)))
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
		output: &mut mpsc::Sender<ServerSubscriptionMessage>,
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
		output: &mut mpsc::Sender<ServerSubscriptionMessage>,
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
						Ok(tungstenite::Message::Close(_)) => {
							info!("ws server disconnected");
							(
								output.send(Ok(ServerSynchronizationEvent::Disconnected)).await.is_ok(),
								Some(ServerConnection::Disconnected)
							)
						},
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
	) -> ServerSubscriptionMessage {
		match SerializedResponse::decrypt(&binary_response, &password) {
			Ok(response) => Ok(ServerSynchronizationEvent::Response(response)),
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
	) -> (Option<ServerSubscriptionMessage>, Option<ServerConnection>) {
		match SerializedRequest::encrypt(&request, password) {
			Ok(request_bytes) => {
				if websocket
					.send(tungstenite::Message::Binary(request_bytes))
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
			Err(e) => {
				error!("failed to serialize request: {e}, cant send request to server");
				(None, None)
			}
		}
	}
}

impl Default for ServerConnectionState {
	fn default() -> Self {
		Self::new()
	}
}
