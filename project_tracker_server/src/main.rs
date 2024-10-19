use project_tracker_server::{ModifiedDate, Request, Response, DEFAULT_PORT};
use std::fs::read_to_string;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::thread;

const PORT: usize = DEFAULT_PORT;
const FILEPATH: &str = "database.json";

fn read_request(stream: &mut TcpStream) -> Option<Request> {
	let mut json = String::new();

	if let Err(e) = stream.read_to_string(&mut json) {
		eprintln!("Failed to get client request: {e}");
		return None;
	}

	let _ = stream.shutdown(std::net::Shutdown::Read);

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

	let _ = stream.shutdown(std::net::Shutdown::Write);
}

fn handle_client(mut stream: TcpStream) {
	if let Some(request) = read_request(&mut stream) {
		match request {
			Request::GetModifiedDate => {
				use filetime::FileTime;

				match PathBuf::from(FILEPATH).metadata() {
					Ok(metadata) => {
						let modification_file_time = FileTime::from_last_modification_time(&metadata);

						send_response(&mut stream, &Response::ModifiedDate(ModifiedDate{
							seconds_since_epoch: modification_file_time.unix_seconds() as u64,
						}));
					},
					Err(e) => panic!("cant access database file: {}, error: {e}", FILEPATH),
				}
			},
			Request::UpdateDatabase { database_json } => {
				match std::fs::write(FILEPATH, database_json) {
					Ok(_) => println!("Updated database file"),
					Err(e) => eprintln!("Failed to update database: {e}"),
				}
			},
			Request::DownloadDatabase => {
				match read_to_string(FILEPATH) {
					Ok(database_content) => {
						send_response(&mut stream, &Response::Database {
							database_json: database_content
						});
						println!("Sent database");
					},
					Err(e) => {
						eprintln!("Failed to read {}: {e}", FILEPATH);
					}
				}
			}
		}
	}
}

fn main() {
	let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).expect("Failed to bind to port");

	println!("Server is listening on port {}", PORT);

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				thread::spawn(move || {
					handle_client(stream);
				});
			}
			Err(e) => {
				eprintln!("Failed to establish a connection: {e}");
			}
		}
	}
}
