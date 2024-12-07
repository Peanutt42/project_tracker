use project_tracker_server::SharedServerData;
use warp::{body, http::StatusCode, path, path::end, post, reply::{self, html, with_header, with_status, Reply, Response}, serve, ws, ws::{WebSocket, Ws, Message}, Filter};
use futures_util::SinkExt;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast::Receiver;

const INDEX_HTML: &str = include_str!("static/index.html");
const STYLE_CSS: &str = include_str!("static/style.css");
const SCRIPT_JS: &str = include_str!("static/script.js");
const FAVICON_ICO: &[u8] = include_bytes!("static/favicon.ico");

pub async fn run_web_server(password: String, modified_receiver: Receiver<()>, shared_data: Arc<RwLock<SharedServerData>>) {
	let get_database_route = path("load_database")
		.and(post())
		.and(body::json())
		.map(move |body: serde_json::Value| {
			load_database(body, password.clone(), shared_data.clone())
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

fn load_database(body: serde_json::Value, password: String, shared_data: Arc<RwLock<SharedServerData>>) -> Response {
	if body.get("password") == Some(&serde_json::Value::String(password)) {
		reply::json(&shared_data.read().unwrap().database.clone().to_serialized()).into_response()
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