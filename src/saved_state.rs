use serde::{Serialize, Deserialize};
use crate::project::Project;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
	pub projects: Vec<Project>,
	pub dark_mode: bool,
}

impl SavedState {
	pub async fn load() -> SavedState {
		match tokio::fs::read_to_string("save.project_tracker").await {
			Ok(file_content) => {
				serde_json::from_str(&file_content).unwrap_or_default()
			},
			Err(e) => {
				eprintln!("Failed to load previous projects: {e}");
				SavedState {
					projects: Vec::new(),
					dark_mode: true,
				}
			}
		}
	}

	pub async fn save(self) {
		if let Err(e) = tokio::fs::write("save.project_tracker", serde_json::to_string_pretty(&self).unwrap().as_bytes()).await {
			eprintln!("Failed to save: {e}");
		}
	}
}


impl Default for SavedState {
	fn default() -> Self {
		Self {
			projects: Vec::new(),
			dark_mode: true,
		}
	}
}
