use std::sync::Arc;

use project_tracker_core::Database;
use tokio::sync::RwLock;
use tracing::info;
use warp::{
	filters::{body, method::post},
	http::StatusCode,
	path,
	reply::{self, html, with_status, Response},
	Filter, Rejection, Reply,
};

pub fn load_database_route(
	password: String,
	shared_database: Arc<RwLock<Database>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
	path("load_database")
		.and(post())
		.and(body::json())
		.and(warp::any().map(move || password.clone()))
		.and(warp::any().map(move || shared_database.clone()))
		.then(load_database)
}

async fn load_database(
	body: serde_json::Value,
	password: String,
	shared_database: Arc<RwLock<Database>>,
) -> Response {
	if body.get("password") == Some(&serde_json::Value::String(password)) {
		info!("sending database as json");
		reply::json(shared_database.read().await.serialized()).into_response()
	} else {
		info!("invalid password providied, refusing access!");
		with_status(html("Unauthorized".to_string()), StatusCode::UNAUTHORIZED).into_response()
	}
}
