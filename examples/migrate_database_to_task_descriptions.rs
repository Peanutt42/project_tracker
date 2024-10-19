use std::path::PathBuf;

use project_tracker::core::{Database, LoadDatabaseResult, OrderedHashMap, Project, ProjectId, SortMode, Task};



#[tokio::main]
async fn main() {
	let mut args = std::env::args();

	if let Some(filepath) = args.nth(1) {
		let filepath_buf = PathBuf::from(filepath);
		match Database::load_from(filepath_buf.clone()).await {
			LoadDatabaseResult::Ok(db) => {
				let mut migrated_projects: OrderedHashMap<ProjectId, Project> = OrderedHashMap::new();

				let migrate_task = |task: &Task| -> Task {
					if let Some((name, description)) = task.name().split_once('\n') {
						Task::new(
							name.to_string(),
							description.to_string(),
							task.needed_time_minutes,
							task.due_date,
							task.tags.clone()
						)
					}
					else {
						task.clone()
					}
				};

				for (project_id, project) in db.projects().iter() {
					let mut migrated_project = Project::new(project.name.clone(), project.color, project.task_tags.clone(), SortMode::default());

					for (task_id, task) in project.todo_tasks.iter() {
						migrated_project.todo_tasks.insert(task_id, migrate_task(task));
					}

					for (task_id, task) in project.source_code_todos.iter() {
						migrated_project.source_code_todos.insert(*task_id, migrate_task(task));
					}

					for (task_id, task) in project.done_tasks.iter() {
						migrated_project.done_tasks.insert(*task_id, migrate_task(task));
					}

					migrated_projects.insert(project_id, migrated_project);
				}

				let mut migrated_filepath = filepath_buf.clone();
				migrated_filepath.set_file_name("migrated database.json");

				match Database::save_to(
					migrated_filepath.clone(),
					Database::new(migrated_projects).to_json()
				).await {
					Ok(_) => println!(
						"successfully migrated database, saved to: {}",
						migrated_filepath.display()
					),

					Err(e) => eprintln!(
						"failed to save migrated database to: {}, {e}",
						migrated_filepath.display()
					),
				}
			},
			LoadDatabaseResult::FailedToOpenFile(filepath) => {
				eprintln!("failed to open filepath: {}", filepath.display());
			},
			LoadDatabaseResult::FailedToParse(filepath) => {
				eprintln!("failed to parse database in: {}", filepath.display());
			},
		}
	}
	else {
		println!("usage: migrate_database_to_task_descriptions [filepath]");
	}
}