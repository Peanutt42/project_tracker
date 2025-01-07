use std::path::PathBuf;

use project_tracker::integrations::import_google_tasks;

#[tokio::main]
async fn main() {
	let mut args = std::env::args();

	let Some(filepath) = args.nth(1) else {
		eprintln!("usage: print_google_tasks_info /path/to/google/tasks/json");
		return;
	};

	println!("{:#?}", import_google_tasks(PathBuf::from(filepath)).await);
}
