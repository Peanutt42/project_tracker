use project_tracker::{
	run_project_tracker_app, synchronization::ServerSynchronization, AppFlags, Database,
	Preferences,
};
use std::process::Command;
use tracing::info;

fn main() -> Result<(), iced::Error> {
	tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new()).unwrap();

	let temp_dir = std::env::temp_dir();
	let custom_database_filepath = temp_dir.join(Database::FILE_NAME);
	let custom_preferences_filepath = temp_dir.join(Preferences::FILE_NAME);

	// clean up previous temp files from us
	let _ = std::fs::remove_file(&custom_database_filepath);
	let _ = std::fs::remove_file(&custom_preferences_filepath);

	// save custom preferences that configure localhost server sync
	{
		let rt = tokio::runtime::Runtime::new().unwrap();
		let mut tmp_client_preferences = Preferences::default();
		tmp_client_preferences.set_synchronization(Some(ServerSynchronization::default().into()));
		rt.block_on(Preferences::save(
			custom_preferences_filepath.clone(),
			tmp_client_preferences
				.serialized()
				.expect("failed to serialize custom preferences to connect to localhost server"),
		))
		.expect("failed to save custom preferences to connect to localhost server");
	}

	// compile localhost server example
	info!("compiling localhost server example...");
	let compilation_success = Command::new("cargo")
		.args(["b", "--release", "--example", "run_localhost_server"])
		.spawn()
		.expect("failed to spawn cargo to compile locahost server")
		.wait()
		.expect("failed to wait for cargo to compile localhost server")
		.success();
	assert!(compilation_success);

	// start server process
	let mut server_process = Command::new("cargo")
		.args(["r", "--release", "--example", "run_localhost_server"])
		.spawn()
		.expect("failed to spawn server process");

	let result = run_project_tracker_app(AppFlags::custom(
		custom_database_filepath,
		custom_preferences_filepath,
	));

	info!("closing for server process");
	let _ = server_process.kill();
	let _ = server_process.wait().unwrap();

	result
}
