use std::fs::{File, read_to_string};
use std::path::PathBuf;
use std::io::Write;
use std::process::exit;
use project_tracker_server::{run_server, DEFAULT_PASSWORD, DEFAULT_PORT};

fn main() {
	let mut args = std::env::args();

	let server_data_directory_str = args.nth(1).unwrap_or_else(|| {
		eprintln!("usage: project_tracker_server [SERVER_DATA_DIRECTORY]");
		exit(1);
	});

	let server_data_directory = PathBuf::from(server_data_directory_str);

	if !server_data_directory.exists() {
		eprintln!("the supplied directory doesn't exist!");
		exit(1);
	}

	let database_filepath = server_data_directory.join("database.json");
	let password_filepath = server_data_directory.join("password.txt");

	if !database_filepath.exists() {
		if let Err(e) = File::create(&database_filepath) {
			eprintln!("failed to create database file: {}, error: {e}", database_filepath.display());
			exit(1);
		}
	}

	if !password_filepath.exists() {
		match File::create(&password_filepath) {
			Ok(mut file) => {
				if let Err(e) = file.write_all(DEFAULT_PASSWORD.as_bytes()) {
					eprintln!("failed to write default password to password file: {}, error: {e}", password_filepath.display());
					exit(1);
				}
				eprintln!("IMPORTANT: Setting default password to {DEFAULT_PASSWORD}! PLEASE change it!");
			},
			Err(e) => {
				eprintln!("failed to create default password file: {}, error: {e}", password_filepath.display());
				exit(1);
			}
		}
	}

	let password = read_to_string(&password_filepath)
		.unwrap_or_else(|e| {
			eprintln!("failed to read password file: {}, error: {e}", password_filepath.display());
			exit(1);
		});

	run_server(DEFAULT_PORT, database_filepath, password);
}