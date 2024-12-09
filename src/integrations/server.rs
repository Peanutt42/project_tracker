use iced::{futures::{self, channel::mpsc, SinkExt, Stream, StreamExt}, stream};
use project_tracker_server::{Request, Response, DEFAULT_HOSTNAME, DEFAULT_PASSWORD, DEFAULT_PORT};
use serde::{Deserialize, Serialize};
use async_tungstenite::tungstenite;

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

pub fn connect_ws() -> impl Stream<Item = ServerWsEvent> {
	stream::channel(100, |mut output| async move {
		let mut state = WsServerConnectionState {
			message_receiver: None,
			connection: WsServerConnection::Disconnected,
		};
		let mut message_sender_sent = false;

		loop {
			match &mut state.connection {
				WsServerConnection::Disconnected => {
					if !message_sender_sent {
						let (sender, receiver) = mpsc::channel(100);

						state.message_receiver = Some(receiver);

						if output.send(ServerWsEvent::MessageSender(ServerWsMessageSender(sender))).await.is_ok() {
							message_sender_sent = true;
						}
						else {
							return;
						}
					}

					if let Some(message_receiver) = &mut state.message_receiver {
						if let Some(ServerWsMessage::Connect(server_config)) = message_receiver.next().await {
							state.connection = WsServerConnection::Connecting(server_config);
						}
					}
				},
				WsServerConnection::Connecting(server_config) => {
					match &mut state.message_receiver {
						Some(message_receiver) => {
							if let Ok(Some(ServerWsMessage::Connect(new_server_config))) = message_receiver.try_next() {
								*server_config = new_server_config;
							}
							match async_tungstenite::tokio::connect_async(
								format!("ws://{}:{}", server_config.hostname, server_config.port)
							)
							.await
							{
								Ok((webserver, _)) => {
									if output.send(ServerWsEvent::Connected).await.is_err() {
										return;
									}
									state.connection = WsServerConnection::Connected(webserver, server_config.clone());
								},
								Err(e) => {
									eprintln!("failed to connect to ws: {e}");
									if output.send(ServerWsEvent::Error(format!("{e}"))).await.is_err() {
										return;
									}
									if output.send(ServerWsEvent::Disconnected).await.is_err() {
										return;
									}
									tokio::time::sleep(std::time::Duration::from_secs(1)).await;
								}
							}
						},
						None => {
							message_sender_sent = false;
							state.connection = WsServerConnection::Disconnected;
						},
					}
				}
				WsServerConnection::Connected(websocket, server_config) => {
					let mut fused_websocket = websocket.by_ref().fuse();

					match &mut state.message_receiver {
						Some(message_receiver) => futures::select! {
							received = fused_websocket.select_next_some() => {
								match received {
									Ok(tungstenite::Message::Binary(binary)) => {
										match Response::deserialize(binary) {
											Ok(response) => {
												if output.send(ServerWsEvent::Response{
													response,
													password: server_config.password.clone()
												})
												.await.is_err()
												{
													return;
												}
											},
											Err(e) => {
												eprintln!("failed to parse response from server: {e}");
												if output.send(ServerWsEvent::Error(format!("{e}"))).await.is_err() {
													return;
												}
											}
										}
									}
									Ok(tungstenite::Message::Close(_)) => {
										state.connection = WsServerConnection::Disconnected;
										if output.send(ServerWsEvent::Disconnected).await.is_err() {
											return;
										}
									}
									Err(e) => {
										eprintln!("ws server disconnected: {e}");
										state.connection = WsServerConnection::Disconnected;
										if output.send(ServerWsEvent::Disconnected).await.is_err() {
											return;
										}
									}
									_ => {},
								}
							}

							message = message_receiver.select_next_some() => match message {
								ServerWsMessage::Connect(server_config) => {
									state.connection = WsServerConnection::Connecting(server_config);
								},
								ServerWsMessage::Request(request) => {
									match request.encrypt(&server_config.password) {
										Ok(request_binary) => {
											let result = websocket.send(tungstenite::Message::Binary(request_binary)).await;

											if result.is_err() {
												if output.send(ServerWsEvent::Disconnected).await.is_err() {
													return;
												}

												state.connection = WsServerConnection::Disconnected;
											}
										},
										Err(e) => {
											eprintln!("failed to encrypt request: {e}");
											if output.send(ServerWsEvent::Error(format!("{e}"))).await.is_err() {
												return;
											}
										}
									}
								}
							}
						},
						None => {
							message_sender_sent = false;
							state.connection = WsServerConnection::Disconnected;
						},
					}
				}
			}
		}
	})
}

struct WsServerConnectionState {
	message_receiver: Option<mpsc::Receiver<ServerWsMessage>>,
	connection: WsServerConnection,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum WsServerConnection {
	Disconnected,
	Connecting(ServerConfig),
	Connected(
		async_tungstenite::WebSocketStream<
			async_tungstenite::tokio::ConnectStream
		>,
		ServerConfig
	),
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
	Error(String),
}

#[derive(Debug, Clone)]
pub struct ServerWsMessageSender(mpsc::Sender<ServerWsMessage>);

impl ServerWsMessageSender {
	pub fn send(&mut self, message: ServerWsMessage) -> Result<(), mpsc::TrySendError<ServerWsMessage>> {
		self.0.try_send(message)
	}
}

#[derive(Debug, Clone)]
pub enum ServerWsMessage {
	Connect(ServerConfig),
	Request(Request),
}

#[derive(Debug, Clone)]
pub enum ServerConnectionStatus {
	Disconected,
	Connected,
	Connecting,
	Error(String),
}