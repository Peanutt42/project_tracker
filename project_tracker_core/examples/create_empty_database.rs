use std::path::PathBuf;
use project_tracker_core::Database;

#[tokio::main]
async fn main() {
	let mut args = std::env::args();

	if let Some(filepath) = args.nth(1) {
		Database::save_to(
			PathBuf::from(filepath),
			Database::default().to_binary().unwrap(),
		)
		.await
		.unwrap();
	} else {
		println!("usage: create_empty_database [FILEPATH]");
	}
}