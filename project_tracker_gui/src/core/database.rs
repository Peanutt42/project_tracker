use std::path::PathBuf;
use std::time::Instant;
use iced::Command;
use serde::{Serialize, Deserialize};
use crate::components::ErrorMsgModalMessage;
use crate::project_tracker::UiMessage;
use crate::core::{OrderedHashMap, ProjectId, Project, TaskId};

use super::TaskState;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Database {
	pub projects: OrderedHashMap<ProjectId, Project>,

	#[serde(skip, default = "Instant::now")]
	last_changed_time: Instant,

	#[serde(skip, default = "Instant::now")]
	last_saved_time: Instant,
}

#[derive(Clone, Debug)]
pub enum DatabaseMessage {
	Save,
	Saved(Instant), // begin_time since saving
	Clear,
	Export,
	Exported,
	Import,
	ImportCanceled,

	CreateProject {
		project_id: ProjectId,
		name: String,
	},
	ChangeProjectName {
		project_id: ProjectId,
		new_name: String,
	},
	MoveProjectUp(ProjectId),
	MoveProjectDown(ProjectId),
	DeleteProject(ProjectId),
	DeleteDoneTasks(ProjectId),

	CreateTask {
		project_id: ProjectId,
		task_id: TaskId,
		task_name: String,
	},
	ChangeTaskName {
		project_id: ProjectId,
		task_id: TaskId,
		new_task_name: String,
	},
	ChangeTaskState {
		project_id: ProjectId,
		task_id: TaskId,
		new_task_state: TaskState,
	},
	MoveTaskUp {
		project_id: ProjectId,
		task_id: TaskId,
	},
	MoveTaskDown {
		project_id: ProjectId,
		task_id: TaskId,
	},
	DeleteTask {
		project_id: ProjectId,
		task_id: TaskId,
	},
}

impl From<DatabaseMessage> for UiMessage {
	fn from(value: DatabaseMessage) -> Self {
		UiMessage::DatabaseMessage(value)
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadDatabaseResult {
	Ok(Database),
	FailedToOpenFile(PathBuf),
	FailedToParse(PathBuf),
}

impl Database {
	const FILE_NAME: &'static str = "database.json";

	pub fn new() -> Self {
		Self {
			projects: OrderedHashMap::new(),
			last_changed_time: Instant::now(),
			last_saved_time: Instant::now(),
		}
	}

	pub fn has_same_content_as(&self, other: &Database) -> bool {
		self.projects == other.projects
	}

	fn change_was_made(&mut self) {
		self.last_changed_time = Instant::now();
	}

	pub fn has_unsaved_changes(&self) -> bool {
		self.last_changed_time > self.last_saved_time
	}

	pub fn update(&mut self, message: DatabaseMessage) -> Command<UiMessage> {
		match message {
			DatabaseMessage::Save => Command::perform(
				self.clone().save(),
				|result| {
					match result {
						Ok(begin_time) => DatabaseMessage::Saved(begin_time).into(),
						Err(error_msg) => ErrorMsgModalMessage::open(error_msg),
					}
				}
			),
			DatabaseMessage::Saved(begin_time) => { self.last_saved_time = begin_time; Command::none() },
			DatabaseMessage::Clear => { *self = Self::new(); self.change_was_made(); Command::none() },
			DatabaseMessage::Export => Command::perform(self.clone().export_file_dialog(), |_| DatabaseMessage::Exported.into()),
			DatabaseMessage::Exported => Command::none(),
			DatabaseMessage::Import => Command::perform(
				Self::import_file_dialog(),
				|result| {
					if let Some(load_database_result) = result {
						UiMessage::LoadedDatabase(load_database_result)
					}
					else {
						DatabaseMessage::ImportCanceled.into()
					}
				}
			),
			DatabaseMessage::ImportCanceled => Command::none(),

			DatabaseMessage::CreateProject { project_id, name } => {
				self.projects.insert(project_id, Project::new(name));
				self.change_was_made();
				Command::none()
			},
			DatabaseMessage::ChangeProjectName { project_id, new_name } => {
				if let Some(project) = self.projects.get_mut(&project_id) {
					project.name = new_name;
					self.change_was_made();
				}
				Command::none()
			},
			DatabaseMessage::MoveProjectUp(project_id) => {
				self.projects.move_up(&project_id);
				self.change_was_made();
				Command::none()
			},
			DatabaseMessage::MoveProjectDown(project_id) => {
				self.projects.move_down(&project_id);
				self.change_was_made();
				Command::none()
			},
			DatabaseMessage::DeleteProject(project_id) => {
				self.projects.remove(&project_id);
				self.change_was_made();
				Command::none()
			},
			DatabaseMessage::DeleteDoneTasks(project_id) => {
				if let Some(project) = self.projects.get_mut(&project_id) {
					let done_tasks: Vec<TaskId> = project.tasks.iter()
						.filter(|(_task_id, task)| task.is_done())
						.map(|(task_id, _task)| task_id)
						.collect();
					for task_id in done_tasks {
						project.tasks.remove(&task_id);
					}
					self.change_was_made();
				}
				Command::none()
			},

			DatabaseMessage::CreateTask { project_id, task_id, task_name } => {
				if let Some(project) = self.projects.get_mut(&project_id) {
					project.add_task(task_id, task_name);
					self.change_was_made();
				}
				Command::none()
			},
			DatabaseMessage::ChangeTaskName { project_id, task_id, new_task_name } => {
				if let Some(project) = self.projects.get_mut(&project_id) {
					project.set_task_name(task_id, new_task_name);
					self.change_was_made();
				}
				Command::none()
			},
			DatabaseMessage::ChangeTaskState { project_id, task_id, new_task_state } => {
				if let Some(project) = self.projects.get_mut(&project_id) {
					project.set_task_state(task_id, new_task_state);
					self.change_was_made();
				}
				Command::none()
			},
			DatabaseMessage::MoveTaskUp { project_id, task_id } => {
				if let Some(project) = self.projects.get_mut(&project_id) {
					project.tasks.move_up(&task_id);
					self.change_was_made();
				}
				Command::none()
			},
			DatabaseMessage::MoveTaskDown { project_id, task_id } => {
				if let Some(project) = self.projects.get_mut(&project_id) {
					project.tasks.move_down(&task_id);
					self.change_was_made();
				}
				Command::none()
			},
			DatabaseMessage::DeleteTask { project_id, task_id } => {
				if let Some(project) = self.projects.get_mut(&project_id) {
					project.tasks.remove(&task_id);
					self.change_was_made();
				}
				Command::none()
			},
		}
	}

	pub fn get_filepath() -> PathBuf {
		let project_dirs = directories::ProjectDirs::from("", "", "ProjectTracker")
				.expect("Failed to get saved state filepath");

		project_dirs.data_local_dir().join(Self::FILE_NAME)
			.to_path_buf()
	}

	async fn get_and_ensure_filepath() -> PathBuf {
		let filepath = Self::get_filepath();

		tokio::fs::create_dir_all(filepath.parent().unwrap()).await.expect("Failed to create Local Data Directories");

		filepath
	}

	pub async fn load_from(filepath: PathBuf) -> LoadDatabaseResult {
		let file_content = if let Ok(file_content) = tokio::fs::read_to_string(filepath.clone()).await {
			file_content
		}
		else {
			return LoadDatabaseResult::FailedToOpenFile(filepath);
		};

		match serde_json::from_str(&file_content) {
			Ok(database) => LoadDatabaseResult::Ok(database),
			Err(_) => LoadDatabaseResult::FailedToParse(filepath),
		}
	}

	pub async fn load() -> LoadDatabaseResult {
		Self::load_from(Self::get_and_ensure_filepath().await).await
	}

	pub async fn save_to(self, filepath: PathBuf) -> Result<(), String> {
		if let Err(e) = tokio::fs::write(filepath.clone(), serde_json::to_string_pretty(&self).unwrap().as_bytes()).await {
			Err(format!("Failed to save to {}: {e}", filepath.display()))
		}
		else {
			Ok(())
		}
	}

	// returns begin time of saving
	async fn save(self) -> Result<Instant, String> {
		let begin_time = Instant::now();
		self.save_to(Self::get_and_ensure_filepath().await).await?;
		Ok(begin_time)
	}

	async fn export_file_dialog(self) -> Result<(), String> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Export ProjectTracker Database")
			.set_file_name(Self::FILE_NAME)
			.add_filter("Database (.json)", &["json"])
			.save_file()
			.await;

		if let Some(result) = file_dialog_result {
			self.save_to(result.path().to_path_buf()).await?;
		}
		Ok(())
	}

	async fn import_file_dialog() -> Option<LoadDatabaseResult> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Import ProjectTracker Database")
			.add_filter("Database (.json)", &["json"])
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

impl Default for Database {
	fn default() -> Self {
		Self::new()
	}
}