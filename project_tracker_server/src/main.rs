use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::exit;
use project_tracker_server::{SharedServerData, DEFAULT_PORT};

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
		eprintln!("the supplied server data directory doesn't exist!");
		exit(1);
	}

	let database_filepath = server_data_directory.join("database.project_tracker");
	let password_filepath = server_data_directory.join("password.txt");

	if !database_filepath.exists() {
		eprintln!("no database file found!\nto create a empty database, run the 'scripts/install_server_linux.sh' script again!");
		exit(1);
	}

	if !password_filepath.exists() {
		eprintln!("no password is set, set it using the 'scripts/set_server_password_linux.sh' script!");
		exit(1);
	}

	let password = read_to_string(&password_filepath)
		.unwrap_or_else(|e| {
			eprintln!("failed to read password file!\nset password using the 'scripts/set_server_password_linux.sh' script\n{}, error: {e}", password_filepath.display());
			exit(1);
		});

	let shared_data = SharedServerData::new(database_filepath.clone());

	let (modified_sender, modified_receiver) = tokio::sync::broadcast::channel(10);

	let password_clone = password.clone();
	let shared_data_clone = shared_data.clone();
	std::thread::Builder::new()
		.name("Web Server".to_string())
		.spawn(move || {
			let rt = tokio::runtime::Runtime::new().unwrap();

			rt.block_on(async {
				web_server::run_web_server(password_clone, modified_receiver, shared_data_clone).await;
			});
		})
		.expect("failed to start web server thread");

	project_tracker_server::run_server(
		DEFAULT_PORT,
		database_filepath,
		password,
		modified_sender,
		shared_data
	)
	.await;
}