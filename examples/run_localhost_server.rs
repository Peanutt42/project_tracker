use project_tracker::Database;
use project_tracker_server::{
	messure_cpu_usage_avg_thread, CpuUsageAverage, DEFAULT_PASSWORD, DEFAULT_PORT,
};
use std::{collections::HashSet, fs::OpenOptions, sync::Arc};
use tokio::sync::RwLock;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, Layer};

// simplified 'project_tracker_server::run_server' procedure to only handle the one test client and then quit
#[tokio::main]
async fn main() {
	let temp_dir = std::env::temp_dir();

	let mut server_database_filepath = temp_dir.join(Database::FILE_NAME);
	server_database_filepath.set_file_name(format!("server_{}", Database::FILE_NAME));
	let server_log_filepath = temp_dir.join("project_tracker_server.log");

	// clean up previous temp files from us
	let _ = std::fs::remove_file(&server_database_filepath);
	let _ = std::fs::remove_file(&server_log_filepath);

	let stdout_layer = tracing_subscriber::fmt::layer()
		.with_writer(std::io::stdout)
		.with_filter(LevelFilter::INFO);
	let log_file = OpenOptions::new()
		.append(true)
		.create(true)
		.open(&server_log_filepath)
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

	let shared_database = Arc::new(RwLock::new(Database::default()));
	let connected_clients = Arc::new(RwLock::new(HashSet::new()));
	let (modified_sender, _modified_receiver) = tokio::sync::broadcast::channel(10);
	let cpu_usage_avg = Arc::new(CpuUsageAverage::new());
	let cpu_usage_avg_clone = cpu_usage_avg.clone();
	tokio::spawn(messure_cpu_usage_avg_thread(cpu_usage_avg_clone));

	project_tracker_server::run_server(
		DEFAULT_PORT,
		server_database_filepath,
		server_log_filepath,
		DEFAULT_PASSWORD.to_string(),
		modified_sender,
		shared_database,
		connected_clients,
		cpu_usage_avg,
	)
	.await;
}
