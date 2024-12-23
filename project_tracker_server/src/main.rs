use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::exit;
use project_tracker_core::Database;
use project_tracker_server::{SharedServerData, DEFAULT_PASSWORD, DEFAULT_PORT};

#[cfg(feature = "web_server")]
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

	let password = if password_filepath.exists() {
		read_to_string(&password_filepath)
			.unwrap_or_else(|e| {
				eprintln!("failed to read password file!\nset password using the 'scripts/set_server_password_linux.sh' script\n{}, error: {e}", password_filepath.display());
				exit(1);
			})
	}
	else {
		eprintln!(
			"no password is set, using default password: 1234\nset it using the 'scripts/set_server_password_linux.sh' script\nor create a plaintext password.txt with the password inside the 'SERVER_DATA_DIRECTORY'!"
		);
		DEFAULT_PASSWORD.to_string()
	};

	let shared_data = if database_filepath.exists() {
		SharedServerData::new(database_filepath.clone())
	}
	else {
		eprintln!("no previous database found -> creating a empty database!");
		SharedServerData::from_memory(Database::default())
	};

	#[allow(unused)]
	let (modified_sender, modified_receiver) = tokio::sync::broadcast::channel(10);



	#[cfg(feature = "web_server")]
	{
		let password_clone = password.clone();
		let shared_data_clone = shared_data.clone();
		let opt_custom_cert_pem_filepath = server_data_directory.join("cert.pem");
		let opt_custom_key_pem_filepath = server_data_directory.join("key.pem");
		let custom_cert_pem = tokio::fs::read(opt_custom_cert_pem_filepath).await.ok();
		let custom_key_pem = tokio::fs::read(opt_custom_key_pem_filepath).await.ok();
		std::thread::Builder::new()
			.name("Web Server".to_string())
			.spawn(move || {
				let rt = tokio::runtime::Runtime::new().unwrap();

				rt.block_on(async {
					web_server::run_web_server(
						password_clone,
						modified_receiver,
						shared_data_clone,
						custom_cert_pem,
						custom_key_pem,
					)
					.await;
				});
			})
			.expect("failed to start web server thread");
	}

	project_tracker_server::run_server(
		DEFAULT_PORT,
		database_filepath,
		password,
		modified_sender,
		shared_data
	)
	.await;
}