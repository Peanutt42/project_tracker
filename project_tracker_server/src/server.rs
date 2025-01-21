use crate::{
	AdminInfos, ConnectedClient, DatabaseUpdateEvent, EncryptedResponse, ModifiedEvent, Request,
	Response, ServerError,
};
use async_tungstenite::{
	tokio::accept_async,
	tungstenite::{self, Message},
};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use project_tracker_core::Database;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::{collections::HashSet, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::{error, info, warn};

#[allow(clippy::too_many_arguments)]
pub async fn run_server(
	port: usize,
	database_filepath: PathBuf,
	log_filepath: PathBuf,
	password: String,
	modified_sender: Sender<ModifiedEvent>,
	shared_database: Arc<RwLock<Database>>,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
	cpu_usage_avg: Arc<RwLock<f32>>,
) {
	let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
		.await
		.expect("failed to bind to port");

	info!("native ws listening on port {}", port);

	loop {
		match listener.accept().await {
			Ok((stream, _addr)) => {
				let database_filepath_clone = database_filepath.clone();
				let log_filepath_clone = log_filepath.clone();
				let password_clone = password.clone();
				let modified_sender_clone = modified_sender.clone();
				let modified_receiver = modified_sender.subscribe();
				let shared_database_clone = shared_database.clone();
				let connected_clients_clone = connected_clients.clone();
				let cpu_usage_avg_clone = cpu_usage_avg.clone();
				tokio::spawn(async move {
					handle_client(
						stream,
						database_filepath_clone,
						log_filepath_clone,
						password_clone,
						modified_sender_clone,
						modified_receiver,
						shared_database_clone,
						connected_clients_clone,
						cpu_usage_avg_clone,
					)
					.await
				});
			}
			Err(e) => {
				error!("failed to establish ws connection: {e}")
			}
		}
	}
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_client(
	stream: TcpStream,
	database_filepath: PathBuf,
	log_filepath: PathBuf,
	password: String,
	modified_sender: Sender<ModifiedEvent>,
	modified_receiver: Receiver<ModifiedEvent>,
	shared_database: Arc<RwLock<Database>>,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
	cpu_usage_avg: Arc<RwLock<f32>>,
) {
	let client_addr = match stream.peer_addr() {
		Ok(client_addr) => client_addr,
		Err(e) => {
			error!("failed to get ws client socket address, ignoring client, error: {e}");
			return;
		}
	};

	let connected_client = ConnectedClient::NativeGUI(client_addr);

	connected_clients.write().unwrap().insert(connected_client);

	listen_client_thread(
		stream,
		client_addr,
		database_filepath,
		log_filepath,
		password,
		modified_sender,
		modified_receiver,
		shared_database.clone(),
		connected_clients.clone(),
		cpu_usage_avg.clone(),
	)
	.await;

	connected_clients.write().unwrap().remove(&connected_client);
}

#[allow(clippy::too_many_arguments)]
async fn listen_client_thread(
	stream: TcpStream,
	client_addr: SocketAddr,
	database_filepath: PathBuf,
	log_filepath: PathBuf,
	password: String,
	modified_sender: Sender<ModifiedEvent>,
	mut modified_receiver: Receiver<ModifiedEvent>,
	shared_database: Arc<RwLock<Database>>,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
	cpu_usage_avg: Arc<RwLock<f32>>,
) {
	let ws_stream = match accept_async(stream).await {
		Ok(ws) => {
			info!("ws client connected");
			ws
		}
		Err(e) => {
			error!("ws handshake failed: {e}");
			return;
		}
	};

	let (mut write, mut read) = ws_stream.split();

	loop {
		tokio::select! {
			ws_message = read.next() => match ws_message {
				Some(Ok(tungstenite::Message::Binary(request_binary))) => {
					match Request::decrypt(request_binary, &password) {
						Ok(request) => {
							respond_to_client_request(
								request,
								client_addr,
								&shared_database,
								&connected_clients,
								&cpu_usage_avg,
								&modified_sender,
								&mut write,
								&database_filepath,
								&log_filepath,
								&password,
							)
							.await
						}
						Err(e) => match e {
							ServerError::ResponseError(e) => {
								error!("{e}");
								let _ = write
									.send(Message::binary(Response(Err(e)).serialize().unwrap()))
									.await;
							}
							_ => error!("failed to parse ws request: {e}"),
						},
					}
				}
				Some(Err(ref e))
					if matches!(
						e,
						tungstenite::Error::ConnectionClosed
							| tungstenite::Error::AlreadyClosed
							| tungstenite::Error::Protocol(
								tungstenite::error::ProtocolError::ResetWithoutClosingHandshake
							)
					) =>
				{
					info!("client disconnected");
					return;
				}
				None => {
					info!("client disconnected");
					return;
				}
				Some(Err(e)) => {
					error!("failed to read ws message: {e}")
				}
				Some(Ok(_)) => { info!("ignoring ws msg that isnt binary"); }
			},
			modified_event = modified_receiver.recv() => if let Ok(modified_event) = modified_event {
				// do not resend database updated msg to the sender that made that update
				if modified_event.modified_sender_address != client_addr {
					info!("sending database modified event in ws");
					let database_modified_response = match modified_event.database_update_event {
						DatabaseUpdateEvent::DatabaseMessage { database_message, before_modification_checksum } => {
							EncryptedResponse::DatabaseChanged {
								database_before_update_checksum: before_modification_checksum,
								database_message,
							}
						},
						DatabaseUpdateEvent::ImportDatabase => {
							let last_modified_time = *modified_event.modified_database.last_changed_time();
							EncryptedResponse::MoreUpToDateDatabase {
								database: modified_event.modified_database.into_serialized(),
								last_modified_time,
							}
						}
					};

					let failed_to_send_msg = write
						.send(Message::binary(
							Response(Ok(database_modified_response.encrypt(&password).unwrap()))
								.serialize()
								.unwrap(),
						))
						.await
						.is_err();

					if failed_to_send_msg {
						error!("failed to send modified event in ws, closing connection");
						break;
					}
				}
			}
		}
	}
}

#[allow(clippy::too_many_arguments)]
async fn respond_to_client_request(
	request: Request,
	client_addr: SocketAddr,
	shared_database: &Arc<RwLock<Database>>,
	connected_clients: &Arc<RwLock<HashSet<ConnectedClient>>>,
	cpu_usage_avg: &Arc<RwLock<f32>>,
	modified_sender: &Sender<ModifiedEvent>,
	ws_write: &mut WsWriteSink,
	database_filepath: &PathBuf,
	log_filepath: &PathBuf,
	password: &str,
) {
	match request {
		Request::CheckUpToDate { database_checksum } => {
			info!("sending last modified date");
			let is_up_to_date = database_checksum == shared_database.read().unwrap().checksum();
			if is_up_to_date {
				let encrypted_response_msg = EncryptedResponse::DatabaseUpToDate
					.encrypt(password)
					.unwrap();
				let response = Response(Ok(encrypted_response_msg)).serialize().unwrap();

				if let Err(e) = ws_write.send(Message::binary(response)).await {
					error!("failed to respond to 'GetModifiedDate' request: {e}");
				}
			} else {
				warn!("clients checksum doesnt match ours -> sending full db");
				send_more_up_to_date_database(shared_database, ws_write, password).await;
			}
		}
		Request::UpdateDatabase {
			database_message,
			database_before_update_checksum,
		} => {
			let database_synced =
				database_before_update_checksum == shared_database.read().unwrap().checksum();

			if database_synced {
				let database = {
					let mut shared_data = shared_database.write().unwrap();
					shared_data.update(database_message.clone());
					shared_data.clone()
				};

				let database_binary = database.to_binary().unwrap();

				let _ = modified_sender.send(ModifiedEvent::new(
					database,
					DatabaseUpdateEvent::DatabaseMessage {
						database_message,
						before_modification_checksum: database_before_update_checksum,
					},
					client_addr,
				));

				let _ = ws_write
					.send(Message::binary(
						Response(Ok(EncryptedResponse::DatabaseUpdated
							.encrypt(password)
							.unwrap()))
						.serialize()
						.unwrap(),
					))
					.await;

				if let Err(e) = tokio::fs::write(database_filepath, database_binary).await {
					panic!(
						"cant write to database file: {}, error: {e}",
						database_filepath.display()
					);
				}

				info!("updated database");
			} else {
				warn!("clients wanted to update db but checksum doesnt match ours -> sending full db instead");
				send_more_up_to_date_database(shared_database, ws_write, password).await;
			}
		}
		Request::ImportDatabase { database } => {
			let database = {
				let mut shared_data = shared_database.write().unwrap();
				*shared_data = Database::from_serialized(database, Utc::now());
				shared_data.clone()
			};

			let database_binary = database.to_binary().unwrap();

			let _ = modified_sender.send(ModifiedEvent::new(
				database,
				DatabaseUpdateEvent::ImportDatabase,
				client_addr,
			));

			let _ = ws_write
				.send(Message::binary(
					Response(Ok(EncryptedResponse::DatabaseUpdated
						.encrypt(password)
						.unwrap()))
					.serialize()
					.unwrap(),
				))
				.await;

			if let Err(e) = tokio::fs::write(database_filepath, database_binary).await {
				panic!(
					"cant write to database file: {}, error: {e}",
					database_filepath.display()
				);
			}

			info!("imported database");
		}
		Request::GetFullDatabase => {
			send_more_up_to_date_database(shared_database, ws_write, password).await;
		}
		Request::AdminInfos => {
			let response = Response(Ok(EncryptedResponse::AdminInfos(AdminInfos::generate(
				connected_clients.clone(),
				cpu_usage_avg.clone(),
				log_filepath,
			))
			.encrypt(password)
			.unwrap()));
			match ws_write
				.send(Message::binary(response.serialize().unwrap()))
				.await
			{
				Ok(_) => info!("sent admin infos"),
				Err(e) => error!("failed to send admin infos: {e}"),
			}
		}
	}
}

async fn send_more_up_to_date_database(
	shared_database: &Arc<RwLock<Database>>,
	ws_write: &mut WsWriteSink,
	password: &str,
) {
	let (database, last_modified_time) = {
		let shared_database = shared_database.read().unwrap();
		(
			shared_database.serialized().clone(),
			*shared_database.last_changed_time(),
		)
	};

	let response = Response(Ok(EncryptedResponse::MoreUpToDateDatabase {
		database,
		last_modified_time,
	}
	.encrypt(password)
	.unwrap()))
	.serialize()
	.unwrap();

	if let Err(e) = ws_write.send(Message::binary(response)).await {
		error!("failed to respond to 'GetModifiedDate' request: {e}");
	}
}

type WsWriteSink = futures_util::stream::SplitSink<
	async_tungstenite::WebSocketStream<async_tungstenite::tokio::TokioAdapter<TcpStream>>,
	Message,
>;
