use std::fs::{File, read_to_string};
use std::path::PathBuf;
use std::process::exit;
use std::io::{ErrorKind, Write};
use chrono::{DateTime, Utc};
use project_tracker_server::{get_last_modification_date_time, PasswordHash, Request, RequestType, Response, ServerError, DEFAULT_PASSWORD, DEFAULT_PORT};
use std::net::{TcpListener, TcpStream};

const PORT: usize = DEFAULT_PORT;

fn main() {
	let mut args = std::env::args();

	let server_data_directory_str = args.nth(1).unwrap_or_else(|| {
		eprintln!("usage: project_tracker_server [SERVER_DATA_DIRECTORY]");
		exit(1);
	});

	let server_data_directory = PathBuf::from(server_data_directory_str);

	if !server_data_directory.exists() {
		eprintln!("the supplied directory doesn't exist!");
		exit(1);
	}

	let database_filepath = server_data_directory.join("database.json");
	let password_filepath = server_data_directory.join("password.txt");

	if !database_filepath.exists() {
		if let Err(e) = File::create(&database_filepath) {
			eprintln!("failed to create database file: {}, error: {e}", database_filepath.display());
			exit(1);
		}
	}

	if !password_filepath.exists() {
		match File::create(&password_filepath) {
			Ok(mut file) => {
				if let Err(e) = file.write_all(DEFAULT_PASSWORD.as_bytes()) {
					eprintln!("failed to write default password to password file: {}, error: {e}", password_filepath.display());
					exit(1);
				}
			},
			Err(e) => {
				eprintln!("failed to create default password file: {}, error: {e}", password_filepath.display());
				exit(1);
			}
		}
	}

	let password = read_to_string(&password_filepath)
		.unwrap_or_else(|e| {
			eprintln!("failed to read password file: {}, error: {e}", password_filepath.display());
			exit(1);
		});

	let password_hash = PasswordHash::new(password);

	let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).expect("Failed to bind to port");

	println!("Server is listening on port {}", PORT);

	loop {
		match listener.accept() {
			Ok((stream, _addr)) => {
				let database_filepath_clone = database_filepath.clone();
				let password_hash_clone = password_hash.clone();
				std::thread::spawn(move || listen_client_thread(stream, database_filepath_clone, password_hash_clone));
			}
			Err(e) => {
				eprintln!("Failed to establish a connection: {e}");
			}
		}
	}
}

fn listen_client_thread(mut stream: TcpStream, database_filepath: PathBuf, password_hash: PasswordHash) {
	println!("client connected");

	loop {
		match Request::read(&mut stream) {
			Ok(request) => {
				if request.password_hash != password_hash {
					println!("invalid password");
					if let Err(e) = Response::InvalidPassword.send(&mut stream) {
						eprintln!("failed to send invalid password response to client: {e}");
					}
					return;
				}

				match request.request_type {
					RequestType::GetModifiedDate => {
						if database_filepath.exists() {
							match database_filepath.metadata() {
								Ok(metadata) => {
									if let Err(e) = Response::ModifiedDate(get_last_modification_date_time(&metadata))
										.send(&mut stream)
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
							if let Err(e) = response.send(&mut stream) {
								eprintln!("failed to send modification date to client: {e}");
							}
						}

					},
					RequestType::UpdateDatabase { database_json } => {
						match std::fs::write(&database_filepath, database_json) {
							Ok(_) => {
								println!("Updated database file");
								// TODO: broadcast download database to all other connected clients
							},
							Err(e) => {
								eprintln!("Failed to update database: {e}");
							},
						}
					},
					RequestType::DownloadDatabase => {
						match read_to_string(&database_filepath) {
							Ok(database_json) => {
								let response = Response::Database{ database_json };
								match response.send(&mut stream) {
									Ok(_) => println!("Sent database"),
									Err(e) => eprintln!("failed to send database to client: {e}"),
								}
							},
							Err(e) => {
								eprintln!("Failed to read {}: {e}", database_filepath.display());
							}
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
					_ => eprintln!("failed to read client request: {e}"),
				}
				return;
			},
		}
	}
}