use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::io::ErrorKind;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast::Sender;
use project_tracker_core::Database;
use crate::{Request, Response, ServerError, SharedServerData};

pub fn run_server(port: usize, database_filepath: PathBuf, password: String, modified_sender: Sender<()>, shared_data: Arc<RwLock<SharedServerData>>) {
	let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind to port");

	println!("Server is listening on port {}", port);

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

fn listen_client_thread(mut stream: TcpStream, database_filepath: PathBuf, password: String, modified_sender: Sender<()>, shared_data: Arc<RwLock<SharedServerData>>) {
	println!("client connected");

	loop {
		match Request::read(&mut stream, &password) {
			Ok(request) => match request {
				Request::GetModifiedDate => {
					if let Err(e) = Response::ModifiedDate(shared_data.read().unwrap().last_modified_time)
						.send(&mut stream, &password)
					{
						eprintln!("failed to send modified date response to client: {e}");
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
							let _ = Response::DatabaseUpdated.send(&mut stream, &password);
							let _ = modified_sender.send(());
							println!("Updated database file");
						},
						Err(e) => {
							eprintln!("failed to parse database binary of client: {e}");
							let _ = Response::InvalidDatabaseBinary.send(&mut stream, &password);
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
							match response.send(&mut stream, &password) {
								Ok(_) => println!("Sent database"),
								Err(e) => eprintln!("failed to send database to client: {e}"),
							}
						},
						None => {
							eprintln!("Failed to serialize database to binary in order to send to client");
							let _ = Response::InvalidDatabaseBinary.send(&mut stream, &password);
						}
					}
				}
			},
			Err(e) => {
				match e {
					ServerError::ConnectionError(e) if matches!(
						e.kind(),
						ErrorKind::UnexpectedEof |
						ErrorKind::ConnectionAborted |
						ErrorKind::ConnectionReset
					) => println!("client disconnected"),
					ServerError::InvalidPassword => {
						println!("invalid password");
						let _ = Response::InvalidPassword.send(&mut stream, &password);
					},
					_ => eprintln!("failed to read client request: {e}"),
				}
				return;
			},
		}
	}
}