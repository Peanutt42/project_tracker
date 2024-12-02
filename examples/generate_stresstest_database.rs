use std::{collections::HashSet, path::PathBuf};
use chrono::{Duration, Local};
use project_tracker::core::{Database, OrderedHashMap, Project, ProjectId, SerializableColor, SortMode, Task, TaskId};

#[tokio::main]
async fn main() {
	let mut db = Database::default();

	let today_date = Local::now().naive_local().date();
	let tomorrow_date = today_date + Duration::days(1);

	for i in 0..20 {
		let mut project = Project::new(format!("{i}. Project"), SerializableColor::default(), OrderedHashMap::new(), SortMode::default());
		for j in 0..1000 {
			let task_id = TaskId::generate();
			let task = Task::new(
				format!("{j}. Task"),
				"A detailed description of the task".to_string(),
				None,
				None,
				if i % 20 == 0 {
					Some(
						if j % 200 == 0 {
							today_date
						}
						else {
							tomorrow_date
						}
						.into()
					)
				} else {
					None
				},
				HashSet::new()
			);
			if j % 2 == 0 {
				project.todo_tasks.insert(task_id, task);
			} else {
				project.done_tasks.insert(task_id, task);
			}
		}
		db.modify(|projects| projects.insert(ProjectId(i), project));
	}

	Database::save_to(
		PathBuf::from("stresstest_database.project_tracker"),
		db.to_binary(),
	)
	.await
	.unwrap();
}
