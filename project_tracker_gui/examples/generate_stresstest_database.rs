use std::path::PathBuf;

use project_tracker_gui::core::{Database, Project, Task, TaskState};

#[tokio::main]
async fn main() {
	let mut db = Database::new();

	for i in 0..1000 {
		let mut project = Project::new(format!("{i}. Project"));
		for j in 0..1000 {
			let task = Task::new(format!("{j}. Task"), if j % 2 == 0 { TaskState::Todo } else { TaskState::Done });
			project.tasks.insert(j, task);
		}
		db.projects.insert(i, project);
	}

	db.save_to(PathBuf::from("stresstest_database.json")).await.unwrap();
}