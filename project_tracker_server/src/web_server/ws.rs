use futures_util::{SinkExt, StreamExt};
use project_tracker_core::{Database, DatabaseMessage, ProjectId, TaskId};
use project_tracker_server::{
	save_database_to_file, ConnectedClient, DatabaseUpdateEvent, ModifiedEvent,
};
use serde::{Deserialize, Serialize};
use std::{
	collections::{BTreeSet, HashSet},
	net::SocketAddr,
	path::PathBuf,
	sync::Arc,
};
use tokio::sync::{
	broadcast::{Receiver, Sender},
	RwLock,
};
use tracing::{error, info, warn};
use warp::{
	filters::ws::{ws, Message, WebSocket, Ws},
	path, Filter, Rejection, Reply,
};

pub fn ws_route(
	database_filepath: PathBuf,
	shared_database: Arc<RwLock<Database>>,
	modified_sender: Sender<ModifiedEvent>,
	modified_receiver: Receiver<ModifiedEvent>,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
	password: String,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
	let modified_sender = Arc::new(RwLock::new(modified_sender));
	let modified_receiver = Arc::new(RwLock::new(modified_receiver));

	path("ws")
		.and(ws())
		.and(warp::addr::remote())
		.and(warp::any().map(move || database_filepath.clone()))
		.and(warp::any().map(move || shared_database.clone()))
		.and(warp::any().map(move || modified_sender.clone()))
		.and(warp::any().map(move || modified_receiver.clone()))
		.and(warp::any().map(move || connected_clients.clone()))
		.and(warp::any().map(move || password.clone()))
		.then(
			move |ws: Ws,
			      client_addr: Option<SocketAddr>,
			      database_filepath: PathBuf,
			      shared_database: Arc<RwLock<Database>>,
			      modified_sender: Arc<RwLock<Sender<ModifiedEvent>>>,
			      modified_receiver: Arc<RwLock<Receiver<ModifiedEvent>>>,
			      connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
			      password: String| {
				async move {
					let modified_sender = modified_sender.read().await.clone();
					let modified_receiver = modified_receiver.read().await.resubscribe();

					ws.on_upgrade(move |socket| {
						on_upgrade_ws(
							socket,
							password,
							client_addr,
							database_filepath,
							shared_database,
							modified_sender,
							modified_receiver,
							connected_clients,
						)
					})
				}
			},
		)
}

#[allow(clippy::too_many_arguments)]
async fn on_upgrade_ws(
	mut ws: WebSocket,
	password: String,
	client_addr: Option<SocketAddr>,
	database_filepath: PathBuf,
	shared_database: Arc<RwLock<Database>>,
	modified_sender: Sender<ModifiedEvent>,
	modified_receiver: Receiver<ModifiedEvent>,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
) {
	let client_addr = match client_addr {
		Some(client_addr) => client_addr,
		None => {
			warn!("client_addr was not specified, ignoring ws connection");
			return;
		}
	};

	#[derive(Deserialize)]
	struct AuthenticateJson {
		password: String,
	}

	#[derive(Serialize)]
	struct AuthenticationResponse {
		successfull: bool,
	}

	// wait until client sends the correct password
	loop {
		if let Some(Ok(message)) = ws.next().await {
			if let Ok(msg_text) = message.to_str() {
				if let Ok(json_msg) = serde_json::from_str::<AuthenticateJson>(msg_text) {
					let successfull = json_msg.password == password;
					let _ = ws
						.send(Message::text(
							serde_json::to_string(&AuthenticationResponse { successfull }).unwrap(),
						))
						.await;
					if successfull {
						break;
					} else {
						info!("invalid password, refusing modified ws access");
					}
				}
			}
		}
	}

	let connected_client = ConnectedClient::Web(client_addr);

	connected_clients.write().await.insert(connected_client);

	handle_ws(
		ws,
		client_addr,
		database_filepath,
		shared_database,
		modified_sender,
		modified_receiver,
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

async fn handle_ws(
	ws: WebSocket,
	client_addr: SocketAddr,
	database_filepath: PathBuf,
	shared_database: Arc<RwLock<Database>>,
	modified_sender: Sender<ModifiedEvent>,
	mut modified_receiver: Receiver<ModifiedEvent>,
) {
	let (mut write_ws, mut read_ws) = ws.split();

	info!("modified ws client connected");

	loop {
		tokio::select! {
			modified_event_result = modified_receiver.recv() => {
				match modified_event_result {
					Ok(modified_event) => {
						match modified_event.modified_database.to_json() {
							Some(database_json) => {
								info!("sending database modified event in ws");
								if let Err(e) = write_ws.send(Message::text(database_json)).await {
									error!("failed to send modified event: {e}");
									return;
								}
							},
							None => error!("failed to serialize database in order to send to ws clients"),
						}
					},
					Err(e) => {
						error!("failed to receive further database modified events: {e}");
						return;
					},
				}
			},
			message = read_ws.next() => {
				match message {
					Some(Ok(message)) => {
						match message.to_str() {
							Ok(message_str) => match serde_json::from_str::<WsModifyAction>(message_str) {
								Ok(action) => {
									info!("{action:?}");
									let database_message = match action {
										WsModifyAction::ToggleTask { project_id, task_id, checked } => if checked {
											DatabaseMessage::SetTaskDone { project_id, task_id }
										} else {
											DatabaseMessage::SetTaskTodo { project_id, task_id }
										},
										WsModifyAction::CreateTask { project_id, task_name } => {
											DatabaseMessage::CreateTask {
												project_id,
												task_id: TaskId::generate(),
												task_name,
												task_description: String::new(),
												task_tags: BTreeSet::new(),
												due_date: None,
												needed_time_minutes: None,
												time_spend: None,
												create_at_top: true
											}
										}
									};
									let (modified_database, before_modification_checksum) = {
										let mut shared_database = shared_database.write().await;
										let before_modification_checksum = shared_database.checksum();
										shared_database.update(database_message.clone());
										(shared_database.clone(), before_modification_checksum)
									};
									let database_binary = modified_database.to_binary();
									let _ = modified_sender.send(ModifiedEvent::new(
										modified_database,
										DatabaseUpdateEvent::DatabaseMessage {
											database_messages: vec![database_message],
											before_modification_checksum
										},
										client_addr
									));
									match database_binary {
										Some(database_binary) => {
											save_database_to_file(&database_filepath, &database_binary).await;
										}
										None => {
											error!("failed to serialize database to binary -> cant save database to file");
										}
									}
								},
								Err(e) => error!("failed to parse action from web ws: {e}"),
							}
							Err(_) => warn!("web ws only accepts modification actions as json string messages"),
						}
					}
					Some(Err(e)) => {
						warn!("web ws connection closed: {e}");
						let _ = write_ws.close().await;
						return;
					}
					None => {
						info!("web ws connection closed");
						let _ = write_ws.close().await;
						return;
					}
				}
			},
		};
	}
}
