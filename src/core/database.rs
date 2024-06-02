use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::core::{OrderedHashMap, ProjectId, Project};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
	pub projects: OrderedHashMap<ProjectId, Project>,
}

#[derive(Debug, Clone)]
pub enum LoadDatabaseResult {
	Ok(Database),
	FailedToReadFile(PathBuf),
	FailedToParse(PathBuf),
}

impl Database {
	pub fn new() -> Self {
		Self {
			projects: OrderedHashMap::new(),
		}
	}

	async fn filepath() -> PathBuf {
		let project_dirs = directories::ProjectDirs::from("", "", "ProjectTracker")
		.expect("Failed to get saved state filepath");

		let data_dir = project_dirs.data_local_dir();

		tokio::fs::create_dir_all(data_dir).await.expect("Failed to create Local Data Directories");

		data_dir
			.join("database.json")
			.to_path_buf()
	}

	async fn load_from(filepath: PathBuf) -> LoadDatabaseResult {
		let file_content = if let Ok(file_content) = tokio::fs::read_to_string(filepath.clone()).await {
			file_content
		}
		else {
			return LoadDatabaseResult::FailedToReadFile(filepath);
		};

		match serde_json::from_str(&file_content) {
			Ok(database) => LoadDatabaseResult::Ok(database),
			Err(_) => LoadDatabaseResult::FailedToParse(filepath),
		}
	}

	pub async fn load() -> LoadDatabaseResult {
		Self::load_from(Self::filepath().await).await
	}

	async fn save_to(self, filepath: PathBuf) {
		if let Err(e) = tokio::fs::write(filepath.clone(), serde_json::to_string_pretty(&self).unwrap().as_bytes()).await {
			eprintln!("Failed to save to {}: {e}", filepath.display());
		}
	}

	pub async fn save(self) {
		self.save_to(Self::filepath().await).await;		
	}

	pub async fn export_file_dialog(self) {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.add_filter("Database", &["json"])
			.save_file()
			.await;

		if let Some(result) = file_dialog_result {
			self.save_to(result.path().to_path_buf()).await;
		}
	}
	
	pub async fn import_file_dialog() -> Option<LoadDatabaseResult> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.add_filter("Database", &["json"])
			.pick_file()
			.await;

		if let Some(result) = file_dialog_result {
			Some(Self::load_from(result.path().to_path_buf()).await)
		}
		else {
			None
		}
	}
}