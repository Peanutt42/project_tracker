use std::fs::{File, read_to_string};
use std::path::PathBuf;
use std::process::exit;
use std::thread;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use project_tracker_server::{hash_password, ModifiedDate, Request, RequestType, Response, DEFAULT_PASSWORD, DEFAULT_PORT};

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

	let password_hash = hash_password(password);

	let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).expect("Failed to bind to port");

	println!("Server is listening on port {}", PORT);

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				let database_filepath_clone = database_filepath.clone();
				let password_hash_clone = password_hash.clone();
				thread::spawn(|| handle_client(stream, database_filepath_clone, password_hash_clone));
			}
			Err(e) => {
				eprintln!("Failed to establish a connection: {e}");
			}
		}
	}
}


pub fn handle_client(mut stream: TcpStream, database_filepath: PathBuf, password_hash: String) {
	if let Some(request) = read_request(&mut stream) {
		if request.password_hash != password_hash {
			send_response(&mut stream, &Response::InvalidPassword);
			println!("invalid password");
			return;
		}

		match request.request_type {
			RequestType::GetModifiedDate => {
				use filetime::FileTime;

				match database_filepath.metadata() {
					Ok(metadata) => {
						let modification_file_time = FileTime::from_last_modification_time(&metadata);

						send_response(&mut stream, &Response::ModifiedDate(ModifiedDate{
							seconds_since_epoch: modification_file_time.unix_seconds() as u64,
						}));
					},
					Err(e) => panic!("cant access database file: {}, error: {e}", database_filepath.display()),
				}
			},
			RequestType::UpdateDatabase { database_json } => {
				match std::fs::write(&database_filepath, database_json) {
					Ok(_) => {
						println!("Updated database file");
					},
					Err(e) => {
						eprintln!("Failed to update database: {e}");
					},
				}
			},
			RequestType::DownloadDatabase => {
				match read_to_string(&database_filepath) {
					Ok(database_content) => {
						send_response(&mut stream, &Response::Database {
							database_json: database_content
						});
						println!("Sent database");
					},
					Err(e) => {
						eprintln!("Failed to read {}: {e}", database_filepath.display());
					}
				}
			}
		}
	}
}

fn read_request(stream: &mut TcpStream) -> Option<Request> {
	let mut json = String::new();

	if let Err(e) = stream.read_to_string(&mut json) {
		eprintln!("Failed to get client request: {e}");
		return None;
	}

	if let Err(e) = stream.shutdown(Shutdown::Read) {
		eprintln!("Failed to shutdown reading half of the stream: {e}");
	}

	match serde_json::from_str(&json) {
		Ok(request) => Some(request),
		Err(e) => {
			eprintln!("Failed to parse client request: {e}");
			None
		}
	}
}

fn send_response(stream: &mut TcpStream, response: &Response) {
	let response_json = match serde_json::to_string(response) {
		Ok(json) => json,
		Err(e) => {
			eprintln!("Failed to serialize response to json: {e}");
			return;
		}
	};

	if let Err(e) = stream.write_all(response_json.as_bytes()) {
		eprintln!("Failed to send response to client: {e}");
	}

	if let Err(e) = stream.shutdown(Shutdown::Write) {
		eprintln!("Failed to shutdown writing half of the stream: {e}");
	}
}