use chrono::Utc;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use project_tracker_core::{Database, ProjectId, TaskId};
use project_tracker_server::{
	save_database_to_file, AdminInfos, ConnectedClient, CpuUsageAverage, DatabaseUpdateEvent,
	ModifiedEvent, Request, Response, SerializedRequest, SerializedResponse, ServerError,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::{
	broadcast::{Receiver, Sender},
	RwLock,
};
use tracing::{error, info, warn};
use warp::{
	filters::ws::{ws, Message, WebSocket, Ws},
	path, Filter, Rejection, Reply,
};

#[allow(clippy::too_many_arguments)]
pub fn native_ws_route(
	database_filepath: PathBuf,
	log_filepath: PathBuf,
	shared_database: Arc<RwLock<Database>>,
	modified_sender: Sender<ModifiedEvent>,
	modified_receiver: Receiver<ModifiedEvent>,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
	password: String,
	cpu_usage_avg: Arc<CpuUsageAverage>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
	let modified_sender = Arc::new(RwLock::new(modified_sender));
	let modified_receiver = Arc::new(RwLock::new(modified_receiver));

	path("native_ws")
		.and(ws())
		.and(warp::addr::remote())
		.and(warp::any().map(move || database_filepath.clone()))
		.and(warp::any().map(move || log_filepath.clone()))
		.and(warp::any().map(move || shared_database.clone()))
		.and(warp::any().map(move || modified_sender.clone()))
		.and(warp::any().map(move || modified_receiver.clone()))
		.and(warp::any().map(move || connected_clients.clone()))
		.and(warp::any().map(move || password.clone()))
		.and(warp::any().map(move || cpu_usage_avg.clone()))
		.then(
			move |ws: Ws,
			      client_addr: Option<SocketAddr>,
			      database_filepath: PathBuf,
			      log_filepath: PathBuf,
			      shared_database: Arc<RwLock<Database>>,
			      modified_sender: Arc<RwLock<Sender<ModifiedEvent>>>,
			      modified_receiver: Arc<RwLock<Receiver<ModifiedEvent>>>,
			      connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
			      password: String,
			      cpu_usage_avg: Arc<CpuUsageAverage>| {
				async move {
					let modified_sender = modified_sender.read().await.clone();
					let modified_receiver = modified_receiver.read().await.resubscribe();

					ws.on_upgrade(move |socket| {
						on_upgrade_ws(
							socket,
							password,
							client_addr,
							database_filepath,
							log_filepath,
							shared_database,
							modified_sender,
							modified_receiver,
							connected_clients,
							cpu_usage_avg,
						)
					})
				}
			},
		)
}

#[allow(clippy::too_many_arguments)]
async fn on_upgrade_ws(
	ws: WebSocket,
	password: String,
	client_addr: Option<SocketAddr>,
	database_filepath: PathBuf,
	log_filepath: PathBuf,
	shared_database: Arc<RwLock<Database>>,
	modified_sender: Sender<ModifiedEvent>,
	modified_receiver: Receiver<ModifiedEvent>,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
	cpu_usage_avg: Arc<CpuUsageAverage>,
) {
	let client_addr = match client_addr {
		Some(client_addr) => client_addr,
		None => {
			warn!("client_addr was not specified, ignoring ws connection");
			return;
		}
	};

	// TODO: authenticate client password!!!

	let connected_client = ConnectedClient::Web(client_addr);

	connected_clients.write().await.insert(connected_client);

	handle_ws(
		ws,
		password,
		client_addr,
		database_filepath,
		log_filepath,
		shared_database,
		modified_sender,
		modified_receiver,
		connected_clients.clone(),
		cpu_usage_avg,
	)
	.await;

	connected_clients.write().await.remove(&connected_client);
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum WsModifyAction {
	ToggleTask {
		project_id: ProjectId,
		task_id: TaskId,
		checked: bool,
	},
	CreateTask {
		project_id: ProjectId,
		task_name: String,
	},
}

#[allow(clippy::too_many_arguments)]
async fn handle_ws(
	ws: WebSocket,
	password: String,
	client_addr: SocketAddr,
	database_filepath: PathBuf,
	log_filepath: PathBuf,
	shared_database: Arc<RwLock<Database>>,
	modified_sender: Sender<ModifiedEvent>,
	mut modified_receiver: Receiver<ModifiedEvent>,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
	cpu_usage_avg: Arc<CpuUsageAverage>,
) {
	let (mut write, mut read) = ws.split();

	loop {
		tokio::select! {
			ws_message = read.next() => match ws_message {
				Some(Ok(message)) if message.is_binary() => {
					match bincode::deserialize::<SerializedRequest>(message.as_bytes()) {
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
							send_error_response(ServerError::RequestParseError, &mut write).await;
						},
					}
				}
				Some(Err(e)) => {
					error!("failed to read ws message: {e}")
				}
				None => {
					info!("client disconnected");
					return;
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

					let failed_to_send_msg = send_response(database_modified_response, &mut write).await;

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
	serialized_request: SerializedRequest,
	client_addr: SocketAddr,
	shared_database: &Arc<RwLock<Database>>,
	connected_clients: &Arc<RwLock<HashSet<ConnectedClient>>>,
	cpu_usage_avg: &Arc<CpuUsageAverage>,
	modified_sender: &Sender<ModifiedEvent>,
	ws_write: &mut SplitSink<WebSocket, Message>,
	database_filepath: &PathBuf,
	log_filepath: &PathBuf,
	password: &str,
) {
	if serialized_request.password != password {
		send_error_response(ServerError::InvalidPassword, ws_write).await;
		return;
	}

	match serialized_request.request {
		Request::CheckUpToDate { database_checksum } => {
			info!("sending last modified date");
			let is_up_to_date = database_checksum == shared_database.read().await.checksum();
			if is_up_to_date {
				send_response(Response::DatabaseUpToDate, ws_write).await;
			} else {
				warn!("clients checksum doesnt match ours -> sending full db");
				send_more_up_to_date_database(shared_database, ws_write).await;
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

				send_response(Response::DatabaseUpdated, ws_write).await;

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
				send_more_up_to_date_database(shared_database, ws_write).await;
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

			send_response(Response::DatabaseUpdated, ws_write).await;

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
			send_more_up_to_date_database(shared_database, ws_write).await;
		}
		Request::AdminInfos => {
			info!("sending admin infos");

			send_response(
				Response::AdminInfos(AdminInfos::generate(
					connected_clients.read().await.clone(),
					cpu_usage_avg,
					log_filepath,
				)),
				ws_write,
			)
			.await;
		}
	}
}

async fn send_more_up_to_date_database(
	shared_database: &Arc<RwLock<Database>>,
	ws_write: &mut SplitSink<WebSocket, Message>,
) {
	let (database, last_modified_time) = {
		let shared_database = shared_database.read().await.clone();
		let last_changed_time = *shared_database.last_changed_time();
		(shared_database.into_serialized(), last_changed_time)
	};

	send_response(
		Response::MoreUpToDateDatabase {
			database,
			last_modified_time,
		},
		ws_write,
	)
	.await;
}

/// returns wheter sending failed
async fn send_response(response: Response, ws_write: &mut SplitSink<WebSocket, Message>) -> bool {
	match bincode::serialize::<SerializedResponse>(&Ok(response)) {
		Ok(response_bytes) => match ws_write.send(Message::binary(response_bytes)).await {
			Ok(_) => false,
			Err(e) => {
				error!("failed to send response: {e}");
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
async fn send_error_response(
	error: ServerError,
	ws_write: &mut SplitSink<WebSocket, Message>,
) -> bool {
	match bincode::serialize::<SerializedResponse>(&Err(error)) {
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
