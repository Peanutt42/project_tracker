use project_tracker::{core::{Database, OrderedHashMap, Project, ProjectId, SerializableColor, SortMode, TaskId}, integrations::{sync_database_from_server, ServerConfig, SyncServerDatabaseResponse}};
use project_tracker_server::{get_last_modification_date_time, run_server, DEFAULT_PASSWORD, DEFAULT_PORT};
use tokio::fs::read_to_string;
use std::{collections::HashSet, path::PathBuf};

#[tokio::test]
async fn localhost_server_test() {
	let tmp_client_database_filepath = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
		.join("tmp_test_client_database.json");

	let mut test_client_database = Database::default();
	Database::save_to(tmp_client_database_filepath.clone(), test_client_database.to_json())
		.await
		.unwrap();

	let tmp_server_database_filepath = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
		.join("tmp_test_server_database.json");

	let mut test_server_database = Database::new(OrderedHashMap::default());
	for i in 0..10 {
		let mut project = Project::new(
			format!("Project Nr.{i}"),
			SerializableColor::default(),
			OrderedHashMap::new(),
			SortMode::default()
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

		test_server_database.modify(|projects| projects.insert(ProjectId::generate(), project));
	}
	Database::save_to(tmp_server_database_filepath.clone(), test_server_database.to_json())
		.await
		.unwrap();

	// start test server
	let tmp_server_database_filepath_clone = tmp_server_database_filepath.clone();
	std::thread::spawn(|| run_server(
		DEFAULT_PORT,
		tmp_server_database_filepath_clone,
		DEFAULT_PASSWORD.to_string()
	));

	let test_server_config = ServerConfig {
		hostname: "localhost".to_string(),
		port: DEFAULT_PORT,
		password: DEFAULT_PASSWORD.to_string()
	};

	// sync with server: should download server db to client db
	match sync_database_from_server(
		test_server_config.clone(),
		get_last_modification_date_time(&tmp_client_database_filepath.metadata().unwrap()),
		test_client_database.clone()
	)
	.await
	.unwrap() {
		SyncServerDatabaseResponse::DownloadedDatabase(updated_database) => {
			test_client_database = updated_database.clone();
			Database::save_to(tmp_client_database_filepath.clone(), updated_database.to_json())
				.await
				.unwrap();
		},
		SyncServerDatabaseResponse::UploadedDatabase =>
			panic!("since the server database is more up to date, client db shouldn't override server db!"),
	}

	assert_eq!(
		read_to_string(tmp_client_database_filepath.clone()).await.unwrap(),
		read_to_string(tmp_server_database_filepath.clone()).await.unwrap()
	);

	// client adds project -> should upload client db to server db
	test_client_database.modify(|projects| {
		projects.insert(
			ProjectId::generate(),
			Project::new(
				"Added project to cause a upload sync".to_string(),
				SerializableColor::default(),
				OrderedHashMap::default(),
				SortMode::default()
			)
		);
	});

	Database::save_to(tmp_client_database_filepath.clone(), test_client_database.to_json())
		.await
		.unwrap();

	match sync_database_from_server(
		test_server_config,
		get_last_modification_date_time(&tmp_client_database_filepath.metadata().unwrap()),
		test_client_database.clone()
	)
	.await
	.unwrap() {
		SyncServerDatabaseResponse::DownloadedDatabase(_) =>
			panic!("since client db is more up to date, we should have uploaded that to server!"),
		SyncServerDatabaseResponse::UploadedDatabase => {},
	}

	assert_eq!(
		read_to_string(tmp_client_database_filepath.clone()).await.unwrap(),
		read_to_string(tmp_server_database_filepath.clone()).await.unwrap()
	);
}