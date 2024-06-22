use std::path::PathBuf;
use project_tracker_gui::core::{generate_project_id, generate_task_id, Database, LoadDatabaseResult, Project};

#[tokio::test]
async fn test_database_serialization() {
	let output_filepath: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("tmp_test_database.json");
	let mut database = Database::new();

	for i in 0..10 {
		let mut project = Project::new(format!("Project Nr.{i}"));

		for j in 0..100 {
			project.add_task(generate_task_id(), format!("Task Nr. {j}"));
		}

		database.projects.insert(generate_project_id(), project);
	}

	let original = database.clone();

	database.save_to(output_filepath.clone()).await;

	let load_result = Database::load_from(output_filepath.clone()).await;
	assert_eq!(load_result, LoadDatabaseResult::Ok(original));

	tokio::fs::remove_file(output_filepath).await.expect("failed to remove temporary test database file used for serialization testing");
}