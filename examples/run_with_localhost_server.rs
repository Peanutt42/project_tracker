use project_tracker::{
	integrations::ServerConfig, run_project_tracker_app, AppFlags, Database, Preferences,
	SynchronizationSetting,
};
use project_tracker_server::{SharedServerData, DEFAULT_PASSWORD, DEFAULT_PORT};

#[tokio::main]
async fn main() -> Result<(), iced::Error> {
	tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new()).unwrap();

	let temp_dir = std::env::temp_dir();
	let custom_database_filepath = temp_dir.join(Database::FILE_NAME);
	let mut server_database_filepath = temp_dir.join(Database::FILE_NAME);
	server_database_filepath.set_file_name(format!("server_{}", Database::FILE_NAME));
	let server_log_filepath = temp_dir.join("project_tracker_server.log");
	let custom_preferences_filepath = temp_dir.join(Preferences::FILE_NAME);

	// clean up previous temp files from us
	let _ = std::fs::remove_file(&custom_database_filepath);
	let _ = std::fs::remove_file(&server_database_filepath);
	let _ = std::fs::remove_file(&server_log_filepath);
	let _ = std::fs::remove_file(&custom_preferences_filepath);

	// save custom preferences that configure localhost server sync
	let mut tmp_client_preferences = Preferences::default();
	tmp_client_preferences.set_synchronization(Some(SynchronizationSetting::Server(
		ServerConfig::default(),
	)));
	Preferences::save(
		custom_preferences_filepath.clone(),
		tmp_client_preferences
			.serialized()
			.expect("failed to serialize custom preferences to connect to localhost server"),
	)
	.await
	.expect("failed to save custom preferences to connect to localhost server");

	// start server
	let shared_data = SharedServerData::from_memory(Database::default());
	let (modified_sender, _modified_receiver) = tokio::sync::broadcast::channel(10);
	tokio::spawn(project_tracker_server::run_server(
		DEFAULT_PORT,
		server_database_filepath,
		server_log_filepath,
		DEFAULT_PASSWORD.to_string(),
		modified_sender,
		shared_data,
	));

	run_project_tracker_app(AppFlags::custom(
		custom_database_filepath,
		custom_preferences_filepath,
	))
}
