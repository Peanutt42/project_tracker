use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use async_tungstenite::{tungstenite::{self, Message}, tokio::accept_async};
use futures_util::{SinkExt, StreamExt};
use project_tracker_core::Database;
use crate::{Request, Response, SharedServerData};

pub async fn run_server(port: usize, database_filepath: PathBuf, password: String, modified_sender: Sender<SharedServerData>, shared_data: Arc<RwLock<SharedServerData>>) {
	let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.expect("Failed to bind to port");

	println!("WebServer is listening on port {}", port);

	loop {
		match listener.accept().await {
			Ok((stream, _addr)) => {
				let database_filepath_clone = database_filepath.clone();
				let password_clone = password.clone();
				let modified_sender_clone = modified_sender.clone();
				let modified_receiver = modified_sender.subscribe();
				let shared_data_clone = shared_data.clone();
				tokio::spawn(async move {
					listen_client_thread(stream, database_filepath_clone, password_clone, modified_sender_clone, modified_receiver, shared_data_clone).await
				});
			}
			Err(e) => {
				eprintln!("Failed to establish a connection: {e}");
			}
		}
	}
}

async fn listen_client_thread(stream: TcpStream, database_filepath: PathBuf, password: String, modified_sender: Sender<SharedServerData>, mut modified_receiver: Receiver<SharedServerData>, shared_data: Arc<RwLock<SharedServerData>>) {
	let ws_stream = match accept_async(stream).await {
		Ok(ws) => {
			println!("client connected");
			ws
		},
		Err(e) => {
			eprintln!("WebSocket Handshake failed: {e}");
			return;
		}
	};

	let (mut write, mut read) = ws_stream.split();

	let (write_ws_sender, mut write_ws_receiver) = mpsc::channel(10);
	// write ws socket thread
	tokio::spawn(async move {
		while let Some(message) = write_ws_receiver.recv().await {
			if let Err(e) = write.send(message).await {
				eprintln!("failed to send response: {e}");
			}
		}
	});

	let password_clone = password.clone();
	let write_ws_sender_clone = write_ws_sender.clone();
	tokio::spawn(async move {
		while let Ok(shared_data) = modified_receiver.recv().await {
			if write_ws_sender_clone.send(Message::binary(
				Response::Database{
					database_binary: shared_data.database.to_binary().unwrap(),
					last_modified_time: shared_data.last_modified_time
				}
				.encrypt(&password_clone).unwrap()
			))
			.await
			.is_err()
			{
				break;
			}
		}
	});

	loop {
		match read.next().await {
			Some(Ok(tungstenite::Message::Binary(request_binary))) => match Request::decrypt(request_binary, &password) {
				Ok(request) => {
					match request {
						Request::GetModifiedDate => {
							println!("sending last modified date");
							let response = Response::ModifiedDate(shared_data.read().unwrap().last_modified_time)
								.encrypt(&password)
								.unwrap();
							if let Err(e) = write_ws_sender.send(Message::binary(response)).await {
								eprintln!("failed to respond to 'GetModifiedDate' request: {e}");
							}
						},
						Request::UpdateDatabase { database_binary, last_modified_time } => {
							match Database::from_binary(&database_binary, last_modified_time) {
								Ok(database) => {
									let shared_data_clone = {
										let mut shared_data = shared_data.write().unwrap();
										shared_data.last_modified_time = last_modified_time;
										shared_data.database = database;
										shared_data.clone()
									};

									if let Err(e) = std::fs::write(&database_filepath, database_binary) {
										panic!("cant write to database file: {}, error: {e}", database_filepath.display());
									}

									let _ = write_ws_sender.send(Message::binary(
										Response::DatabaseUpdated
											.encrypt(&password)
											.unwrap()
									))
									.await;
									let _ = modified_sender.send(shared_data_clone);
									println!("Updated database file");
								},
								Err(e) => {
									eprintln!("failed to parse database binary of client: {e}");
									let _ = write_ws_sender.send(Message::binary(
										Response::InvalidDatabaseBinary
											.encrypt(&password)
											.unwrap()
									))
									.await;
								},
							};
						},
						Request::DownloadDatabase => {
							let shared_data_clone = shared_data.read().unwrap().clone();
							match shared_data_clone.database.to_binary() {
								Some(database_binary) => {
									let response = Response::Database{
										database_binary,
										last_modified_time: shared_data_clone.last_modified_time
									};
									match write_ws_sender.send(
										Message::binary(response.encrypt(&password).unwrap())
									)
									.await
									{
										Ok(_) => println!("Sent database"),
										Err(e) => eprintln!("failed to send database to client: {e}"),
									}
								},
								None => {
									eprintln!("Failed to serialize database to binary in order to send to client");
									let _ = write_ws_sender.send(Message::binary(
										Response::InvalidDatabaseBinary
											.encrypt(&password)
											.unwrap()
									))
									.await;
								}
							}
						}
					}
				},
				Err(e) => eprintln!("failed to parse ws request: {e}"),
			},
			Some(Err(ref e)) if matches!(
				e,
				tungstenite::Error::ConnectionClosed |
				tungstenite::Error::AlreadyClosed |
				tungstenite::Error::Protocol(tungstenite::error::ProtocolError::ResetWithoutClosingHandshake)
			) => {
				println!("client disconnected");
				return;
			},
			Some(Err(e)) => eprintln!("failed to read ws message: {e}"),
			_ => {}, // ignore
		}
	}
}