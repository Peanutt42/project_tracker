use project_tracker_core::Database;
use project_tracker_server::{
	load_database_from_file, messure_cpu_usage_avg_thread, ConnectedClient, DEFAULT_PASSWORD,
	DEFAULT_PORT,
};
use std::collections::HashSet;
use std::fs::{read_to_string, OpenOptions};
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, RwLock};
use tracing::level_filters::LevelFilter;
use tracing::warn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;

mod web_server;

#[tokio::main]
async fn main() {
	let mut args = std::env::args();

	let server_data_directory_str = args.nth(1).unwrap_or_else(|| {
		eprintln!("usage: project_tracker_server [SERVER_DATA_DIRECTORY]");
		exit(1);
	});

	let server_data_directory = PathBuf::from(server_data_directory_str);

	if !server_data_directory.exists() {
		std::fs::create_dir_all(&server_data_directory)
			.expect("failed to create the supplied 'server_data_directory'");
	}

	let database_filepath = server_data_directory.join("database.project_tracker");
	let password_filepath = server_data_directory.join("password.txt");

	let password = if password_filepath.exists() {
		read_to_string(&password_filepath)
			.unwrap_or_else(|e| {
				eprintln!("failed to read password file!\nset password using the 'scripts/set_server_password_linux.sh' script\n{}, error: {e}", password_filepath.display());
				exit(1);
			})
	} else {
		eprintln!(
			"no password is set, using default password: 1234\nset it using the 'scripts/set_server_password_linux.sh' script\nor create a plaintext password.txt with the password inside the 'SERVER_DATA_DIRECTORY'!"
		);
		DEFAULT_PASSWORD.to_string()
	};

	let database = if database_filepath.exists() {
		load_database_from_file(database_filepath.clone())
	} else {
		eprintln!("no previous database found -> creating a empty database!");
		Database::default()
	};
	let shared_database = Arc::new(RwLock::new(database));

	let stdout_layer = tracing_subscriber::fmt::layer()
		.with_writer(std::io::stdout)
		.with_filter(LevelFilter::INFO);
	let log_filepath = server_data_directory.join("project_tracker_server.log");
	let log_file = OpenOptions::new()
		.append(true)
		.create(true)
		.open(&log_filepath)
		.expect("failed to open log file");
	let file_layer = tracing_subscriber::fmt::layer()
		.with_writer(log_file)
		.with_ansi(false)
		.with_filter(LevelFilter::INFO);
	tracing::subscriber::set_global_default(
		tracing_subscriber::registry()
			.with(stdout_layer)
			.with(file_layer),
	)
	.unwrap();

	let cpu_usage_avg = Arc::new(RwLock::new(0.0));

	tokio::spawn(messure_cpu_usage_avg_thread(cpu_usage_avg.clone()));

	let connected_clients = Arc::new(RwLock::new(HashSet::<ConnectedClient>::new()));

	let (modified_sender, modified_receiver) = tokio::sync::broadcast::channel(10);

	let password_clone = password.clone();
	let log_filepath_clone = log_filepath.clone();
	let shared_database_clone = shared_database.clone();
	let connected_clients_clone = connected_clients.clone();
	let cpu_usage_avg_clone = cpu_usage_avg.clone();
	let opt_custom_cert_pem_filepath = server_data_directory.join("cert.pem");
	let opt_custom_key_pem_filepath = server_data_directory.join("key.pem");
	let custom_cert_pem = tokio::fs::read(opt_custom_cert_pem_filepath).await.ok();
	let custom_key_pem = tokio::fs::read(opt_custom_key_pem_filepath).await.ok();
	let custom_cert_and_key_pem = match (custom_cert_pem, custom_key_pem) {
		(Some(_), None) => {
			warn!("only the custom cert.pem file is present, no key.pem found --> ignoring custom certificate!");
			None
		}
		(None, Some(_)) => {
			warn!("only the custom key.pem file is present, no cert.pem found --> ignoring custom certificate!");
			None
		}
		(None, None) => None,
		(Some(cert_pem), Some(key_pem)) => Some((cert_pem, key_pem)),
	};

	std::thread::Builder::new()
		.name("Web Server".to_string())
		.spawn(move || {
			let rt = tokio::runtime::Runtime::new().unwrap();

			rt.block_on(async {
				web_server::run_web_server(
					password_clone,
					modified_receiver,
					shared_database_clone,
					connected_clients_clone,
					cpu_usage_avg_clone,
					log_filepath_clone,
					custom_cert_and_key_pem,
				)
				.await;
			});
		})
		.expect("failed to start web server thread");

	project_tracker_server::run_server(
		DEFAULT_PORT,
		database_filepath,
		log_filepath,
		password,
		modified_sender,
		shared_database,
		connected_clients,
		cpu_usage_avg,
	)
	.await;
}
