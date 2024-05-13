use serde::{Serialize, Deserialize};
use crate::project::Project;
use crate::theme_mode::ThemeMode;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SavedState {
	pub projects: Vec<Project>,
	pub theme_mode: ThemeMode,
}

impl SavedState {
	pub async fn load() -> SavedState {
		match tokio::fs::read_to_string("save.project_tracker").await {
			Ok(file_content) => {
				match serde_json::from_str(&file_content) {
					Ok(saved_state) => saved_state,
					Err(_) => {
						println!("Failed to load previous projects");
						SavedState::default()
					}
				}
			},
			Err(_) => {
				println!("Could not find previous projects");
				SavedState::default()
			}
		}
	}

	pub async fn save(self) {
		if let Err(e) = tokio::fs::write("save.project_tracker", serde_json::to_string_pretty(&self).unwrap().as_bytes()).await {
			eprintln!("Failed to save: {e}");
		}
	}
}