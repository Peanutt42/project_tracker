use crate::{
	OrderedHashMap, Project, ProjectId, SortMode, SerializableColor, SerializableDate, Task, TaskId, TaskTag,
	TaskTagId, TaskType, TimeSpend
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Database {
	projects: OrderedHashMap<ProjectId, Project>,

	#[serde(skip, default = "SystemTime::now")]
	last_changed_time: SystemTime,

	#[serde(skip, default = "SystemTime::now")]
	last_saved_time: SystemTime,
}

#[derive(Clone, Debug)]
pub enum DatabaseMessage {
	Clear,

	ImportProjects(Vec<Project>),

	ImportSourceCodeTodos{
		project_id: ProjectId,
		source_code_todo_tasks: Vec<Task>,
	},

	CreateProject {
		project_id: ProjectId,
		name: String,
		color: SerializableColor,
	},
	ChangeProjectName {
		project_id: ProjectId,
		new_name: String,
	},
	ChangeProjectColor {
		project_id: ProjectId,
		new_color: SerializableColor,
	},
	ChangeProjectSortMode {
		project_id: ProjectId,
		new_sort_mode: SortMode,
	},
	MoveProjectUp(ProjectId),
	MoveProjectDown(ProjectId),
	MoveProjectBeforeOtherProject {
		project_id: ProjectId,
		other_project_id: ProjectId,
	},
	MoveProjectToEnd(ProjectId),
	DeleteProject(ProjectId),
	DeleteDoneTasks(ProjectId),

	MoveTask {
		task_id: TaskId,
		src_project_id: ProjectId,
		dst_project_id: ProjectId,
	},
	MoveTodoTaskToEnd {
		project_id: ProjectId,
		task_id: TaskId,
	},

	CreateTask {
		project_id: ProjectId,
		task_id: TaskId,
		task_name: String,
		task_description: String,
		task_tags: HashSet<TaskTagId>,
		due_date: Option<SerializableDate>,
		needed_time_minutes: Option<usize>,
		time_spend: Option<TimeSpend>,
		create_at_top: bool,
	},
	ChangeTaskName {
		project_id: ProjectId,
		task_id: TaskId,
		new_task_name: String,
	},
	ChangeTaskDescription {
		project_id: ProjectId,
		task_id: TaskId,
		new_task_description: String,
	},
	SetTaskTodo {
		project_id: ProjectId,
		task_id: TaskId,
	},
	SetTaskDone {
		project_id: ProjectId,
		task_id: TaskId,
	},
	ChangeTaskNeededTime {
		project_id: ProjectId,
		task_id: TaskId,
		new_needed_time_minutes: Option<usize>,
	},
	ChangeTaskTimeSpend {
		project_id: ProjectId,
		task_id: TaskId,
		new_time_spend: Option<TimeSpend>,
	},
	StartTaskTimeSpend {
		project_id: ProjectId,
		task_id: TaskId,
	},
	StopTaskTimeSpend {
		project_id: ProjectId,
		task_id: TaskId,
	},
	ChangeTaskDueDate {
		project_id: ProjectId,
		task_id: TaskId,
		new_due_date: Option<SerializableDate>,
	},
	ToggleTaskTag {
		project_id: ProjectId,
		task_id: TaskId,
		task_tag_id: TaskTagId,
	},
	MoveTaskBeforeOtherTask {
		project_id: ProjectId,
		task_id: TaskId,
		other_task_id: TaskId,
	},
	DeleteTask {
		project_id: ProjectId,
		task_id: TaskId,
	},

	CreateTaskTag {
		project_id: ProjectId,
		task_tag_id: TaskTagId,
		task_tag: TaskTag,
	},
	ChangeTaskTagColor {
		project_id: ProjectId,
		task_tag_id: TaskTagId,
		new_color: SerializableColor,
	},
	ChangeTaskTagName {
		project_id: ProjectId,
		task_tag_id: TaskTagId,
		new_name: String,
	},
	DeleteTaskTag {
		project_id: ProjectId,
		task_tag_id: TaskTagId,
	},
}

#[derive(Debug, Error)]
pub enum LoadDatabaseError {
	#[error("failed to open file: {filepath}, error: {error}")]
	FailedToOpenFile {
		filepath: PathBuf,
		error: std::io::Error,
	},
	#[error("failed to parse database: {filepath}, error: {error}")]
	FailedToParse {
		filepath: PathBuf,
		error: bincode::Error,
	},
}

pub type LoadDatabaseResult = Result<Database, LoadDatabaseError>;

#[derive(Clone, Debug)]
pub enum SyncDatabaseResult {
	InvalidSynchronizationFilepath,
	Upload,
	Download,
}

impl Database {
	const FILE_NAME: &'static str = "database.project_tracker";

	pub fn new(projects: OrderedHashMap<ProjectId, Project>) -> Self {
		Self {
			projects,
			last_changed_time: SystemTime::now(),
			last_saved_time: SystemTime::now(),
		}
	}

	pub fn projects(&self) -> &OrderedHashMap<ProjectId, Project> {
		&self.projects
	}

	pub fn get_project(&self, project_id: &ProjectId) -> Option<&Project> {
		self.projects.get(project_id)
	}

	pub fn get_task(&self, project_id: &ProjectId, task_id: &TaskId) -> Option<&Task> {
		self.projects
			.get(project_id)
			.and_then(|project| project.get_task(task_id))
	}

	pub fn get_task_and_type(&self, project_id: &ProjectId, task_id: &TaskId) -> Option<(&Task, TaskType)> {
		self.projects
			.get(project_id)
			.and_then(|project| project.get_task_and_type(task_id))
	}

	pub fn last_changed_time(&self) -> &SystemTime {
		&self.last_changed_time
	}

	pub fn saved(&mut self, saved_time: SystemTime) {
		self.last_saved_time = saved_time;
	}

	pub fn modify(&mut self, f: impl FnOnce(&mut OrderedHashMap<ProjectId, Project>)) {
		f(&mut self.projects);
		self.modified();
	}

	fn modified(&mut self) {
		self.last_changed_time = SystemTime::now();
	}

	pub fn has_same_content_as(&self, other: &Database) -> bool {
		if self.projects.len() != other.projects.len() {
			return false;
		}

		for (project_id, project) in self.projects.iter() {
			if let Some(other_project) = other.projects.get(&project_id) {
				if !project.has_same_content_as(other_project) {
					return false;
				}
			} else {
				return false;
			}
		}

		true
	}

	pub fn has_unsaved_changes(&self) -> bool {
		self.last_changed_time > self.last_saved_time
	}

	pub fn update(&mut self, message: DatabaseMessage) {
		match message {
			DatabaseMessage::Clear => {
				*self = Self::default();
				self.modified();
			}

			DatabaseMessage::ImportProjects(new_projects) => self.modify(|projects| {
				projects.reserve(new_projects.len());
				for new_project in new_projects {
					projects.insert(ProjectId::generate(), new_project);
				}
			}),

			DatabaseMessage::ImportSourceCodeTodos{ project_id, source_code_todo_tasks } => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.source_code_todos.clear();
					for task in source_code_todo_tasks {
						project.source_code_todos.insert(TaskId::generate(), task);
					}
				}
			}),

			DatabaseMessage::CreateProject {
				project_id,
				name,
				color,
			} => self.modify(|projects| {
				projects.insert(project_id, Project::new(name, color, OrderedHashMap::new(), SortMode::default()));
			}),
			DatabaseMessage::ChangeProjectName {
				project_id,
				new_name,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.name = new_name;
				}
			}),
			DatabaseMessage::ChangeProjectColor {
				project_id,
				new_color,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.color = new_color;
				}
			}),
			DatabaseMessage::ChangeProjectSortMode { project_id, new_sort_mode } => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.sort_mode = new_sort_mode;
				}
			}),
			DatabaseMessage::MoveProjectUp(project_id) => {
				self.modify(|projects| projects.move_up(&project_id))
			}
			DatabaseMessage::MoveProjectDown(project_id) => {
				self.modify(|projects| projects.move_down(&project_id))
			}
			DatabaseMessage::MoveTaskBeforeOtherTask {
				project_id,
				task_id,
				other_task_id,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.todo_tasks.move_before_other(task_id, other_task_id);
				}
			}),
			DatabaseMessage::MoveProjectBeforeOtherProject {
				project_id,
				other_project_id,
			} => self.modify(|projects| {
				projects.move_before_other(project_id, other_project_id);
			}),
			DatabaseMessage::MoveProjectToEnd(project_id) => self.modify(|projects| {
				projects.move_to_end(&project_id);
			}),
			DatabaseMessage::DeleteProject(project_id) => self.modify(|projects| {
				projects.remove(&project_id);
			}),
			DatabaseMessage::DeleteDoneTasks(project_id) => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.done_tasks.clear();
				}
			}),

			DatabaseMessage::MoveTask {
				task_id,
				src_project_id,
				dst_project_id,
			} => self.modify(|projects| {
				let removed_task: Option<(TaskType, Task)> = projects
					.get_mut(&src_project_id)
					.map(|src_project| src_project.remove_task(&task_id))
					.unwrap_or(None);

				if let Some((task_type, task)) = removed_task {
					let missing_tags = projects
						.get(&dst_project_id)
						.and_then(|dst_project| {
							projects.get(&src_project_id).map(|src_project| {
								let mut missing_tag_ids = HashMap::new();
								for tag_id in task.tags.iter() {
									if !dst_project.task_tags.contains_key(tag_id) {
										if let Some(tag) = src_project.task_tags.get(tag_id) {
											missing_tag_ids.insert(*tag_id, tag.clone());
										}
									}
								}
								missing_tag_ids
							})
						})
						.unwrap_or_default();

					if let Some(dst_project) = projects.get_mut(&dst_project_id) {
						for (tag_id, tag) in missing_tags {
							dst_project.task_tags.insert(tag_id, tag);
						}

						match task_type {
							TaskType::Todo => dst_project.todo_tasks.insert(task_id, task),
							TaskType::Done => {
								let _ = dst_project.done_tasks.insert(task_id, task);
							}
							TaskType::SourceCodeTodo => {
								let _ = dst_project.source_code_todos.insert(task_id, task);
							}
						}
					}
				}
			}),

			DatabaseMessage::MoveTodoTaskToEnd{ project_id, task_id } => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.todo_tasks.move_to_end(&task_id);
				}
			}),

			DatabaseMessage::CreateTask {
				project_id,
				task_id,
				task_name,
				task_description,
				task_tags,
				due_date,
				needed_time_minutes,
				time_spend,
				create_at_top,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.add_task(task_id, task_name, task_description, task_tags, due_date, needed_time_minutes, time_spend, create_at_top);
				}
			}),
			DatabaseMessage::ChangeTaskName {
				project_id,
				task_id,
				new_task_name,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.set_task_name(task_id, new_task_name);
				}
			}),
			DatabaseMessage::ChangeTaskDescription {
				project_id,
				task_id,
				new_task_description,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.set_task_description(task_id, new_task_description);
				}
			}),
			DatabaseMessage::SetTaskTodo {
				project_id,
				task_id,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.set_task_todo(task_id);
				}
			}),
			DatabaseMessage::SetTaskDone {
				project_id,
				task_id,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.set_task_done(task_id);
				}
			}),
			DatabaseMessage::ChangeTaskNeededTime {
				project_id,
				task_id,
				new_needed_time_minutes,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.set_task_needed_time(task_id, new_needed_time_minutes);
				}
			}),
			DatabaseMessage::ChangeTaskTimeSpend { project_id, task_id, new_time_spend } => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.set_task_time_spend(task_id, new_time_spend);
				}
			}),
			DatabaseMessage::StartTaskTimeSpend { project_id, task_id } => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.start_task_time_spend(task_id);
				}
			}),
			DatabaseMessage::StopTaskTimeSpend { project_id, task_id } => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.stop_task_time_spend(task_id);
				}
			}),
			DatabaseMessage::ChangeTaskDueDate {
				project_id,
				task_id,
				new_due_date,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.set_task_due_date(task_id, new_due_date);
				}
			}),
			DatabaseMessage::ToggleTaskTag {
				project_id,
				task_id,
				task_tag_id,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.toggle_task_tag(task_id, task_tag_id);
				}
			}),
			DatabaseMessage::DeleteTask {
				project_id,
				task_id,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.remove_task(&task_id);
				}
			}),

			DatabaseMessage::CreateTaskTag {
				project_id,
				task_tag_id,
				task_tag,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.task_tags.insert(task_tag_id, task_tag);
				}
			}),
			DatabaseMessage::ChangeTaskTagColor {
				project_id,
				task_tag_id,
				new_color,
			} => self.modify(|projects| {
				if let Some(tag) = projects
					.get_mut(&project_id)
					.and_then(|project| project.task_tags.get_mut(&task_tag_id))
				{
					tag.color = new_color;
				}
			}),
			DatabaseMessage::ChangeTaskTagName {
				project_id,
				task_tag_id,
				new_name,
			} => self.modify(|projects| {
				if let Some(tag) = projects
					.get_mut(&project_id)
					.and_then(|project| project.task_tags.get_mut(&task_tag_id))
				{
					tag.name = new_name;
				}
			}),
			DatabaseMessage::DeleteTaskTag {
				project_id,
				task_tag_id,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.task_tags.remove(&task_tag_id);
					for task in project.todo_tasks.values_mut() {
						task.tags.remove(&task_tag_id);
					}
					for task in project.done_tasks.values_mut() {
						task.tags.remove(&task_tag_id);
					}
				}
			}),
		}
	}

	pub fn get_filepath() -> PathBuf {
		let project_dirs = directories::ProjectDirs::from("", "", "ProjectTracker")
			.expect("Failed to get saved state filepath");

		project_dirs
			.data_local_dir()
			.join(Self::FILE_NAME)
			.to_path_buf()
	}

	async fn get_and_ensure_filepath() -> PathBuf {
		let filepath = Self::get_filepath();

		tokio::fs::create_dir_all(filepath.parent().unwrap())
			.await
			.expect("Failed to create Local Data Directories");

		filepath
	}

	pub async fn load_from(filepath: PathBuf) -> LoadDatabaseResult {
		let file_content = tokio::fs::read(&filepath).await
			.map_err(|error| LoadDatabaseError::FailedToOpenFile{
				filepath: filepath.clone(),
				error
			})?;

		bincode::deserialize(&file_content)
			.map_err(|error| LoadDatabaseError::FailedToParse{
				filepath: filepath.clone(),
				error
			})
	}

	pub async fn load() -> LoadDatabaseResult {
		Self::load_from(Self::get_and_ensure_filepath().await).await
	}

	pub fn to_binary(&self) -> Vec<u8> {
		bincode::serialize(self).unwrap()
	}

	pub async fn save_to(filepath: PathBuf, binary: Vec<u8>) -> Result<(), String> {
		if let Err(e) = tokio::fs::write(filepath.as_path(), binary).await {
			Err(format!("Failed to save to {}: {e}", filepath.display()))
		} else {
			Ok(())
		}
	}

	// returns begin time of saving
	pub async fn save(binary: Vec<u8>) -> Result<SystemTime, String> {
		let begin_time = SystemTime::now();
		Self::save_to(Self::get_and_ensure_filepath().await, binary).await?;
		Ok(begin_time)
	}

	pub async fn sync(synchronization_filepath: PathBuf, local_database_last_change_time: SystemTime) -> SyncDatabaseResult {
		let local_last_modification_datetime: DateTime<Utc> = local_database_last_change_time.into();

		let synchronization_filepath_metadata = match synchronization_filepath.metadata() {
			Ok(metadata) => metadata,
			Err(_) => return SyncDatabaseResult::InvalidSynchronizationFilepath,
		};

		let synchronization_last_modification_datetime = get_last_modification_date_time(&synchronization_filepath_metadata);

		if local_last_modification_datetime
			> synchronization_last_modification_datetime
		{
			SyncDatabaseResult::Upload
		} else {
			SyncDatabaseResult::Download
		}
	}

	pub async fn export_file_dialog() -> Option<PathBuf> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Export ProjectTracker Database")
			.set_file_name(Self::FILE_NAME)
			.add_filter("Database (.project_tracker)", &["project_tracker"])
			.save_file()
			.await;

		file_dialog_result.map(|file_handle| file_handle.path().to_path_buf())
	}

	pub async fn import_file_dialog() -> Option<PathBuf> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Import ProjectTracker Database")
			.add_filter("Database (.project_tracker)", &["project_tracker"])
			.pick_file()
			.await;

		file_dialog_result.map(|file_handle| file_handle.path().to_path_buf())
	}
}

impl Default for Database {
	fn default() -> Self {
		Self::new(OrderedHashMap::new())
	}
}

pub fn get_last_modification_date_time(metadata: &std::fs::Metadata) -> DateTime<Utc> {
	use filetime::FileTime;

	let modified = FileTime::from_last_modification_time(metadata);

	let unix_timestamp = modified.unix_seconds();
	let nanos = modified.nanoseconds();

	DateTime::from_timestamp(unix_timestamp, nanos)
		.expect("invalid file modification date timestamp")
}