use std::fs::File;
use std::path::PathBuf;
use std::process::exit;

#[cfg(feature = "dashboard")]
mod dashboard;

mod server;

#[tokio::main]
async fn main() {
	let mut args = std::env::args();

	let filepath_str = args.nth(1).unwrap_or_else(|| {
		eprintln!("usage: project_tracker_server [DATABASE_FILEPATH]");
		exit(1);
	});

	let filepath = PathBuf::from(filepath_str);

	if !filepath.exists() {
		if let Err(e) = File::create(&filepath) {
			eprintln!("failed to create/open database file: {}, error: {e}", filepath.display());
			exit(1);
		}
	}

	#[cfg(feature = "dashboard")]
	dashboard::run_dashboard(filepath);

	#[cfg(not(feature = "dashboard"))]
	{
		let listener = server::create_server().await;

		loop {
			match listener.accept().await {
				Ok((stream, _addr)) => {
					let filepath_clone = filepath.clone();
					tokio::spawn(async move {
						let _ = server::handle_client(stream, filepath_clone).await;
					});
				}
				Err(e) => {
					eprintln!("Failed to establish a connection: {e}");
				}
			}
		}
	}
}
