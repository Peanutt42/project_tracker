use std::path::PathBuf;

use project_tracker::integrations::import_google_tasks;

#[tokio::main]
async fn main() {
	let mut args = std::env::args();

	if let Some(filepath) = args.nth(1) {
		println!("{:#?}", import_google_tasks(PathBuf::from(filepath)).await);
	} else {
		println!("usage: print_google_tasks_info /path/to/google/tasks/json");
	}
}
