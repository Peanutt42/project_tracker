use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use async_tungstenite::tungstenite::{self, Message};
use tokio::sync::broadcast::Sender;
use project_tracker_core::Database;
use crate::{Request, Response, SharedServerData};

pub fn run_server(port: usize, database_filepath: PathBuf, password: String, modified_sender: Sender<()>, shared_data: Arc<RwLock<SharedServerData>>) {
	let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind to port");

	println!("WebServer is listening on port {}", port);

	loop {
		match listener.accept() {
			Ok((stream, _addr)) => {
				let database_filepath_clone = database_filepath.clone();
				let password_clone = password.clone();
				let modified_sender_clone = modified_sender.clone();
				let shared_data_clone = shared_data.clone();
				std::thread::spawn(move || listen_client_thread(stream, database_filepath_clone, password_clone, modified_sender_clone, shared_data_clone));
			}
			Err(e) => {
				eprintln!("Failed to establish a connection: {e}");
			}
		}
	}
}

fn listen_client_thread(stream: TcpStream, database_filepath: PathBuf, password: String, modified_sender: Sender<()>, shared_data: Arc<RwLock<SharedServerData>>) {
	let mut ws_stream = match tungstenite::accept(stream) {
		Ok(ws) => {
			println!("client connected");
			ws
		},
		Err(e) => {
			eprintln!("WebSocket Handshake failed: {e}");
			return;
		}
	};

	loop {
		match ws_stream.read() {
			Ok(tungstenite::Message::Binary(request_binary)) => match Request::decrypt(request_binary, &password) {
				Ok(request) => {
					match request {
						Request::GetModifiedDate => {
							println!("sending last modified date");
							if let Err(e) = ws_stream.send(Message::binary(
								Response::ModifiedDate(shared_data.read().unwrap().last_modified_time)
									.encrypt(&password)
        							.unwrap()
							))
							{
								eprintln!("failed to respond to 'GetModifiedDate' request: {e}");
							}
						},
						Request::UpdateDatabase { database_binary, last_modified_time } => {
							match Database::from_binary(&database_binary, last_modified_time) {
								Ok(database) => {
									{
										let mut shared_data = shared_data.write().unwrap();
										shared_data.last_modified_time = last_modified_time;
										shared_data.database = database;
									}

									if let Err(e) = std::fs::write(&database_filepath, database_binary) {
										panic!("cant write to database file: {}, error: {e}", database_filepath.display());
									}

									// TODO: broadcast download database to all other connected clients (ws gui clients)
									let _ = ws_stream.send(Message::binary(
										Response::DatabaseUpdated
											.encrypt(&password)
											.unwrap()
									));
									let _ = modified_sender.send(());
									println!("Updated database file");
								},
								Err(e) => {
									eprintln!("failed to parse database binary of client: {e}");
									let _ = ws_stream.send(Message::binary(
										Response::InvalidDatabaseBinary
											.encrypt(&password)
											.unwrap()
									));
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
									match ws_stream.send(
										Message::binary(response.encrypt(&password).unwrap())
									)
									{
										Ok(_) => println!("Sent database"),
										Err(e) => eprintln!("failed to send database to client: {e}"),
									}
								},
								None => {
									eprintln!("Failed to serialize database to binary in order to send to client");
									let _ = ws_stream.send(Message::binary(
										Response::InvalidDatabaseBinary
											.encrypt(&password)
											.unwrap()
									));
								}
							}
						}
					}
				},
				Err(e) => eprintln!("failed to parse ws request: {e}"),
			},
			Err(ref e) if matches!(
				e,
				tungstenite::Error::ConnectionClosed |
				tungstenite::Error::AlreadyClosed |
				tungstenite::Error::Protocol(tungstenite::error::ProtocolError::ResetWithoutClosingHandshake)
			) => {
				println!("client disconnected");
				return;
			},
			Err(e) => eprintln!("failed to read ws message: {e}"),
			_ => {}, // ignore
		}
	}
}