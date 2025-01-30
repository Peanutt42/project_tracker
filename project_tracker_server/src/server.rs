use crate::{
	save_database_to_file, AdminInfos, ConnectedClient, CpuUsageAverage, DatabaseUpdateEvent,
	ModifiedEvent, Request, Response, SerializedRequest, SerializedResponse, ServerError,
};
use async_tungstenite::{
	tokio::accept_async,
	tungstenite::{self, Message},
};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use project_tracker_core::Database;
use std::sync::Arc;
use std::{collections::HashSet, net::SocketAddr};
use std::{path::PathBuf, process::exit};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

type WsWriteSink = futures_util::stream::SplitSink<
	async_tungstenite::WebSocketStream<async_tungstenite::tokio::TokioAdapter<TcpStream>>,
	Message,
>;

#[allow(clippy::too_many_arguments)]
pub async fn run_server(
	port: usize,
	database_filepath: PathBuf,
	log_filepath: PathBuf,
	password: String,
	modified_sender: Sender<ModifiedEvent>,
	shared_database: Arc<RwLock<Database>>,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
	cpu_usage_avg: Arc<CpuUsageAverage>,
) {
	let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
		.await
		.unwrap_or_else(|e| {
			error!("failed to bind to port {port}, error: {e}");
			exit(1);
		});

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
	cpu_usage_avg: Arc<CpuUsageAverage>,
) {
	let client_addr = match stream.peer_addr() {
		Ok(client_addr) => client_addr,
		Err(e) => {
			error!("failed to get ws client socket address, ignoring client, error: {e}");
			return;
		}
	};

	let connected_client = ConnectedClient::NativeGUI(client_addr);

	connected_clients.write().await.insert(connected_client);

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
		cpu_usage_avg,
	)
	.await;

	connected_clients.write().await.remove(&connected_client);
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
	cpu_usage_avg: Arc<CpuUsageAverage>,
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
					match SerializedRequest::decrypt(&request_binary, &password) {
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
						Err(e) => {
							error!("{e}");
							send_error_response(e, &mut write).await;
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
						DatabaseUpdateEvent::DatabaseMessage { database_messages, before_modification_checksum } => {
							Response::DatabaseChanged {
								database_before_update_checksum: before_modification_checksum,
								database_messages,
							}
						},
						DatabaseUpdateEvent::ImportDatabase => {
							let last_modified_time = *modified_event.modified_database.last_changed_time();
							Response::MoreUpToDateDatabase {
								database: modified_event.modified_database.into_serialized(),
								last_modified_time,
							}
						}
					};

					let failed_to_send_msg = send_response(&database_modified_response, &mut write, &password).await;

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
	cpu_usage_avg: &Arc<CpuUsageAverage>,
	modified_sender: &Sender<ModifiedEvent>,
	ws_write: &mut WsWriteSink,
	database_filepath: &PathBuf,
	log_filepath: &PathBuf,
	password: &str,
) {
	match request {
		Request::CheckUpToDate { database_checksum } => {
			info!("sending last modified date");
			let is_up_to_date = database_checksum == shared_database.read().await.checksum();
			if is_up_to_date {
				send_response(&Response::DatabaseUpToDate, ws_write, password).await;
			} else {
				warn!("clients checksum doesnt match ours -> sending full db");
				send_more_up_to_date_database(shared_database, ws_write, password).await;
			}
		}
		Request::UpdateDatabase {
			database_messages,
			database_before_update_checksum,
		} => {
			let database_synced =
				database_before_update_checksum == shared_database.read().await.checksum();

			if database_synced {
				info!("updating database");

				let database = {
					let mut shared_database = shared_database.write().await;
					for database_message in database_messages.clone() {
						shared_database.update(database_message);
					}
					shared_database.clone()
				};

				let database_binary = database.to_binary();

				broadcast_modified_event(
					DatabaseUpdateEvent::DatabaseMessage {
						database_messages,
						before_modification_checksum: database_before_update_checksum,
					},
					modified_sender,
					database,
					client_addr,
				);

				send_response(&Response::DatabaseUpdated, ws_write, password).await;

				match database_binary {
					Some(database_binary) => {
						save_database_to_file(database_filepath, &database_binary).await
					}
					None => error!(
						"failed to serialize database to binary -> cant save database to file"
					),
				}
			} else {
				warn!("clients wanted to update db but checksum doesnt match ours -> sending full db instead");
				send_more_up_to_date_database(shared_database, ws_write, password).await;
			}
		}
		Request::ImportDatabase { database } => {
			info!("importing database");

			let database = {
				let mut shared_database = shared_database.write().await;
				*shared_database = Database::from_serialized(database, Utc::now());
				shared_database.clone()
			};

			let database_binary = database.to_binary();

			broadcast_modified_event(
				DatabaseUpdateEvent::ImportDatabase,
				modified_sender,
				database,
				client_addr,
			);

			send_response(&Response::DatabaseUpdated, ws_write, password).await;

			match database_binary {
				Some(database_binary) => {
					save_database_to_file(database_filepath, &database_binary).await;
				}
				None => {
					error!("failed to serialize database to binary -> cant save database to file");
				}
			}
		}
		Request::GetFullDatabase => {
			send_more_up_to_date_database(shared_database, ws_write, password).await;
		}
		Request::AdminInfos => {
			info!("sending admin infos");

			send_response(
				&Response::AdminInfos(AdminInfos::generate(
					connected_clients.read().await.clone(),
					cpu_usage_avg,
					log_filepath,
				)),
				ws_write,
				password,
			)
			.await;
		}
	}
}

async fn send_more_up_to_date_database(
	shared_database: &Arc<RwLock<Database>>,
	ws_write: &mut WsWriteSink,
	password: &str,
) {
	let (database, last_modified_time) = {
		let shared_database = shared_database.read().await.clone();
		let last_changed_time = *shared_database.last_changed_time();
		(shared_database.into_serialized(), last_changed_time)
	};

	send_response(
		&Response::MoreUpToDateDatabase {
			database,
			last_modified_time,
		},
		ws_write,
		password,
	)
	.await;
}

/// returns wheter sending failed
async fn send_response(response: &Response, ws_write: &mut WsWriteSink, password: &str) -> bool {
	match SerializedResponse::ok(response, password) {
		Ok(response_bytes) => match ws_write.send(Message::binary(response_bytes)).await {
			Ok(_) => false,
			Err(e) => {
				error!("failed to send response: {e},\nresponse was: {response:#?}");
				true
			}
		},
		Err(e) => {
			error!("failed to serialize response: {e}");
			false
		}
	}
}

/// returns wheter sending failed
async fn send_error_response(error: ServerError, ws_write: &mut WsWriteSink) -> bool {
	match SerializedResponse::error(error) {
		Ok(response_bytes) => match ws_write.send(Message::binary(response_bytes)).await {
			Ok(_) => false,
			Err(e) => {
				error!("failed to send error response: {e}");
				true
			}
		},
		Err(e) => {
			error!("failed to serialize error response: {e}");
			false
		}
	}
}

fn broadcast_modified_event(
	update_event: DatabaseUpdateEvent,
	modified_sender: &Sender<ModifiedEvent>,
	database: Database,
	client_addr: SocketAddr,
) {
	let _ = modified_sender.send(ModifiedEvent::new(database, update_event, client_addr));
}
