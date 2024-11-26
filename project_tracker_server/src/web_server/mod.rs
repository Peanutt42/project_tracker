use std::path::PathBuf;
use warp::{reply::{Reply, Response}, Filter};

const INDEX_HTML: &str = include_str!("static/index.html");
const SCRIPT_JS: &str = include_str!("static/script.js");
const FAVICON_ICO: &[u8] = include_bytes!("../../../assets/icon.ico");

pub async fn run_web_server(database_filepath: PathBuf, password: String) {
	let get_database_route = warp::path("load_database")
		.and(warp::post())
        .and(warp::body::json())
		.map(move |body: serde_json::Value| {
			load_database(body, &database_filepath, password.clone())
		});

	let index_route = warp::path::end()
		.map(|| warp::reply::html(INDEX_HTML));

	let script_route = warp::path("static")
		.and(warp::path("script.js"))
		.map(|| warp::reply::with_header(SCRIPT_JS, "Content-Type", "application/javascript"));

	let favicon_route = warp::path("favicon.ico")
    	.map(|| warp::reply::with_header(FAVICON_ICO, "Content-Type", "image/x-icon"));

	let routes = index_route
		.or(script_route)
		.or(favicon_route)
		.or(get_database_route);

	println!("Starting web server on port 80 (http)");

	warp::serve(routes)
		.run(([0, 0, 0, 0], 80))
		.await
}

fn load_database(body: serde_json::Value, database_filepath: &PathBuf, password: String) -> Response {
	if body.get("password") == Some(&serde_json::Value::String(password)) {
		match std::fs::read_to_string(database_filepath) {
			Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
				Ok(json) => warp::reply::json(&json).into_response(),
				Err(_) => {
					eprintln!("web-server: database file has invalid json format!");
					warp::reply::with_status(
						warp::reply::html("Database file has invalid json format!".to_string()),
						warp::http::StatusCode::INTERNAL_SERVER_ERROR,
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
				warp::reply::with_status(
					warp::reply::html("Failed to read database file!".to_string()),
					warp::http::StatusCode::INTERNAL_SERVER_ERROR,
				)
				.into_response()
			}
		}
	}
	else {
		println!("web-server: invalid password providied, refusing access!");
		warp::reply::with_status(
			warp::reply::html("Unauthorized".to_string()),
			warp::http::StatusCode::UNAUTHORIZED,
		)
		.into_response()
	}
}