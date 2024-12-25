use std::path::PathBuf;

use project_tracker_core::{Database, DatabaseMessage};
use crate::project_tracker::Message;

impl From<DatabaseMessage> for Message {
	fn from(value: DatabaseMessage) -> Self {
		Message::DatabaseMessage(value)
	}
}

pub async fn export_database_file_dialog() -> Option<PathBuf> {
	let file_dialog_result = rfd::AsyncFileDialog::new()
		.set_title("Export ProjectTracker Database")
		.set_file_name(Database::FILE_NAME)
		.add_filter("Database (.project_tracker)", &["project_tracker"])
		.save_file()
		.await;

	file_dialog_result.map(|file_handle| file_handle.path().to_path_buf())
}

pub async fn import_database_file_dialog() -> Option<PathBuf> {
	let file_dialog_result = rfd::AsyncFileDialog::new()
		.set_title("Import ProjectTracker Database")
		.add_filter("Database (.project_tracker)", &["project_tracker"])
		.pick_file()
		.await;

	file_dialog_result.map(|file_handle| file_handle.path().to_path_buf())
}

pub async fn export_database_as_json_file_dialog() -> Option<PathBuf> {
	let file_dialog_result = rfd::AsyncFileDialog::new()
		.set_title("Export ProjectTracker Database as Json")
		.set_file_name(Database::JSON_FILE_NAME)
		.add_filter("Database (.json)", &["json"])
		.save_file()
		.await;

	file_dialog_result.map(|file_handle| file_handle.path().to_path_buf())
}

pub async fn import_json_database_file_dialog() -> Option<PathBuf> {
	let file_dialog_result = rfd::AsyncFileDialog::new()
		.set_title("Import Json ProjectTracker Database")
		.add_filter("Database (.json)", &["json"])
		.pick_file()
		.await;

	file_dialog_result.map(|file_handle| file_handle.path().to_path_buf())
}