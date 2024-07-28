use std::path::PathBuf;
use std::time::Instant;
use iced::Command;
use serde::{Serialize, Deserialize};
use crate::components::ErrorMsgModalMessage;
use crate::project_tracker::UiMessage;
use crate::core::{OrderedHashMap, ProjectId, Project, SerializableColor, TaskId, TaskState};

fn default_false() -> bool { false }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Database {
	pub projects: OrderedHashMap<ProjectId, Project>,

	#[serde(skip, default = "default_false")]
	pub syncing: bool,

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
	Export(PathBuf),
	ExportDialog,
	ExportDialogCanceled,
	Exported,
	Import(PathBuf),
	ImportDialog,
	ImportDialogCanceled,
	Sync(PathBuf),
	SyncUpload(PathBuf),
	SyncUploaded,
	SyncFailed(String), // error_msg

	CreateProject {
		project_id: ProjectId,
		name: String,
	},
	ChangeProjectName {
		project_id: ProjectId,
		new_name: String,
	},
	ChangeProjectColor {
		project_id: ProjectId,
		new_color: SerializableColor,
	},
	MoveProjectUp(ProjectId),
	MoveProjectDown(ProjectId),
	SwapProjectOrder {
		project_a_id: ProjectId,
		project_b_id: ProjectId,
	},
	DeleteProject(ProjectId),
	DeleteDoneTasks(ProjectId),

	MoveTask {
		task_id: TaskId,
		src_project_id: ProjectId,
		dst_project_id: ProjectId,
	},

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
	SwapTasks {
		project_id: ProjectId,
		task_a_id: TaskId,
		task_b_id: TaskId,
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

#[derive(Clone, Debug)]
enum SyncDatabaseResult {
	InvalidSynchronizationFilepath,
	Upload,
	Download
}

impl Database {
	const FILE_NAME: &'static str = "database.json";

	pub fn new() -> Self {
		Self {
			projects: OrderedHashMap::new(),
			syncing: false,
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
			DatabaseMessage::Export(filepath) => Command::perform(self.clone().save_to(filepath), |_| DatabaseMessage::Exported.into()),
			DatabaseMessage::ExportDialog => Command::perform(
				Self::export_file_dialog(),
				|filepath| {
					match filepath {
						Some(filepath) => DatabaseMessage::Export(filepath).into(),
						None => DatabaseMessage::ExportDialogCanceled.into(),
					}
				}
			),
			DatabaseMessage::Exported => Command::none(),
			DatabaseMessage::Import(filepath) => Command::perform(Self::load_from(filepath), UiMessage::LoadedDatabase),
			DatabaseMessage::ImportDialog => Command::perform(
				Self::import_file_dialog(),
				|filepath| {
					if let Some(filepath) = filepath {
						DatabaseMessage::Import(filepath).into()
					}
					else {
						DatabaseMessage::ImportDialogCanceled.into()
					}
				}
			),
			DatabaseMessage::ImportDialogCanceled | DatabaseMessage::ExportDialogCanceled => Command::none(),
			DatabaseMessage::Sync(filepath) => {
				self.syncing = true;
				Command::perform(Self::sync(filepath.clone()), |result| {
					match result {
						SyncDatabaseResult::InvalidSynchronizationFilepath => DatabaseMessage::SyncFailed(format!("Failed to open synchronization file in\n\"{}\"", filepath.display())).into(),
						SyncDatabaseResult::Upload => DatabaseMessage::SyncUpload(filepath).into(),
						SyncDatabaseResult::Download => DatabaseMessage::Import(filepath).into(),
					}
				})
			},
			DatabaseMessage::SyncUpload(filepath) => Command::perform(
				self.clone().save_to(filepath),
				|_| DatabaseMessage::SyncUploaded.into()
			),
			DatabaseMessage::SyncUploaded | DatabaseMessage::SyncFailed(_) => { self.syncing = false; Command::none() },

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
			DatabaseMessage::ChangeProjectColor { project_id, new_color } => {
				if let Some(project) = self.projects.get_mut(&project_id) {
					project.color = new_color;
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
			DatabaseMessage::SwapTasks { project_id, task_a_id, task_b_id } => {
				if let Some(project) = self.projects.get_mut(&project_id) {
					project.tasks.swap_order(&task_a_id, &task_b_id);
					self.change_was_made();
				}
				Command::none()
			},
			DatabaseMessage::SwapProjectOrder { project_a_id, project_b_id } => {
				self.projects.swap_order(&project_a_id, &project_b_id);
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

			DatabaseMessage::MoveTask { task_id, src_project_id, dst_project_id } => {
				let removed_task = if let Some(src_project) = self.projects.get_mut(&src_project_id) {
					src_project.tasks.remove(&task_id)
				}
				else {
					None
				};
				if let Some(task) = removed_task {
					if let Some(destination_project) = self.projects.get_mut(&dst_project_id) {
						destination_project.tasks.insert(task_id, task);
					}
				}
				self.change_was_made();
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

	async fn sync(synchronization_filepath: PathBuf) -> SyncDatabaseResult {
		use filetime::FileTime;

		let synchronization_filepath_metadata = match synchronization_filepath.metadata() {
			Ok(metadata) =>  metadata,
			Err(_) => return SyncDatabaseResult::InvalidSynchronizationFilepath,
		};

		let local_filepath = Self::get_filepath();
		let local_filepath_metadata = match local_filepath.metadata() {
			Ok(metadata) => metadata,
			Err(_) => return SyncDatabaseResult::Download,
		};

		if FileTime::from_last_modification_time(&local_filepath_metadata) > FileTime::from_last_modification_time(&synchronization_filepath_metadata) {
			SyncDatabaseResult::Upload
		}
		else {
			SyncDatabaseResult::Download
		}
	}

	async fn export_file_dialog() -> Option<PathBuf> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Export ProjectTracker Database")
			.set_file_name(Self::FILE_NAME)
			.add_filter("Database (.json)", &["json"])
			.save_file()
			.await;

		file_dialog_result.map(|file_handle| file_handle.path().to_path_buf())
	}

	pub async fn import_file_dialog() -> Option<PathBuf> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Import ProjectTracker Database")
			.add_filter("Database (.json)", &["json"])
			.pick_file()
			.await;

		file_dialog_result.map(|file_handle| file_handle.path().to_path_buf())
	}
}

impl Default for Database {
	fn default() -> Self {
		Self::new()
	}
}