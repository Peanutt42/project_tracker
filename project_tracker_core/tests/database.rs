use project_tracker_core::{
	Database, LoadDatabaseError, OrderedHashMap, Project, ProjectId, SerializableColor, SortMode,
	TaskId,
};
use std::{collections::HashSet, path::PathBuf};

#[tokio::test]
async fn test_database_serialization() {
	let output_filepath: PathBuf =
		PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("tmp_test_database.project_tracker");
	let mut database = Database::default();

	for i in 0..10 {
		let mut project = Project::new(
			format!("Project Nr.{i}"),
			SerializableColor::default(),
			OrderedHashMap::new(),
			SortMode::default(),
		);

		for j in 0..100 {
			project.add_task(
				TaskId::generate(),
				format!("Task Nr. {j}"),
				"A detailed description of the task".to_string(),
				HashSet::new(),
				None,
				None,
				None,
				false,
			);
		}

		database.modify(|projects| projects.insert(ProjectId::generate(), project));
	}

	let original = database.clone();

	Database::save(output_filepath.clone(), database.to_binary().unwrap())
		.await
		.unwrap();

	match Database::load(output_filepath.clone()).await {
		Ok(database) => assert_eq!(database.projects(), original.projects()),
		Err(e) => match e {
			LoadDatabaseError::FailedToOpenFile { .. } => {
				panic!("Failed to find serialized file, maybe database.save_to failed?")
			}
			LoadDatabaseError::FailedToParseBinary { .. }
			| LoadDatabaseError::FailedToParseJson { .. } => {
				panic!("Failed to parse serialized file!")
			}
		},
	};

	tokio::fs::remove_file(&output_filepath)
		.await
		.expect("failed to remove temporary test database file used for serialization testing");
}
