use crate::{
	duration_str, round_duration_to_minutes, OrderedHashMap, Project, ProjectId, SerializableColor,
	SerializableDate, SortMode, Task, TaskId, TaskTag, TaskTagId, TaskType, TimeSpend,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use thiserror::Error;

pub type SerializedDatabase = OrderedHashMap<ProjectId, Project>;

#[derive(Clone, Debug)]
pub struct Database {
	projects: OrderedHashMap<ProjectId, Project>,
	last_changed_time: DateTime<Utc>,
	last_saved_time: SystemTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DatabaseMessage {
	Clear,

	ImportProjects(Vec<Project>),

	ImportSourceCodeTodos {
		project_id: ProjectId,
		source_code_directory: PathBuf,
		source_code_todo_tasks: OrderedHashMap<TaskId, Task>,
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
		task_tags: BTreeSet<TaskTagId>,
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
	ToggleTaskDescriptionMarkdownTask {
		project_id: ProjectId,
		task_id: TaskId,
		range: Range<usize>,
		checked: bool,
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
	#[error("failed to open file: {filepath}, io-error: {error:?}")]
	FailedToOpenFile {
		filepath: PathBuf,
		error: Option<std::io::Error>,
	},
	#[error("failed to parse database: {filepath}, error: {error}")]
	FailedToParseBinary {
		filepath: PathBuf,
		error: bincode::error::DecodeError,
	},
	#[error("failed to parse database: {filepath}, error: {error}")]
	FailedToParseJson {
		filepath: PathBuf,
		error: serde_json::Error,
	},
}
pub type LoadDatabaseResult = Result<Database, LoadDatabaseError>;

#[derive(Debug, Error)]
pub enum SaveDatabaseError {
	#[error("failed to write to file: {filepath}, error: {error}")]
	FailedToWriteToFile {
		filepath: PathBuf,
		error: std::io::Error,
	},
}
pub type SaveDatabaseResult<T> = Result<T, SaveDatabaseError>;

#[derive(Clone, Debug)]
pub enum SyncDatabaseResult {
	InvalidSynchronizationFilepath,
	Upload,
	Download,
}

impl Database {
	pub const FILE_NAME: &'static str = "database.project_tracker";
	pub const JSON_FILE_NAME: &'static str = "database.json";

	pub fn new(
		projects: OrderedHashMap<ProjectId, Project>,
		last_changed_time: DateTime<Utc>,
	) -> Self {
		Self {
			projects,
			last_changed_time,
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

	pub fn get_task_and_type(
		&self,
		project_id: &ProjectId,
		task_id: &TaskId,
	) -> Option<(&Task, TaskType)> {
		self.projects
			.get(project_id)
			.and_then(|project| project.get_task_and_type(task_id))
	}

	pub fn get_project_task_type(
		&self,
		project_id: &ProjectId,
		task_id: &TaskId,
	) -> Option<(&Project, &Task, TaskType)> {
		self.projects.get(project_id).and_then(|project| {
			project
				.get_task_and_type(task_id)
				.map(|(task, task_type)| (project, task, task_type))
		})
	}

	pub fn last_changed_time(&self) -> &DateTime<Utc> {
		&self.last_changed_time
	}

	pub fn saved(&mut self, saved_time: SystemTime) {
		self.last_saved_time = saved_time;
	}

	pub fn last_saved_time(&self) -> &SystemTime {
		&self.last_saved_time
	}

	pub fn modify<O>(&mut self, f: impl FnOnce(&mut OrderedHashMap<ProjectId, Project>) -> O) -> O {
		let output = f(&mut self.projects);
		self.modified();
		output
	}

	fn modified(&mut self) {
		self.last_changed_time = SystemTime::now().into();
	}

	pub fn has_unsaved_changes(&self) -> bool {
		let last_saved_date_time: DateTime<Utc> = self.last_saved_time.into();
		self.last_changed_time > last_saved_date_time
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

			DatabaseMessage::ImportSourceCodeTodos {
				project_id,
				source_code_todo_tasks,
				source_code_directory,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.source_code_todos = source_code_todo_tasks;
					project.source_code_directory = Some(source_code_directory);
				}
			}),

			DatabaseMessage::CreateProject {
				project_id,
				name,
				color,
			} => self.modify(|projects| {
				projects.insert(
					project_id,
					Project::new(name, color, OrderedHashMap::new(), SortMode::default()),
				);
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
			DatabaseMessage::ChangeProjectSortMode {
				project_id,
				new_sort_mode,
			} => self.modify(|projects| {
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
								dst_project.done_tasks.insert(task_id, task);
							}
							TaskType::SourceCodeTodo => {
								dst_project.source_code_todos.insert(task_id, task);
							}
						}
					}
				}
			}),

			DatabaseMessage::MoveTodoTaskToEnd {
				project_id,
				task_id,
			} => self.modify(|projects| {
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
					project.add_task(
						task_id,
						task_name,
						task_description,
						task_tags,
						due_date,
						needed_time_minutes,
						time_spend,
						create_at_top,
					);
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
			DatabaseMessage::ToggleTaskDescriptionMarkdownTask {
				project_id,
				task_id,
				range,
				checked,
			} => {
				self.modify(|projects| {
					projects.get_mut(&project_id).and_then(|project| {
						project.get_task_mut(&task_id).map(|task| {
							toggle_task_description_markdown_task(
								&mut task.description,
								checked,
								range,
							);
						})
					})
				});
			}
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
			DatabaseMessage::ChangeTaskTimeSpend {
				project_id,
				task_id,
				new_time_spend,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.set_task_time_spend(task_id, new_time_spend);
				}
			}),
			DatabaseMessage::StartTaskTimeSpend {
				project_id,
				task_id,
			} => self.modify(|projects| {
				if let Some(project) = projects.get_mut(&project_id) {
					project.start_task_time_spend(task_id);
				}
			}),
			DatabaseMessage::StopTaskTimeSpend {
				project_id,
				task_id,
			} => self.modify(|projects| {
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
					for task in project.values_mut() {
						task.tags.remove(&task_tag_id);
					}
				}
			}),
		}
	}

	fn get_default_filepath() -> Option<PathBuf> {
		let project_dirs = directories::ProjectDirs::from("", "", "ProjectTracker")?;

		Some(
			project_dirs
				.data_local_dir()
				.join(Self::FILE_NAME)
				.to_path_buf(),
		)
	}

	// either returns custom filepath or the default filepath based on the system
	pub fn get_filepath(custom_filepath: Option<PathBuf>) -> Option<PathBuf> {
		custom_filepath.or(Self::get_default_filepath())
	}

	pub async fn load_json(filepath: PathBuf) -> LoadDatabaseResult {
		let file_metadata =
			filepath
				.metadata()
				.map_err(|e| LoadDatabaseError::FailedToOpenFile {
					filepath: filepath.clone(),
					error: Some(e),
				})?;
		let file_last_modification_time = get_last_modification_date_time(&file_metadata).ok_or(
			LoadDatabaseError::FailedToOpenFile {
				filepath: filepath.clone(),
				error: None,
			},
		)?;
		let file_content = tokio::fs::read_to_string(&filepath).await.map_err(|e| {
			LoadDatabaseError::FailedToOpenFile {
				filepath: filepath.clone(),
				error: Some(e),
			}
		})?;

		let serialized = serde_json::from_str(&file_content).map_err(|error| {
			LoadDatabaseError::FailedToParseJson {
				filepath: filepath.clone(),
				error,
			}
		})?;

		Ok(Self::from_serialized(
			serialized,
			file_last_modification_time,
		))
	}

	pub fn to_json(&self) -> Option<String> {
		serde_json::to_string_pretty(self.serialized()).ok()
	}

	pub async fn export_as_json(filepath: PathBuf, json: String) -> SaveDatabaseResult<()> {
		tokio::fs::write(filepath.as_path(), json)
			.await
			.map_err(|error| SaveDatabaseError::FailedToWriteToFile { filepath, error })
	}

	pub async fn export_as_markdown(
		folder_path: PathBuf,
		serialized_database: SerializedDatabase,
	) -> SaveDatabaseResult<()> {
		let folder_path_clone = folder_path.clone();
		tokio::fs::remove_dir_all(&folder_path_clone)
			.await
			.map_err(move |error| SaveDatabaseError::FailedToWriteToFile {
				filepath: folder_path_clone,
				error,
			})?;
		let folder_path_clone = folder_path.clone();
		tokio::fs::create_dir(&folder_path)
			.await
			.map_err(move |error| SaveDatabaseError::FailedToWriteToFile {
				filepath: folder_path_clone,
				error,
			})?;

		for (_project_id, project) in serialized_database.iter() {
			let mut project_file_content = String::new();

			for (_task_id, task, task_type) in project.iter() {
				let task_type_checkbox_str = if task_type.is_done() { 'X' } else { ' ' };
				let task_tags_str: String = task
					.tags
					.iter()
					.filter_map(|task_tag_id| {
						project
							.task_tags
							.get(task_tag_id)
							.map(|task_tag| format!("#{} ", task_tag.name))
					})
					.collect();
				let task_name_str = task.name.replace('<', " < ").replace('>', " > ");
				let due_date_str = match &task.due_date {
					Some(due_date) => {
						format!(" 📅 {}-{}-{}", due_date.year, due_date.month, due_date.day)
					}
					None => String::new(),
				};
				let time_spend_str = match (&task.time_spend, &task.needed_time_minutes) {
					(Some(time_spend), Some(needed_time_minutes)) => format!(
						" {}/{}",
						duration_str(round_duration_to_minutes(time_spend.get_duration())),
						duration_str(Duration::from_secs(*needed_time_minutes as u64 * 60)),
					),
					(None, Some(needed_time_minutes)) => format!(
						" time needed: {}",
						duration_str(Duration::from_secs(*needed_time_minutes as u64 * 60))
					),
					(Some(time_spend), None) => format!(
						" time spend: {}",
						duration_str(round_duration_to_minutes(time_spend.get_duration())),
					),
					(None, None) => String::new(),
				};
				let task_description_prefix = if task.description.is_empty() {
					""
				} else {
					"\n"
				};
				let task_description_str: String =
					task.description
						.lines()
						.fold(String::new(), |mut output, line| {
							let _ = writeln!(output, "\t{line}");
							output
						});
				project_file_content.push_str(&format!(
					"- [{task_type_checkbox_str}] {task_tags_str} {task_name_str}{due_date_str}{time_spend_str}{task_description_prefix}{task_description_str}\n"
				));
			}
			let filepath = folder_path.join(format!("{}.md", project.name));
			tokio::fs::write(&filepath, project_file_content)
				.await
				.map_err(move |error| SaveDatabaseError::FailedToWriteToFile { filepath, error })?;
		}
		Ok(())
	}

	pub async fn load(filepath: PathBuf) -> LoadDatabaseResult {
		let file_metadata =
			filepath
				.metadata()
				.map_err(|e| LoadDatabaseError::FailedToOpenFile {
					filepath: filepath.clone(),
					error: Some(e),
				})?;
		let file_last_modification_time = get_last_modification_date_time(&file_metadata).ok_or(
			LoadDatabaseError::FailedToOpenFile {
				filepath: filepath.clone(),
				error: None,
			},
		)?;
		let file_content =
			tokio::fs::read(&filepath)
				.await
				.map_err(|e| LoadDatabaseError::FailedToOpenFile {
					filepath: filepath.clone(),
					error: Some(e),
				})?;

		Self::from_binary(&file_content, file_last_modification_time).map_err(|error| {
			LoadDatabaseError::FailedToParseBinary {
				filepath: filepath.clone(),
				error,
			}
		})
	}

	/// for now just uses the hash of 'self.projects'
	pub fn checksum(&self) -> u64 {
		let mut hasher = std::hash::DefaultHasher::default();
		self.serialized().hash(&mut hasher);
		hasher.finish()
	}

	pub fn serialized(&self) -> &SerializedDatabase {
		&self.projects
	}

	pub fn into_serialized(self) -> SerializedDatabase {
		self.projects
	}

	pub fn from_serialized(
		serialized: SerializedDatabase,
		last_changed_time: DateTime<Utc>,
	) -> Self {
		Self::new(serialized, last_changed_time)
	}

	pub fn from_binary(
		binary: &[u8],
		last_changed_time: DateTime<Utc>,
	) -> Result<Self, bincode::error::DecodeError> {
		let (serialized_database, _) =
			bincode::serde::decode_from_slice(binary, bincode::config::legacy())?;
		Ok(Self::from_serialized(
			serialized_database,
			last_changed_time,
		))
	}

	pub fn to_binary(&self) -> Option<Vec<u8>> {
		bincode::serde::encode_to_vec(self.serialized(), bincode::config::legacy()).ok()
	}

	// returns begin time of saving
	pub async fn save(filepath: PathBuf, binary: Vec<u8>) -> SaveDatabaseResult<SystemTime> {
		let begin_time = SystemTime::now();
		if let Some(parent_filepath) = filepath.parent() {
			// if this fails, 'tokio::fs::write' will also fail --> correct io error
			let _ = tokio::fs::create_dir_all(parent_filepath).await;
		}
		tokio::fs::write(filepath.as_path(), binary)
			.await
			.map_err(|error| SaveDatabaseError::FailedToWriteToFile { filepath, error })?;
		Ok(begin_time)
	}

	pub async fn sync(
		synchronization_filepath: PathBuf,
		local_database_last_change_time: DateTime<Utc>,
	) -> SyncDatabaseResult {
		let synchronization_filepath_metadata = match synchronization_filepath.metadata() {
			Ok(metadata) => metadata,
			Err(_) => return SyncDatabaseResult::InvalidSynchronizationFilepath,
		};

		match get_last_modification_date_time(&synchronization_filepath_metadata) {
			Some(synchronization_last_modification_datetime) => {
				if local_database_last_change_time > synchronization_last_modification_datetime {
					SyncDatabaseResult::Upload
				} else {
					SyncDatabaseResult::Download
				}
			}
			None => SyncDatabaseResult::Download,
		}
	}
}

impl Default for Database {
	fn default() -> Self {
		Self::new(OrderedHashMap::new(), Utc::now())
	}
}

pub fn toggle_task_description_markdown_task(
	task_description: &mut String,
	checked: bool,
	range: Range<usize>,
) {
	task_description.replace_range(range, if checked { "[X]" } else { "[ ]" });
}

pub fn get_last_modification_date_time(metadata: &std::fs::Metadata) -> Option<DateTime<Utc>> {
	use filetime::FileTime;

	let modified = FileTime::from_last_modification_time(metadata);

	let unix_timestamp = modified.unix_seconds();
	let nanos = modified.nanoseconds();

	DateTime::from_timestamp(unix_timestamp, nanos)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod tests {
	use chrono::Utc;

	use crate::{
		Database, LoadDatabaseError, OrderedHashMap, Project, ProjectId, SerializableColor,
		SerializableDate, SortMode, TaskId, TimeSpend,
	};
	use std::{collections::BTreeSet, path::PathBuf};

	fn create_test_database() -> Database {
		let mut database = Database::default();

		let mut time_spend = TimeSpend::new(0.0);
		time_spend.start();

		for i in 0..10 {
			let mut project = Project::new(
				format!("Project Nr.{i}"),
				SerializableColor::default(),
				OrderedHashMap::new(),
				SortMode::default(),
			);

			for j in 0..100 {
				project.add_task(
					TaskId::generate(),
					format!("Task Nr. {j}"),
					"A detailed description of the task".to_string(),
					BTreeSet::new(),
					Some(SerializableDate {
						year: 2000,
						month: 1,
						day: 1,
					}),
					Some(30),
					Some(time_spend.clone()),
					false,
				);
			}

			database.modify(|projects| projects.insert(ProjectId::generate(), project));
		}

		database
	}

	#[tokio::test]
	async fn test_database_serialization() {
		let output_filepath: PathBuf =
			std::env::temp_dir().join("tmp_test_database.project_tracker");

		let database = create_test_database();

		let original = database.clone();

		Database::save(output_filepath.clone(), database.to_binary().unwrap())
			.await
			.unwrap();

		match Database::load(output_filepath.clone()).await {
			Ok(database) => assert_eq!(database.projects(), original.projects()),
			Err(e) => match e {
				LoadDatabaseError::FailedToOpenFile { .. } => {
					panic!("Failed to find serialized file, maybe database.save_to failed?")
				}
				LoadDatabaseError::FailedToParseBinary { .. }
				| LoadDatabaseError::FailedToParseJson { .. } => {
					panic!("Failed to parse serialized file!")
				}
			},
		};

		tokio::fs::remove_file(&output_filepath)
			.await
			.expect("failed to remove temporary test database file used for serialization testing");
	}

	#[test]
	fn test_database_checksum() {
		let database = create_test_database();
		let database_binary = database.to_binary().unwrap();
		let loaded_from_binary_database =
			Database::from_binary(&database_binary, Utc::now()).unwrap();
		assert_eq!(database.checksum(), loaded_from_binary_database.checksum());
	}
}
