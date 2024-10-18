use std::{collections::HashSet, path::PathBuf};

use project_tracker_gui::core::{Database, OrderedHashMap, Project, SortMode, ProjectId, SerializableColor, Task, TaskId};

#[tokio::main]
async fn main() {
	let mut db = Database::default();

	for i in 0..1000 {
		let mut project = Project::new(format!("{i}. Project"), SerializableColor::default(), OrderedHashMap::new(), SortMode::default());
		for j in 0..1000 {
			let task_id = TaskId::generate();
			let task = Task::new(format!("{j}. Task"), "A detailed description of the task".to_string(), None, None, HashSet::new());
			if j % 2 == 0 {
				project.todo_tasks.insert(task_id, task);
			} else {
				project.done_tasks.insert(task_id, task);
			}
		}
		db.modify(|projects| projects.insert(ProjectId(i), project));
	}

	Database::save_to(
		PathBuf::from("stresstest_database.json"),
		serde_json::to_string_pretty(&db).unwrap(),
	)
	.await
	.unwrap();
}
