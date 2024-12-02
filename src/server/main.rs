use std::fs::{File, read_to_string};
use std::path::PathBuf;
use std::io::Write;
use std::process::exit;
use std::sync::{Arc, RwLock};
use futures_util::SinkExt;
use project_tracker::core::Database;
use tokio::sync::broadcast::Receiver;
use warp::{body, http::StatusCode, path, path::end, post, reply::{self, html, with_header, with_status, Reply, Response}, serve, ws, ws::{WebSocket, Ws, Message}, Filter};
use project_tracker::server::{run_server, DEFAULT_PASSWORD, DEFAULT_PORT};

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

	let database_filepath = server_data_directory.join("database.project_tracker");
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
				eprintln!("IMPORTANT: Setting default password to {DEFAULT_PASSWORD}! PLEASE change it!");
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

	let (modified_sender, modified_receiver) = tokio::sync::broadcast::channel(10);

	let database_filepath_clone = database_filepath.clone();
	let password_clone = password.clone();
	std::thread::Builder::new()
		.name("Web Server".to_string())
		.spawn(move || {
			let rt = tokio::runtime::Runtime::new().unwrap();

			rt.block_on(async {
				run_web_server(database_filepath_clone, password_clone, modified_receiver).await;
			});
		})
		.expect("failed to start web server thread");

	run_server(DEFAULT_PORT, database_filepath, password, modified_sender);
}

const INDEX_HTML: &str = include_str!("web_server/static/index.html");
const STYLE_CSS: &str = include_str!("web_server/static/style.css");
const SCRIPT_JS: &str = include_str!("web_server/static/script.js");
const FAVICON_ICO: &[u8] = include_bytes!("web_server/static/favicon.ico");

pub async fn run_web_server(database_filepath: PathBuf, password: String, modified_receiver: Receiver<()>) {
	let get_database_route = path("load_database")
		.and(post())
		.and(body::json())
		.map(move |body: serde_json::Value| {
			load_database(body, &database_filepath, password.clone())
		});

	let modified_receiver = Arc::new(RwLock::new(modified_receiver));
	let modified_receiver = warp::any().map(move || modified_receiver.clone());

	let modified_ws_route = path("modified")
		.and(ws())
		.and(modified_receiver)
		.map(|ws: Ws, modified_receiver: Arc<RwLock<Receiver<()>>>| {
			ws.on_upgrade(move |socket| modified_ws_connected(socket, modified_receiver.read().unwrap().resubscribe()))
		});

	let index_route = end()
		.map(|| html(INDEX_HTML));

	let style_route = path("static")
		.and(path("style.css"))
		.map(|| with_header(STYLE_CSS, "Content-Type", "text/css"));

	let script_route = path("static")
		.and(path("script.js"))
		.map(|| with_header(SCRIPT_JS, "Content-Type", "application/javascript"));

	let favicon_route = path("static")
		.and(path("favicon.ico"))
		.map(|| with_header(FAVICON_ICO, "Content-Type", "image/x-icon"));

	let routes = index_route
		.or(style_route)
		.or(script_route)
		.or(favicon_route)
		.or(get_database_route)
		.or(modified_ws_route);

	println!("Starting web server on port 80 (http)");

	serve(routes)
		.run(([0, 0, 0, 0], 80))
		.await
}

fn load_database(body: serde_json::Value, database_filepath: &PathBuf, password: String) -> Response {
	if body.get("password") == Some(&serde_json::Value::String(password)) {
		match std::fs::read(database_filepath) {
			Ok(bin_content) => match bincode::deserialize::<Database>(&bin_content) {
				Ok(db) => reply::json(&db).into_response(),
				Err(_) => {
					eprintln!("web-server: database file has invalid json format!");
					with_status(
						html("Database file has invalid json format!".to_string()),
						StatusCode::INTERNAL_SERVER_ERROR,
					)
					.into_response()
				},
			},
			Err(e) => {
				eprintln!(
					"web-server: failed to read database file in {}: {}",
					database_filepath.display(),
					e
				);
				with_status(
					html("Failed to read database file!".to_string()),
					StatusCode::INTERNAL_SERVER_ERROR,
				)
				.into_response()
			}
		}
	}
	else {
		println!("web-server: invalid password providied, refusing access!");
		with_status(
			html("Unauthorized".to_string()),
			StatusCode::UNAUTHORIZED,
		)
		.into_response()
	}
}

async fn modified_ws_connected(mut ws: WebSocket, mut modified_receiver: Receiver<()>) {
	loop {
		match modified_receiver.recv().await {
			Ok(()) => {
				if let Err(e) = ws.send(Message::text(String::new())).await {
					eprintln!("failed to send modified event to ws client: {e}");
				}
			},
			Err(e) => panic!("failed to receive further database modified events: {e}"),
		};
	}
}