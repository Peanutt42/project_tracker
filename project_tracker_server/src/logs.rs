use std::{
	fs::File,
	io::{BufReader, Read, Seek, SeekFrom},
	path::PathBuf,
	process::{Command, Stdio},
	str::from_utf8,
};

const MAX_ADMIN_INFO_LOG_COUNT: usize = 1000;
// on avarage you can expect 100 chars per line
const MAX_ADMIN_INFO_LOG_CHARS: usize = MAX_ADMIN_INFO_LOG_COUNT * 100;

fn get_latest_journal_logs() -> Result<String, String> {
	let output = Command::new("journalctl")
		.arg("-u")
		.arg("project_tracker_server.service")
		.arg("-n")
		.arg(MAX_ADMIN_INFO_LOG_COUNT.to_string())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.output()
		.map_err(|e| format!("Failed to execute journalctl: {}", e))?;

	let logs = from_utf8(&output.stdout)
		.map_err(|e| format!("Failed to convert output to string: {}", e))?;

	Ok(logs.to_string())
}

fn read_log_file(log_filepath: &PathBuf) -> Result<String, String> {
	let file = File::open(log_filepath).map_err(|e| {
		format!(
			"Failed to open log file in '{}', error: {e}",
			log_filepath.display()
		)
	})?;
	let mut reader = BufReader::new(file);

	let file_size = reader.seek(SeekFrom::End(0)).map_err(|e| {
		format!(
			"Failed to get file size of log file in '{}', error: {e}",
			log_filepath.display()
		)
	})?;

	let start_read_position = file_size.saturating_sub(MAX_ADMIN_INFO_LOG_CHARS as u64);
	reader
		.seek(SeekFrom::Start(start_read_position))
		.map_err(|e| {
			format!(
				"Failed to read log file in '{}', error: {e}",
				log_filepath.display()
			)
		})?;
	let mut log_str = String::new();
	reader.read_to_string(&mut log_str).map_err(|e| {
		format!(
			"Failed to read log file in '{}', error: {e}",
			log_filepath.display()
		)
	})?;
	Ok(log_str)
}

// returns journal logs if possible, otherwise reads the project_tracker_server.log file
pub fn get_logs_as_string(log_filepath: &PathBuf) -> Result<String, String> {
	get_latest_journal_logs().or(read_log_file(log_filepath))
}
