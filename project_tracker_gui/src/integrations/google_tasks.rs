use std::{
	collections::HashSet,
	path::{Path, PathBuf},
};

use chrono::{DateTime, Datelike};
use iced::Color;
use serde::{Deserialize, Serialize};

use crate::core::{OrderedHashMap, Project, SerializableDate, SortMode, Task, TaskId};

#[derive(Debug)]
pub enum ImportGoogleTasksError {
	IoError(std::io::Error),
	ParseError(serde_json::Error),
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
	filepath: impl AsRef<Path>,
) -> Result<Vec<Project>, ImportGoogleTasksError> {
	let json = tokio::fs::read_to_string(filepath)
		.await
		.map_err(ImportGoogleTasksError::IoError)?;

	import_google_tasks_json(&json).map_err(ImportGoogleTasksError::ParseError)
}

pub fn import_google_tasks_json(json: &str) -> Result<Vec<Project>, serde_json::Error> {
	let google_tasks_format: GoogleTasksFormat = serde_json::from_str(json)?;

	let projects: Vec<Project> = google_tasks_format
		.items
		.into_iter()
		.map(|google_tasks_list| {
			let mut project = Project::new(google_tasks_list.title, Color::WHITE.into(), OrderedHashMap::new(), SortMode::default());

			for google_tasks_task in google_tasks_list.items {
				let is_todo = google_tasks_task
					.status
					.map(|status| status == "needsAction")
					.unwrap_or(true);

				let task_name = google_tasks_task.title;
				let task_description = google_tasks_task.notes.unwrap_or_default();
				let mut task = Task::new(task_name, task_description, None, None, HashSet::new());
				task.due_date = google_tasks_task.due.and_then(|due_date_str| {
					DateTime::parse_from_rfc3339(&due_date_str)
						.map(|parsed_due_date| {
							let naive_due_date = parsed_due_date.naive_utc().date();
							SerializableDate {
								year: naive_due_date.year(),
								month: naive_due_date.month(),
								day: naive_due_date.day(),
							}
						})
						.ok()
				});

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

pub async fn import_google_tasks_dialog(
) -> Option<(Result<Vec<Project>, ImportGoogleTasksError>, PathBuf)> {
	let file_dialog_result = rfd::AsyncFileDialog::new()
		.set_title("Import Google Tasks Takeout")
		.add_filter("Tasks.json (.json)", &["json"])
		.pick_file()
		.await;

	if let Some(file_handle) = file_dialog_result {
		let filepath = file_handle.path().to_path_buf();
		Some((import_google_tasks(&filepath).await, filepath))
	} else {
		None
	}
}
