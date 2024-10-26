use std::fs::{File, read_to_string};
use std::path::PathBuf;
use std::process::exit;
use std::thread;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use project_tracker_server::{ModifiedDate, Request, Response, DEFAULT_PORT};

const PORT: usize = DEFAULT_PORT;

fn main() {
	let mut args = std::env::args();

	let filepath_str = args.nth(1).unwrap_or_else(|| {
		eprintln!("usage: project_tracker_server [DATABASE_FILEPATH]");
		exit(1);
	});

	let filepath = PathBuf::from(filepath_str);

	if !filepath.exists() {
		if let Err(e) = File::create(&filepath) {
			eprintln!("failed to create/open database file: {}, error: {e}", filepath.display());
			exit(1);
		}
	}

	let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).expect("Failed to bind to port");

	println!("Server is listening on port {}", PORT);

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				let filepath_clone = filepath.clone();
				thread::spawn(|| handle_client(stream, filepath_clone));
			}
			Err(e) => {
				eprintln!("Failed to establish a connection: {e}");
			}
		}
	}
}


pub fn handle_client(mut stream: TcpStream, filepath: PathBuf) {
	if let Some(request) = read_request(&mut stream) {
		match request {
			Request::GetModifiedDate => {
				use filetime::FileTime;

				match filepath.metadata() {
					Ok(metadata) => {
						let modification_file_time = FileTime::from_last_modification_time(&metadata);

						send_response(&mut stream, &Response::ModifiedDate(ModifiedDate{
							seconds_since_epoch: modification_file_time.unix_seconds() as u64,
						}));
					},
					Err(e) => panic!("cant access database file: {}, error: {e}", filepath.display()),
				}
			},
			Request::UpdateDatabase { database_json } => {
				match std::fs::write(&filepath, database_json) {
					Ok(_) => {
						println!("Updated database file");
					},
					Err(e) => {
						eprintln!("Failed to update database: {e}");
					},
				}
			},
			Request::DownloadDatabase => {
				match read_to_string(&filepath) {
					Ok(database_content) => {
						send_response(&mut stream, &Response::Database {
							database_json: database_content
						});
						println!("Sent database");
					},
					Err(e) => {
						eprintln!("Failed to read {}: {e}", filepath.display());
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