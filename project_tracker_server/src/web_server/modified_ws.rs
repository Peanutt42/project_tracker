use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use project_tracker_server::{ConnectedClient, ModifiedEvent};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast::Receiver, RwLock};
use tracing::{error, info};
use warp::{
	filters::ws::{ws, Message, WebSocket, Ws},
	path, Filter, Rejection, Reply,
};

pub fn modified_ws_route(
	modified_receiver: Receiver<ModifiedEvent>,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
	password: String,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
	let modified_receiver = Arc::new(RwLock::new(modified_receiver));

	path("modified_ws")
		.and(ws())
		.and(warp::addr::remote())
		.and(warp::any().map(move || modified_receiver.clone()))
		.and(warp::any().map(move || connected_clients.clone()))
		.and(warp::any().map(move || password.clone()))
		.then(
			move |ws: Ws,
			      client_addr: Option<SocketAddr>,
			      modified_receiver: Arc<RwLock<Receiver<ModifiedEvent>>>,
			      connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
			      password: String| {
				async move {
					let modified_receiver = modified_receiver.read().await.resubscribe();

					ws.on_upgrade(move |socket| {
						on_upgrade_modified_ws(
							socket,
							password,
							client_addr,
							modified_receiver,
							connected_clients,
						)
					})
				}
			},
		)
}

async fn on_upgrade_modified_ws(
	mut ws: WebSocket,
	password: String,
	client_addr: Option<SocketAddr>,
	modified_receiver: Receiver<ModifiedEvent>,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
) {
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

	let connected_client = client_addr.map(ConnectedClient::Web);

	if let Some(connected_client) = connected_client {
		connected_clients.write().await.insert(connected_client);
	}

	handle_modified_ws(ws, modified_receiver).await;

	if let Some(connected_client) = connected_client {
		connected_clients.write().await.remove(&connected_client);
	}
}

async fn handle_modified_ws(ws: WebSocket, mut modified_receiver: Receiver<ModifiedEvent>) {
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
				if matches!(message, None | Some(Err(_))) {
					info!("modified ws connection closed");
					let _ = write_ws.close().await;
					return;
				}
			},
		};
	}
}
