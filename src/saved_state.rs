use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::project::{Project, ProjectId};
use crate::theme_mode::ThemeMode;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SavedState {
	pub projects: HashMap<ProjectId, Project>,
	pub theme_mode: ThemeMode,
}

impl SavedState {
	async fn filepath() -> PathBuf {
		let project_dirs = directories::ProjectDirs::from("", "", "ProjectTracker")
		.expect("Failed to get saved state filepath");

		let data_dir = project_dirs.data_local_dir();

		tokio::fs::create_dir_all(data_dir).await.expect("Failed to create Local Data Directories");

		data_dir
			.join("project_tracker_db.json")
			.to_path_buf()
	}

	pub async fn load() -> SavedState {
		let filepath = Self::filepath().await;

		match tokio::fs::read_to_string(filepath.clone()).await {
			Ok(file_content) => {
				serde_json::from_str(&file_content).unwrap_or_else(|_| {
					println!("Failed to load previous projects in {}", filepath.display());
					SavedState::default()
				})
			},
			Err(_) => {
				println!("Could not find previous projects in {}", filepath.display());
				SavedState::default()
			}
		}
	}

	pub async fn save(self) {
		let filepath = Self::filepath().await;

		if let Err(e) = tokio::fs::write(filepath.clone(), serde_json::to_string_pretty(&self).unwrap().as_bytes()).await {
			eprintln!("Failed to save to {}: {e}", filepath.display());
		}
	}
}