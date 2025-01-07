use project_tracker_core::Database;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
	let mut args = std::env::args();

	let Some(filepath) = args.nth(1) else {
		eprintln!("usage: create_empty_database [FILEPATH]");
		return;
	};

	Database::save(
		PathBuf::from(filepath),
		Database::default().to_binary().unwrap(),
	)
	.await
	.unwrap();
}
