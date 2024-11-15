use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::io::ErrorKind;
use std::fs::read_to_string;
use chrono::{DateTime, Utc};
use crate::{get_last_modification_date_time, Request, Response, ServerError};


pub fn run_server(port: usize, database_filepath: PathBuf, password: String) {
	let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind to port");

	println!("Server is listening on port {}", port);

	loop {
		match listener.accept() {
			Ok((stream, _addr)) => {
				let database_filepath_clone = database_filepath.clone();
				let password_clone = password.clone();
				std::thread::spawn(move || listen_client_thread(stream, database_filepath_clone, password_clone));
			}
			Err(e) => {
				eprintln!("Failed to establish a connection: {e}");
			}
		}
	}
}

fn listen_client_thread(mut stream: TcpStream, database_filepath: PathBuf, password: String) {
	println!("client connected");

	loop {
		match Request::read(&mut stream, &password) {
			Ok(request) => match request {
				Request::GetModifiedDate => {
					if database_filepath.exists() {
						match database_filepath.metadata() {
							Ok(metadata) => {
								if let Err(e) = Response::ModifiedDate(get_last_modification_date_time(&metadata))
									.send(&mut stream, &password)
								{
									eprintln!("failed to send modified date response to client: {e}");
								}
							},
							Err(e) => panic!("cant access database file: {}, error: {e}", database_filepath.display()),
						}
					}
					else {
						// as the server doesn't have any database saved, any database of the client is more
						// MIN_UTC is the oldest possible Date
						// -> client will send the database
						let response = Response::ModifiedDate(DateTime::<Utc>::MIN_UTC);
						if let Err(e) = response.send(&mut stream, &password) {
							eprintln!("failed to send modification date to client: {e}");
						}
					}

				},
				Request::UpdateDatabase { database_json } => match std::fs::write(&database_filepath, database_json) {
					Ok(_) => {
						println!("Updated database file");
						// TODO: broadcast download database to all other connected clients
						let _ = Response::DatabaseUpdated.send(&mut stream, &password);
					},
					Err(e) => panic!("cant write to database file: {}, error: {e}", database_filepath.display()),
				},
				Request::DownloadDatabase => {
					match read_to_string(&database_filepath) {
						Ok(database_json) => {
							let response = Response::Database{ database_json };
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