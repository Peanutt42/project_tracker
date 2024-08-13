use std::{collections::BTreeSet, path::PathBuf};

use project_tracker_gui::core::{Database, Project, ProjectId, Task};

#[tokio::main]
async fn main() {
	let mut db = Database::new();

	for i in 0..1000 {
		let mut project = Project::new(format!("{i}. Project"));
		for j in 0..1000 {
			let task = Task::new(format!("{j}. Task"), BTreeSet::new());
			if j % 2 == 0 {
				project.todo_tasks.insert(j, task);
			}
			else {
				project.done_tasks.insert(j, task);
			}
		}
		db.modify(|projects| projects.insert(ProjectId(i), project));
	}

	Database::save_to(PathBuf::from("stresstest_database.json"), serde_json::to_string_pretty(&db).unwrap()).await.unwrap();
}