use project_tracker::{run_project_tracker_app, AppFlags, Database, Preferences};

fn main() -> iced::Result {
	tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new()).unwrap();

	let temp_dir = std::env::temp_dir();
	let custom_database_filepath = temp_dir.join(Database::FILE_NAME);
	let custom_preferences_filepath = temp_dir.join(Preferences::FILE_NAME);

	// clean up previous temp files from us
	let _ = std::fs::remove_file(&custom_database_filepath);
	let _ = std::fs::remove_file(&custom_preferences_filepath);

	run_project_tracker_app(AppFlags::custom(
		custom_database_filepath,
		custom_preferences_filepath,
	))
}
