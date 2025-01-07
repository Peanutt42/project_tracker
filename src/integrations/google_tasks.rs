use std::{collections::HashSet, path::PathBuf};

use chrono::{DateTime, Datelike, Local, TimeZone};
use iced::Color;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use project_tracker_core::{
	OrderedHashMap, Project, SerializableColor, SerializableDate, SortMode, Task, TaskId,
};

use crate::core::IcedColorConversion;

#[derive(Debug, Error)]
pub enum ImportGoogleTasksError {
	#[error("failed to open google tasks file: {filepath}, error: {error}")]
	FailedToOpenFile {
		filepath: PathBuf,
		error: std::io::Error,
	},
	#[error("failed to parse google tasks: {filepath}, error: {error}")]
	ParseError {
		filepath: PathBuf,
		error: serde_json::Error,
	},
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleTasksFormat {
	kind: String,
	items: Vec<GoogleTasksList>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleTasksList {
	kind: String,
	id: String,
	title: String,
	updated: Option<String>,
	#[serde(default)]
	items: Vec<GoogleTasksTask>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleTasksTask {
	kind: String,
	id: String,
	title: String,
	#[serde(default)]
	notes: Option<String>,
	#[serde(default)]
	due: Option<String>,
	#[serde(default)]
	status: Option<String>,
}

pub async fn import_google_tasks(
	filepath: PathBuf,
) -> Result<Vec<Project>, ImportGoogleTasksError> {
	let json = tokio::fs::read_to_string(&filepath)
		.await
		.map_err(|error| ImportGoogleTasksError::FailedToOpenFile {
			filepath: filepath.clone(),
			error,
		})?;

	import_google_tasks_json(&json)
		.map_err(|error| ImportGoogleTasksError::ParseError { filepath, error })
}

pub fn import_google_tasks_json(json: &str) -> Result<Vec<Project>, serde_json::Error> {
	let google_tasks_format: GoogleTasksFormat = serde_json::from_str(json)?;

	let projects: Vec<Project> = google_tasks_format
		.items
		.into_iter()
		.map(|google_tasks_list| {
			let mut project = Project::new(
				google_tasks_list.title,
				SerializableColor::from_iced_color(Color::WHITE),
				OrderedHashMap::new(),
				SortMode::default(),
			);

			for google_tasks_task in google_tasks_list.items {
				let is_todo = google_tasks_task
					.status
					.map(|status| status == "needsAction")
					.unwrap_or(true);

				let task_name = google_tasks_task.title;
				let task_description = google_tasks_task.notes.unwrap_or_default();
				let task_due_date = google_tasks_task.due.and_then(|due_date_str| {
					DateTime::parse_from_rfc3339(&due_date_str)
						.map(|parsed_due_date| {
							let local_due_date =
								Local.from_utc_datetime(&parsed_due_date.naive_utc());
							SerializableDate {
								year: local_due_date.year(),
								month: local_due_date.month(),
								day: local_due_date.day(),
							}
						})
						.ok()
				});

				let task = Task::new(
					task_name,
					task_description,
					None,
					None,
					task_due_date,
					HashSet::new(),
				);

				let task_id = TaskId::generate();

				if is_todo {
					project.todo_tasks.insert(task_id, task);
				} else {
					project.done_tasks.insert(task_id, task);
				}
			}

			project
		})
		.collect();

	Ok(projects)
}

pub async fn import_google_tasks_dialog() -> Option<Result<Vec<Project>, ImportGoogleTasksError>> {
	let file_dialog_result = rfd::AsyncFileDialog::new()
		.set_title("Import Google Tasks Takeout")
		.add_filter("Tasks.json (.json)", &["json"])
		.pick_file()
		.await;

	match file_dialog_result {
		Some(file_handle) => {
			let filepath = file_handle.path().to_path_buf();
			Some(import_google_tasks(filepath).await)
		}
		None => None,
	}
}
