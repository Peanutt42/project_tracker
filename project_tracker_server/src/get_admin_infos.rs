use std::{collections::HashSet, path::PathBuf, sync::Arc};

use project_tracker_server::{AdminInfos, ConnectedClient, CpuUsageAverage};
use tokio::sync::RwLock;
use tracing::info;
use warp::{
	filters::{body, method::post},
	http::StatusCode,
	path,
	reply::{self, html, with_status, Response},
	Filter, Rejection, Reply,
};

pub fn get_admin_infos_route(
	password: String,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
	cpu_usage_avg: Arc<CpuUsageAverage>,
	log_filepath: PathBuf,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
	path!("api" / "admin_infos")
		.and(post())
		.and(body::json())
		.and(warp::any().map(move || password.clone()))
		.and(warp::any().map(move || connected_clients.clone()))
		.and(warp::any().map(move || cpu_usage_avg.clone()))
		.and(warp::any().map(move || log_filepath.clone()))
		.then(get_admin_infos)
}

async fn get_admin_infos(
	body: serde_json::Value,
	password: String,
	connected_clients: Arc<RwLock<HashSet<ConnectedClient>>>,
	cpu_usage_avg: Arc<CpuUsageAverage>,
	log_filepath: PathBuf,
) -> Response {
	if body.get("password") == Some(&serde_json::Value::String(password)) {
		info!("sending admin infos");
		reply::json(&AdminInfos::generate(
			connected_clients.read().await.clone(),
			cpu_usage_avg.as_ref(),
			&log_filepath,
		))
		.into_response()
	} else {
		info!("invalid password, refusing admin infos!");
		with_status(html("Unauthorized".to_string()), StatusCode::UNAUTHORIZED).into_response()
	}
}
