use std::sync::Arc;

use iced::{futures::{self, channel::mpsc, SinkExt, Stream, StreamExt}, stream};
use project_tracker_server::{Request, Response, ServerError, DEFAULT_HOSTNAME, DEFAULT_PASSWORD, DEFAULT_PORT};
use serde::{Deserialize, Serialize};
use async_tungstenite::tungstenite;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
				WsServerConnection::Disconnected => if message_sender_sent {

				}
				else {
					let (sender, mut receiver) = mpsc::channel(100);

					if output.send(ServerWsEvent::MessageSender(ServerWsMessageSender(sender))).await.is_ok() {
						message_sender_sent = true;
					}
					else {
						return;
					}
					if let Some(ServerWsMessage::Connect(server_config)) = receiver.next().await {
						message_sender_sent = false;
						state.message_receiver = Some(receiver);
						state.connection = WsServerConnection::Connecting(server_config);
					}
				},
				WsServerConnection::Connecting(server_config) => {
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
							tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
							if output.send(ServerWsEvent::Disconnected).await.is_err() {
								return;
							}
						}
					}
				}
				WsServerConnection::Connected(websocket, server_config) => {
					let mut fused_websocket = websocket.by_ref().fuse();

					match &mut state.message_receiver {
						Some(message_receiver) => futures::select! {
							received = fused_websocket.select_next_some() => {
								match received {
									Ok(tungstenite::Message::Binary(binary)) => {
										match Response::decrypt(binary, &server_config.password) {
											Ok(response) => {
												if output.send(ServerWsEvent::Response(response)).await.is_err() {
													return;
												}
											},
											Err(e) => {
												eprintln!("failed to parse response from server: {e}");
												if output.send(ServerWsEvent::ServerError(Arc::new(e))).await.is_err() {
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
											if output.send(ServerWsEvent::ServerError(Arc::new(e))).await.is_err() {
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
	Response(Response),
	ServerError(Arc<ServerError>),
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