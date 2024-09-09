use std::{collections::HashSet, path::PathBuf};
use project_tracker_gui::core::{generate_task_id, Database, LoadDatabaseResult, Project, ProjectId, SerializableColor};

#[tokio::test]
async fn test_database_serialization() {
	let output_filepath: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("tmp_test_database.json");
	let mut database = Database::default();

	for i in 0..10 {
		let mut project = Project::new(format!("Project Nr.{i}"), SerializableColor::default());

		for j in 0..100 {
			project.add_task(generate_task_id(), format!("Task Nr. {j}"), HashSet::new(), false);
		}

		database.modify(|projects| projects.insert(ProjectId::generate(), project));
	}

	let original = database.clone();

	Database::save_to(output_filepath.clone(), database.to_json()).await.unwrap();

	match Database::load_from(output_filepath.clone()).await {
		LoadDatabaseResult::Ok(database) => assert!(database.has_same_content_as(&original)),
		LoadDatabaseResult::FailedToOpenFile(_) => panic!("Failed to find serialized file, maybe database.save_to failed?"),
		LoadDatabaseResult::FailedToParse(_) => panic!("Failed to parse serialized file!"),
	};

	tokio::fs::remove_file(&output_filepath).await.expect("failed to remove temporary test database file used for serialization testing");
}