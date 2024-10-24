use project_tracker_server::{ModifiedDate, Request, Response, ServerEvent, DEFAULT_PORT};
use tokio::net::TcpStream;
use std::fs::read_to_string;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, tcp::{OwnedReadHalf, OwnedWriteHalf}};

const PORT: usize = DEFAULT_PORT;

async fn read_request(mut stream: OwnedReadHalf) -> Option<Request> {
	let mut json = String::new();

	if let Err(e) = stream.read_to_string(&mut json).await {
		eprintln!("Failed to get client request: {e}");
		return None;
	}

	match serde_json::from_str(&json) {
		Ok(request) => Some(request),
		Err(e) => {
			eprintln!("Failed to parse client request: {e}");
			None
		}
	}
}

async fn send_response(mut stream: OwnedWriteHalf, response: &Response) {
	let response_json = match serde_json::to_string(response) {
		Ok(json) => json,
		Err(e) => {
			eprintln!("Failed to serialize response to json: {e}");
			return;
		}
	};

	if let Err(e) = stream.write_all(response_json.as_bytes()).await {
		eprintln!("Failed to send response to client: {e}");
	}
}

pub async fn handle_client(stream: TcpStream, filepath: PathBuf) -> Option<ServerEvent> {
	let (read_stream, write_stream) = stream.into_split();

	if let Some(request) = read_request(read_stream).await {
		match request {
			Request::GetModifiedDate => {
				use filetime::FileTime;

				match filepath.metadata() {
					Ok(metadata) => {
						let modification_file_time = FileTime::from_last_modification_time(&metadata);

						send_response(write_stream, &Response::ModifiedDate(ModifiedDate{
							seconds_since_epoch: modification_file_time.unix_seconds() as u64,
						})).await;
						None
					},
					Err(e) => panic!("cant access database file: {}, error: {e}", filepath.display()),
				}
			},
			Request::UpdateDatabase { database_json } => {
				match std::fs::write(&filepath, database_json) {
					Ok(_) => {
						println!("Updated database file");
						Some(ServerEvent::UpdatedDatabase)
					},
					Err(e) => {
						let error_msg = format!("Failed to update database: {e}");
						eprintln!("{error_msg}");
						Some(ServerEvent::Error(error_msg))
					},
				}
			},
			Request::DownloadDatabase => {
				match read_to_string(&filepath) {
					Ok(database_content) => {
						send_response(write_stream, &Response::Database {
							database_json: database_content
						}).await;
						println!("Sent database");
						Some(ServerEvent::SentDatabase)
					},
					Err(e) => {
						let error_msg = format!("Failed to read {}: {e}", filepath.display());
						eprintln!("{error_msg}");
						Some(ServerEvent::Error(error_msg))
					}
				}
			}
		}
	}
	else {
		None
	}
}

pub async fn create_server() -> TcpListener {
	let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).await.expect("Failed to bind to port");

	println!("Server is listening on port {}", PORT);

	listener
}