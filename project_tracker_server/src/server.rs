use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::io::ErrorKind;
use std::fs::read;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use tokio::sync::broadcast::Sender;
use project_tracker_core::get_last_modification_date_time;
use crate::{Request, Response, ServerError};


pub fn run_server(port: usize, database_filepath: PathBuf, password: String, modified_sender: Sender<()>) {
	let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind to port");

	let last_modified_database_file_time = get_last_modification_date_time(
		&database_filepath.metadata().expect("Failed to get the last modified metadata of database file")
	)
	.expect("Failed to get the last modified metadata of database file");
	let last_modified_database_time = Arc::new(RwLock::new(last_modified_database_file_time));

	println!("Server is listening on port {}", port);

	loop {
		match listener.accept() {
			Ok((stream, _addr)) => {
				let database_filepath_clone = database_filepath.clone();
				let password_clone = password.clone();
				let modified_sender_clone = modified_sender.clone();
				let last_modified_database_time_clone = last_modified_database_time.clone();
				std::thread::spawn(move || listen_client_thread(stream, database_filepath_clone, password_clone, modified_sender_clone, last_modified_database_time_clone));
			}
			Err(e) => {
				eprintln!("Failed to establish a connection: {e}");
			}
		}
	}
}

fn listen_client_thread(mut stream: TcpStream, database_filepath: PathBuf, password: String, modified_sender: Sender<()>, last_modified_database_time: Arc<RwLock<DateTime<Utc>>>) {
	println!("client connected");

	loop {
		match Request::read(&mut stream, &password) {
			Ok(request) => match request {
				Request::GetModifiedDate => {
					if let Err(e) = Response::ModifiedDate(*last_modified_database_time.read().unwrap())
						.send(&mut stream, &password)
					{
						eprintln!("failed to send modified date response to client: {e}");
					}
				},
				Request::UpdateDatabase { database_binary, last_modified_time } => {
					*last_modified_database_time.write().unwrap() = last_modified_time;
					match std::fs::write(&database_filepath, database_binary) {
						Ok(_) => {
							println!("Updated database file");
							// TODO: broadcast download database to all other connected clients (ws gui clients)
							let _ = Response::DatabaseUpdated.send(&mut stream, &password);
							let _ = modified_sender.send(());
						},
						Err(e) => panic!("cant write to database file: {}, error: {e}", database_filepath.display()),
					}
				},
				Request::DownloadDatabase => {
					match read(&database_filepath) {
						Ok(database_binary) => {
							let response = Response::Database{ database_binary };
							match response.send(&mut stream, &password) {
								Ok(_) => println!("Sent database"),
								Err(e) => eprintln!("failed to send database to client: {e}"),
							}
						},
						Err(e) => {
							eprintln!("Failed to read {}: {e}", database_filepath.display());
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