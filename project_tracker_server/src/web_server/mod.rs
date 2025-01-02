use futures_util::{SinkExt, StreamExt};
use humantime::format_duration;
use project_tracker_server::{ConnectedClient, ModifiedEvent, SharedServerData};
use serde::Serialize;
use std::{
	convert::Infallible,
	net::SocketAddr,
	path::PathBuf,
	process::{Command, Stdio},
	str::from_utf8,
	sync::{Arc, RwLock},
};
use systemstat::{saturating_sub_bytes, Platform, System};
use tokio::sync::broadcast::Receiver;
use tracing::{error, info};
use warp::{
	body,
	filters::{
		body::BodyDeserializeError, cors::CorsForbidden, ext::MissingExtension,
		ws::MissingConnectionUpgrade,
	},
	http::StatusCode,
	path,
	path::end,
	post,
	reject::{
		InvalidHeader, InvalidQuery, LengthRequired, MethodNotAllowed, MissingCookie,
		MissingHeader, PayloadTooLarge, Rejection, UnsupportedMediaType,
	},
	reply,
	reply::{html, with_header, with_status, Reply, Response},
	serve, ws,
	ws::{Message, WebSocket, Ws},
	Filter,
};

// get generated by 'build.rs'
const SELF_SIGNED_KEY_PEM: &[u8] = include_bytes!("self_signed_certificates/key.pem");
const SELF_SIGNED_CERT_PEM: &[u8] = include_bytes!("self_signed_certificates/cert.pem");

const FAVICON_ICO: &[u8] = include_bytes!("static/favicon.ico");
const ICON_180X180_PNG: &[u8] = include_bytes!("static/icon_180x180.png");
const CARET_DOWN_SVG: &str = include_str!("static/caret-down-fill.svg");
const CARET_RIGHT_SVG: &str = include_str!("static/caret-right-fill.svg");
const BAR_CHART_SVG: &str = include_str!("static/bar-chart-fill.svg");
const HOUSE_CHART_SVG: &str = include_str!("static/house-fill.svg");
const GLOBE_SVG: &str = include_str!("static/globe.svg");
const WINDOW_SVG: &str = include_str!("static/window.svg");
const CPU_SVG: &str = include_str!("static/cpu.svg");
const HOURGLASS_SVG: &str = include_str!("static/hourglass-split.svg");
const MEMORY_SVG: &str = include_str!("static/memory.svg");
const THERMOMETER_HALF_SVG: &str = include_str!("static/thermometer-half.svg");
const FILE_TEXT_SVG: &str = include_str!("static/file-earmark-text.svg");

const INDEX_HTML: &str = include_str!("static/index.html");
const STYLE_CSS: &str = include_str!("static/style.css");
const SCRIPT_JS: &str = include_str!("static/script.js");

const LOGIN_INDEX_HTML: &str = include_str!("static/login/index.html");
const LOGIN_STYLE_CSS: &str = include_str!("static/login/style.css");
const LOGIN_SCRIPT_JS: &str = include_str!("static/login/script.js");

const ADMIN_INDEX_HTML: &str = include_str!("static/admin/index.html");
const ADMIN_STYLE_CSS: &str = include_str!("static/admin/style.css");
const ADMIN_SCRIPT_JS: &str = include_str!("static/admin/script.js");

#[derive(Debug)]
struct InvalidPassword;
impl warp::reject::Reject for InvalidPassword {}

pub async fn run_web_server(
	password: String,
	modified_receiver: Receiver<ModifiedEvent>,
	shared_data: Arc<RwLock<SharedServerData>>,
	log_filepath: PathBuf,
	custom_cert_pem: Option<Vec<u8>>,
	custom_key_pem: Option<Vec<u8>>,
) {
	let password_clone = password.clone();

	let shared_data_clone = shared_data.clone();

	let get_database_route =
		path("load_database")
			.and(post())
			.and(body::json())
			.map(move |body: serde_json::Value| {
				load_database(body, password_clone.clone(), shared_data_clone.clone())
			});

	let password_clone = password.clone();

	let shared_data_clone = shared_data.clone();
	let shared_data_clone = warp::any().map(move || shared_data_clone.clone());

	let admin_infos_route = path("admin_infos")
		.and(post())
		.and(body::json())
		.and(shared_data_clone)
		.and(warp::any().map(move || log_filepath.clone()))
		.map(
			move |body: serde_json::Value,
			      shared_data: Arc<RwLock<SharedServerData>>,
			      log_filepath: PathBuf| {
				get_admin_infos(body, password_clone.clone(), shared_data, log_filepath)
			},
		);

	let modified_receiver = Arc::new(RwLock::new(modified_receiver));
	let modified_receiver = warp::any().map(move || modified_receiver.clone());

	let password_clone = password.clone();
	let password_clone = warp::any().map(move || password_clone.clone());

	let shared_data_clone = shared_data.clone();
	let shared_data_clone = warp::any().map(move || shared_data_clone.clone());

	let modified_ws_route = path("modified")
		.and(ws())
		.and(path::param())
		.and(warp::addr::remote())
		.and(modified_receiver)
		.and(shared_data_clone)
		.and(password_clone)
		.and_then(
			move |ws: Ws,
			      client_password: String,
			      client_addr: Option<SocketAddr>,
			      modified_receiver: Arc<RwLock<Receiver<ModifiedEvent>>>,
			      shared_data: Arc<RwLock<SharedServerData>>,
			      password: String| {
				async move {
					if client_password == password {
						Ok(ws.on_upgrade(move |socket| {
							on_upgrade_modified_ws(
								socket,
								client_addr,
								modified_receiver.read().unwrap().resubscribe(),
								shared_data,
							)
						}))
					} else {
						Err(warp::reject::custom(InvalidPassword))
					}
				}
			},
		)
		.recover(recover_rejection);

	let static_path = path("static");

	let favicon_route = static_path
		.and(path("favicon.ico"))
		.map(|| with_header(FAVICON_ICO, "Content-Type", "image/x-icon"));

	let icon_180x180_png_route = static_path
		.and(path("icon_180x180.png"))
		.map(|| with_header(ICON_180X180_PNG, "Content-Type", "image/png"));

	let caret_down_svg_route = static_path
		.and(path("caret-down-fill.svg"))
		.map(|| with_header(CARET_DOWN_SVG, "Content-Type", "image/svg+xml"));

	let caret_right_svg_route = static_path
		.and(path("caret-right-fill.svg"))
		.map(|| with_header(CARET_RIGHT_SVG, "Content-Type", "image/svg+xml"));

	let bar_chart_svg_route = static_path
		.and(path("bar-chart-fill.svg"))
		.map(|| with_header(BAR_CHART_SVG, "Content-Type", "image/svg+xml"));

	let house_svg_route = static_path
		.and(path("house-fill.svg"))
		.map(|| with_header(HOUSE_CHART_SVG, "Content-Type", "image/svg+xml"));

	let globe_svg_route = static_path
		.and(path("globe.svg"))
		.map(|| with_header(GLOBE_SVG, "Content-Type", "image/svg+xml"));

	let window_svg_route = static_path
		.and(path("window.svg"))
		.map(|| with_header(WINDOW_SVG, "Content-Type", "image/svg+xml"));

	let cpu_svg_route = static_path
		.and(path("cpu.svg"))
		.map(|| with_header(CPU_SVG, "Content-Type", "image/svg+xml"));

	let hourglass_svg_route = static_path
		.and(path("hourglass-split.svg"))
		.map(|| with_header(HOURGLASS_SVG, "Content-Type", "image/svg+xml"));

	let memory_svg_route = static_path
		.and(path("memory.svg"))
		.map(|| with_header(MEMORY_SVG, "Content-Type", "image/svg+xml"));

	let thermometer_svg_route = static_path
		.and(path("thermometer-half.svg"))
		.map(|| with_header(THERMOMETER_HALF_SVG, "Content-Type", "image/svg+xml"));

	let file_text_svg_route = static_path
		.and(path("file-earmark-text.svg"))
		.map(|| with_header(FILE_TEXT_SVG, "Content-Type", "image/svg+xml"));

	let asset_routes = favicon_route
		.or(icon_180x180_png_route)
		.or(caret_down_svg_route)
		.or(caret_right_svg_route)
		.or(bar_chart_svg_route)
		.or(house_svg_route)
		.or(globe_svg_route)
		.or(window_svg_route)
		.or(cpu_svg_route)
		.or(hourglass_svg_route)
		.or(memory_svg_route)
		.or(thermometer_svg_route)
		.or(file_text_svg_route);

	let index_route = end().map(|| html(INDEX_HTML));

	let style_route = static_path
		.and(path("style.css"))
		.map(|| with_header(STYLE_CSS, "Content-Type", "text/css"));

	let script_route = static_path
		.and(path("script.js"))
		.map(|| with_header(SCRIPT_JS, "Content-Type", "application/javascript"));

	let main_routes = index_route.or(style_route).or(script_route);

	let login_path = static_path.and(path("login"));

	let login_html_route = path("login").and(end()).map(|| html(LOGIN_INDEX_HTML));

	let login_index_route = login_path.and(end()).map(|| html(LOGIN_INDEX_HTML));

	let login_style_route = login_path
		.and(path("style.css"))
		.map(|| with_header(LOGIN_STYLE_CSS, "Content-Type", "text/css"));

	let login_script_route = login_path
		.and(path("script.js"))
		.map(|| with_header(LOGIN_SCRIPT_JS, "Content-Type", "application/javascript"));

	let login_routes = login_html_route
		.or(login_index_route)
		.or(login_script_route)
		.or(login_style_route);

	let admin_path = static_path.and(path("admin"));

	let admin_html_route = path("admin").and(end()).map(|| html(ADMIN_INDEX_HTML));

	let admin_index_route = admin_path.and(end()).map(|| html(ADMIN_INDEX_HTML));

	let admin_style_route = admin_path
		.and(path("style.css"))
		.map(|| with_header(ADMIN_STYLE_CSS, "Content-Type", "text/css"));

	let admin_script_route = admin_path
		.and(path("script.js"))
		.map(|| with_header(ADMIN_SCRIPT_JS, "Content-Type", "application/javascript"));

	let admin_routes = admin_infos_route
		.or(admin_html_route)
		.or(admin_index_route)
		.or(admin_script_route)
		.or(admin_style_route);

	let routes = main_routes
		.or(login_routes)
		.or(admin_routes)
		.or(asset_routes)
		.or(get_database_route)
		.or(modified_ws_route);

	let (https_addr, https_warp) = serve(routes)
		.tls()
		.cert(custom_cert_pem.unwrap_or(SELF_SIGNED_CERT_PEM.to_vec()))
		.key(custom_key_pem.unwrap_or(SELF_SIGNED_KEY_PEM.to_vec()))
		.bind_ephemeral(([0, 0, 0, 0], 443));

	info!("https server listening on {https_addr}");

	tokio::spawn(messure_cpu_usage_avg_thread(shared_data.clone()));

	https_warp.await;
}

fn load_database(
	body: serde_json::Value,
	password: String,
	shared_data: Arc<RwLock<SharedServerData>>,
) -> Response {
	if body.get("password") == Some(&serde_json::Value::String(password)) {
		info!("sending database as json");
		reply::json(&shared_data.read().unwrap().database.clone().to_serialized()).into_response()
	} else {
		info!("invalid password providied, refusing access!");
		with_status(html("Unauthorized".to_string()), StatusCode::UNAUTHORIZED).into_response()
	}
}

#[derive(Debug, Serialize)]
struct AdminInfos {
	connected_native_gui_clients: Vec<SocketAddr>,
	connected_web_clients: Vec<SocketAddr>,
	cpu_usage: f32,
	cpu_temp: Option<f32>,
	ram_info: String,
	uptime: String,
	latest_logs_of_the_day: String,
}

fn get_admin_infos(
	body: serde_json::Value,
	password: String,
	shared_data: Arc<RwLock<SharedServerData>>,
	log_filepath: PathBuf,
) -> Response {
	if body.get("password") == Some(&serde_json::Value::String(password)) {
		info!("sending admin infos");

		let (cpu_usage, connected_clients) = {
			let shared_data = shared_data.read().unwrap();
			(
				shared_data.cpu_usage_avg,
				shared_data.connected_clients.clone(),
			)
		};

		let mut connected_native_gui_clients = Vec::new();
		let mut connected_web_clients = Vec::new();

		for connected_client in connected_clients {
			match connected_client {
				ConnectedClient::NativeGUI(addr) => connected_native_gui_clients.push(addr),
				ConnectedClient::Web(addr) => connected_web_clients.push(addr),
			}
		}

		let sys = System::new();

		let cpu_temp = sys.cpu_temp().ok();

		let ram_info = match sys.memory() {
			Ok(mem) => format!(
				"{} / {}",
				saturating_sub_bytes(mem.total, mem.free),
				mem.total
			),
			Err(_) => "failed to get ram info".to_string(),
		};

		let uptime = match sys.uptime() {
			Ok(uptime) => format_duration(uptime).to_string(),
			Err(_) => "failed to get uptime".to_string(),
		};

		let latest_logs_of_the_day = match get_logs_as_string(log_filepath) {
			Ok(logs) => logs,
			Err(error_str) => error_str,
		};

		reply::json(&AdminInfos {
			connected_native_gui_clients,
			connected_web_clients,
			cpu_usage,
			cpu_temp,
			ram_info,
			uptime,
			latest_logs_of_the_day,
		})
		.into_response()
	} else {
		info!("invalid password, refusing admin infos!");
		with_status(html("Unauthorized".to_string()), StatusCode::UNAUTHORIZED).into_response()
	}
}

fn get_latest_journal_logs() -> Result<String, String> {
	let output = Command::new("journalctl")
		.arg("-u")
		.arg("project_tracker_server.service")
		.arg("--since")
		.arg("1 day ago")
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.output()
		.map_err(|e| format!("Failed to execute journalctl: {}", e))?;

	let logs = from_utf8(&output.stdout)
		.map_err(|e| format!("Failed to convert output to string: {}", e))?;

	Ok(logs.to_string())
}

fn read_log_file(log_filepath: PathBuf) -> Result<String, String> {
	std::fs::read_to_string(&log_filepath).map_err(|e| {
		format!(
			"Failed to read log file in '{}', error: {e}",
			log_filepath.display()
		)
	})
}

// returns journal logs if possible, otherwise reads the project_tracker_server.log file
fn get_logs_as_string(log_filepath: PathBuf) -> Result<String, String> {
	get_latest_journal_logs().or(read_log_file(log_filepath))
}

async fn messure_cpu_usage_avg_thread(shared_data: Arc<RwLock<SharedServerData>>) {
	let sys = System::new();
	loop {
		if let Ok(cpu_load) = sys.cpu_load_aggregate() {
			tokio::time::sleep(std::time::Duration::from_secs(2)).await;
			if let Ok(cpu_load) = cpu_load.done() {
				shared_data.write().unwrap().cpu_usage_avg = 1.0 - cpu_load.idle;
			}
		}
	}
}

async fn on_upgrade_modified_ws(
	ws: WebSocket,
	client_addr: Option<SocketAddr>,
	modified_receiver: Receiver<ModifiedEvent>,
	shared_data: Arc<RwLock<SharedServerData>>,
) {
	let connected_client = client_addr.map(ConnectedClient::Web);

	if let Some(connected_client) = connected_client {
		shared_data
			.write()
			.unwrap()
			.connected_clients
			.insert(connected_client);
	}

	handle_modified_ws(ws, modified_receiver).await;

	if let Some(connected_client) = connected_client {
		shared_data
			.write()
			.unwrap()
			.connected_clients
			.remove(&connected_client);
	}
}

async fn handle_modified_ws(ws: WebSocket, mut modified_receiver: Receiver<ModifiedEvent>) {
	let (mut write_ws, mut read_ws) = ws.split();

	info!("modified ws client connected");

	loop {
		tokio::select! {
			modified_event_result = modified_receiver.recv() => {
				match modified_event_result {
					Ok(modified_event) => {
						match serde_json::to_string(&modified_event.modified_database.to_serialized()) {
							Ok(database_json) => {
								info!("sending database modified event in ws");
								if let Err(e) = write_ws.send(Message::text(database_json)).await {
									error!("failed to send modified event: {e}");
									return;
								}
							},
							Err(e) => error!("failed to serialize database in order to send to ws clients: {e}"),
						}
					},
					Err(e) => {
						error!("failed to receive further database modified events: {e}");
						return;
					},
				}
			},
			message = read_ws.next() => {
				if matches!(message, None | Some(Err(_))) {
					info!("modified ws connection closed");
					let _ = write_ws.close().await;
					return;
				}
			},
		};
	}
}

async fn recover_rejection(rejection: Rejection) -> Result<impl Reply, Infallible> {
	let code;
	let message;

	if rejection.is_not_found() {
		code = StatusCode::NOT_FOUND;
		message = String::new();
	} else if let Some(InvalidPassword) = rejection.find() {
		code = StatusCode::UNAUTHORIZED;
		message = "Invalid Password!".to_string();
	} else if rejection.find::<MethodNotAllowed>().is_some() {
		code = StatusCode::METHOD_NOT_ALLOWED;
		message = String::new();
	} else if let Some(e) = rejection.find::<InvalidHeader>() {
		code = StatusCode::BAD_REQUEST;
		message = e.to_string();
	} else if let Some(e) = rejection.find::<MissingHeader>() {
		code = StatusCode::BAD_REQUEST;
		message = e.to_string();
	} else if let Some(e) = rejection.find::<MissingCookie>() {
		code = StatusCode::BAD_REQUEST;
		message = e.to_string();
	} else if rejection.find::<InvalidQuery>().is_some()
		|| rejection.find::<MissingConnectionUpgrade>().is_some()
		|| rejection.find::<LengthRequired>().is_some()
	{
		code = StatusCode::BAD_REQUEST;
		message = String::new();
	} else if let Some(e) = rejection.find::<BodyDeserializeError>() {
		code = StatusCode::BAD_REQUEST;
		message = e.to_string();
	} else if rejection.find::<PayloadTooLarge>().is_some() {
		code = StatusCode::PAYLOAD_TOO_LARGE;
		message = String::new();
	} else if rejection.find::<UnsupportedMediaType>().is_some() {
		code = StatusCode::UNSUPPORTED_MEDIA_TYPE;
		message = String::new();
	} else if let Some(e) = rejection.find::<CorsForbidden>() {
		code = StatusCode::FORBIDDEN;
		message = e.to_string();
	} else if rejection.find::<MissingExtension>().is_some() {
		code = StatusCode::INTERNAL_SERVER_ERROR;
		message = String::new();
	} else {
		error!("unhandled web rejection: {rejection:?}");
		code = StatusCode::INTERNAL_SERVER_ERROR;
		message = "Unhandled Rejection".to_string();
	}

	let json = warp::reply::html(format!(
		"<!DOCTYPE html>
<head>
	<meta charset=\"UTF-8\" />
	<meta name=\"viewport\" content=\"width=device-width\" />
	<link rel=\"icon\" type=\"image/x-icon\" href=\"/static/favicon.ico\">
	<link rel=\"apple-touch-icon\" sizes=\"180x180\" href=\"/static/icon_180x180.png\">
	<title>{code}</title>
	<style>
		body {{
			background-color: #1f1f1f;
			color: #e0e0e0;
			font-family: Arial, sans-serif;
			font-size: 30px;
			font-weight: bold;
			line-height: 1.6;
			height: 100vh;
			margin: 0 auto;
			padding: 15px;
		}}
	</style>
</head>
<body>
	<div>{code}\n{message}</div>
</body>
</html>
"
	));

	Ok(warp::reply::with_status(json, code))
}
