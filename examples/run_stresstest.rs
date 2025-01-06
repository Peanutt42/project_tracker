use chrono::Local;
use project_tracker::{run_project_tracker_app, AppFlags, Preferences};
use project_tracker_core::{
	Database, OrderedHashMap, Project, ProjectId, SerializableColor, SortMode, Task, TaskId,
};
use std::{collections::HashSet, time::Duration};
use tokio::time::Instant;
use tracing::info;

#[tokio::main]
async fn main() -> iced::Result {
	tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new()).unwrap();

	let gen_stresstest_start = Instant::now();
	info!("generating stresstest database...");

	let temp_dir = std::env::temp_dir();
	let custom_database_filepath = temp_dir.join(Database::FILE_NAME);
	let custom_preferences_filepath = temp_dir.join(Preferences::FILE_NAME);

	// clean up previous temp files from us
	let _ = std::fs::remove_file(&custom_database_filepath);
	let _ = std::fs::remove_file(&custom_preferences_filepath);

	let mut db = Database::default();

	let today_date = Local::now().naive_local().date();
	let tomorrow_date = today_date + chrono::Duration::days(1);

	for i in 0..20 {
		let mut project = Project::new(
			format!("{i}. Project"),
			SerializableColor::default(),
			OrderedHashMap::new(),
			SortMode::default(),
		);
		for j in 0..1000 {
			let task_id = TaskId::generate();
			let task = Task::new(
				format!("{j}. Task"),
				"A detailed description of the task".to_string(),
				None,
				None,
				if i % 20 == 0 {
					Some(if j % 200 == 0 {
						today_date.into()
					} else {
						tomorrow_date.into()
					})
				} else {
					None
				},
				HashSet::new(),
			);
			if j % 2 == 0 {
				project.todo_tasks.insert(task_id, task);
			} else {
				project.done_tasks.insert(task_id, task);
			}
		}
		db.modify(|projects| projects.insert(ProjectId::generate(), project));
	}

	Database::save(custom_database_filepath.clone(), db.to_binary().unwrap())
		.await
		.unwrap();

	// rounds to milliseconds
	let gen_stresstest_elapsed =
		Duration::from_millis((Instant::now() - gen_stresstest_start).as_millis() as u64);
	info!("finished generating stresstest database, elapsed: {gen_stresstest_elapsed:?}");

	info!("running gui...");

	run_project_tracker_app(AppFlags::custom(
		custom_database_filepath,
		custom_preferences_filepath,
	))
}
