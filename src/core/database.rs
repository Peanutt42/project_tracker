use std::path::PathBuf;
use iced::Command;
use serde::{Serialize, Deserialize};
use crate::project_tracker::UiMessage;
use crate::core::{OrderedHashMap, ProjectId, Project};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
	pub projects: OrderedHashMap<ProjectId, Project>,
}

#[derive(Debug, Clone)]
pub enum DatabaseMessage {
	Save,
	Saved,
	Clear,
	Export,
	Exported,
	Import,
	ImportFailed,
}

impl From<DatabaseMessage> for UiMessage {
	fn from(value: DatabaseMessage) -> Self {
		UiMessage::DatabaseMessage(value)
	}
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

	pub fn update(&mut self, message: DatabaseMessage) -> Command<UiMessage> {
		match message {
			DatabaseMessage::Save => Command::perform(self.clone().save(), |_| DatabaseMessage::Saved.into()),
			DatabaseMessage::Saved => Command::none(),
			DatabaseMessage::Clear => { *self = Self::new(); self.update(DatabaseMessage::Save) },
			DatabaseMessage::Export => Command::perform(self.clone().export_file_dialog(), |_| DatabaseMessage::Exported.into()),
			DatabaseMessage::Exported => Command::none(),
			DatabaseMessage::Import => Command::perform(
								Self::import_file_dialog(),
								|result| {
									if let Some(load_database_result) = result {
										UiMessage::LoadedDatabase(load_database_result)
									}
									else {
										DatabaseMessage::ImportFailed.into()
									}
								}),
			DatabaseMessage::ImportFailed => Command::none(),
		}
	}

	pub fn get_filepath() -> PathBuf {
		let project_dirs = directories::ProjectDirs::from("", "", "ProjectTracker")
				.expect("Failed to get saved state filepath");

		project_dirs.data_local_dir().join("database.json")
			.to_path_buf()
	}

	async fn get_and_ensure_filepath() -> PathBuf {
		let filepath = Self::get_filepath();

		tokio::fs::create_dir_all(filepath.parent().unwrap()).await.expect("Failed to create Local Data Directories");

		filepath
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
		Self::load_from(Self::get_and_ensure_filepath().await).await
	}

	async fn save_to(self, filepath: PathBuf) {
		if let Err(e) = tokio::fs::write(filepath.clone(), serde_json::to_string_pretty(&self).unwrap().as_bytes()).await {
			eprintln!("Failed to save to {}: {e}", filepath.display());
		}
	}

	async fn save(self) {
		self.save_to(Self::get_and_ensure_filepath().await).await;
	}

	async fn export_file_dialog(self) {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.add_filter("Database", &["json"])
			.save_file()
			.await;

		if let Some(result) = file_dialog_result {
			self.save_to(result.path().to_path_buf()).await;
		}
	}

	async fn import_file_dialog() -> Option<LoadDatabaseResult> {
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
