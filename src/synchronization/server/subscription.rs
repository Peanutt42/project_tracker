use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use crate::synchronization::server::ServerSynchronizationError;
use crate::synchronization::{ServerConfig, SynchronizationMessage};
use async_tungstenite::tungstenite;
use flume::{unbounded, Receiver, Sender};
use iced::futures::{self, channel::mpsc, SinkExt, StreamExt};
use iced::{stream, Subscription};
use project_tracker_server::{Request, Response, SerializedRequest, SerializedResponse};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub enum ServerSynchronizationEvent {
	RequestSender(Sender<Request>),
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
	request_receiver: Option<Receiver<Request>>,
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
					let (sender, receiver) = unbounded();

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

		match &self.request_receiver {
			Some(request_receiver) => {
				let mut request_receiver_stream = request_receiver.stream();
				futures::select! {
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

					first_request = request_receiver_stream.select_next_some() => {
						let mut pending_requests: VecDeque<Request> = request_receiver.try_iter().collect();
						pending_requests.push_front(first_request);

						let (continue_subscription, opt_new_connection) = Self::bulk_send_requests(pending_requests, &config.password, websocket, output).await;

						tokio::time::sleep(std::time::Duration::from_secs(1)).await;

						(
							continue_subscription,
							opt_new_connection,
						)
					},
				}
			}
			None => {
				self.request_sender_sent = false;
				(true, Some(ServerConnection::Disconnected))
			}
		}
	}

	/// combines continues database message requests into a single "bulk" 'DatabaseMessage' request
	async fn bulk_send_requests(
		mut requests: VecDeque<Request>,
		password: &str,
		websocket: &mut WebSocketStream,
		output: &mut mpsc::Sender<ServerSubscriptionMessage>,
	) -> (bool, Option<ServerConnection>) {
		let mut opt_new_connection = None;

		while let Some(first_request) = requests.pop_front() {
			let (continue_subscription, new_connection) = match first_request {
				Request::UpdateDatabase {
					mut database_messages,
					database_before_update_checksum,
				} => {
					while let Some(next_request) = requests.front() {
						match next_request {
							Request::UpdateDatabase { .. } => {
								database_messages.extend(match requests.pop_front() {
									Some(Request::UpdateDatabase { database_messages, .. }) => database_messages,
									_ => unreachable!(".front() was DatabaseMessage, .pop_front() should therefore also be"),
								});
							}
							_ => break,
						}
					}

					Self::send_request(
						Request::UpdateDatabase {
							database_messages,
							database_before_update_checksum,
						},
						password,
						websocket,
						output,
					)
					.await
				}
				_ => Self::send_request(first_request, password, websocket, output).await,
			};
			if !continue_subscription {
				return (false, new_connection);
			}
			if let Some(new_connection) = new_connection {
				opt_new_connection = Some(new_connection);
			}
		}

		(true, opt_new_connection)
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
	// 1. 'bool': wheter the subscription should continue
	// 2. 'Option<ServerConnection>': a potential new server connection state ('ServerConnection::Disconnected')
	async fn send_request(
		request: Request,
		password: &str,
		websocket: &mut WebSocketStream,
		output: &mut mpsc::Sender<ServerSubscriptionMessage>,
	) -> (bool, Option<ServerConnection>) {
		match SerializedRequest::encrypt(&request, password) {
			Ok(request_bytes) => {
				if websocket
					.send(tungstenite::Message::Binary(request_bytes))
					.await
					.is_ok()
				{
					(true, None)
				} else {
					let continue_subscription = output
						.send(Ok(ServerSynchronizationEvent::Disconnected))
						.await
						.is_ok();
					(continue_subscription, Some(ServerConnection::Disconnected))
				}
			}
			Err(e) => {
				error!("failed to serialize request: {e}, cant send request to server");
				(true, None)
			}
		}
	}
}

impl Default for ServerConnectionState {
	fn default() -> Self {
		Self::new()
	}
}
