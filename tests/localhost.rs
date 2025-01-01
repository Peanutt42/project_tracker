use iced::futures::{channel::mpsc::Receiver, StreamExt};
use project_tracker::{
	integrations::{
		ServerConfig, ServerWsEvent, ServerWsMessage, WsServerConnection, WsServerConnectionState,
	},
	Database, OrderedHashMap, Project, ProjectId, SerializableColor, SortMode, TaskId,
};
use project_tracker_server::{EncryptedResponse, ModifiedEvent, Request, SharedServerData};
use std::{
	collections::HashSet,
	path::PathBuf,
	sync::{Arc, RwLock},
};
use tokio::{net::TcpListener, sync::broadcast::Sender};

const TEST_HOSTNAME: &str = "localhost";
const TEST_PORT: usize = 8000;
const TEST_PASSWORD: &str = "testing password 1234!";

/*
1. server sets up a empty db
2. client connects to server and requests to download db
3. after receiving empty db, client generates actual db with 'generate_testing_db' and updates the server
4. after client receives confirmation 'DatabaseUpdated' from server, client sends last request to download server db
5. after client receives the database -> confirm if the downloaded db matches the local generated db
*/
#[tokio::test]
async fn localhost_client_and_server() {
	let server_test_db_filepath: PathBuf =
		PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("tmp_test_server_database.project_tracker");

	let shared_data = SharedServerData::from_memory(Database::default());
	let (modified_sender, _modified_receiver) = tokio::sync::broadcast::channel(10);

	spawn_server_thread(
		server_test_db_filepath,
		shared_data.clone(),
		modified_sender,
	);

	// test client
	let mut client_state = WsServerConnectionState::new();
	let mut client_connection = WsServerConnection::Disconnected;
	let (mut client_output, client_output_receiver) = iced::futures::channel::mpsc::channel(100);

	// listens to 'ServerWsEvent'
	spawn_client_listen_events_thread(client_output_receiver);

	while client_state
		.update(&mut client_connection, &mut client_output)
		.await
	{}
}

// simplified 'project_tracker_server::run_server' procedure to only handle the one test client and then quit
fn spawn_server_thread(
	database_filepath: PathBuf,
	shared_data: Arc<RwLock<SharedServerData>>,
	modified_sender: Sender<ModifiedEvent>,
) {
	tokio::spawn(async move {
		let listener = TcpListener::bind(format!("{TEST_HOSTNAME}:{TEST_PORT}"))
			.await
			.expect("Failed to bind to port");

		let (stream, _addr) = listener
			.accept()
			.await
			.expect("failed to establish a connection to test client");

		let modified_receiver = modified_sender.subscribe();
		project_tracker_server::handle_client(
			stream,
			database_filepath,
			TEST_PASSWORD.to_string(),
			modified_sender,
			modified_receiver,
			shared_data,
		)
		.await;
	});
}

fn spawn_client_listen_events_thread(mut receiver: Receiver<ServerWsEvent>) {
	tokio::spawn(async move {
		let test_server_config = ServerConfig {
			hostname: TEST_HOSTNAME.to_string(),
			port: TEST_PORT,
			password: TEST_PASSWORD.to_string(),
		};
		let mut message_sender = None;
		let mut downloaded_first_empty_server_database = false;
		let mut uploaded_actual_testing_db_to_server = false;

		let actual_testing_db = generate_testing_db();

		loop {
			match receiver.next().await {
				Some(event) => match event {
					ServerWsEvent::MessageSender(new_message_sender) => {
						message_sender = Some(new_message_sender);
						message_sender
							.as_mut()
							.unwrap()
							.send(ServerWsMessage::Connect(test_server_config.clone()))
							.unwrap();
					}
					ServerWsEvent::Connected => {
						message_sender
							.as_mut()
							.unwrap()
							.send(ServerWsMessage::Request(Request::DownloadDatabase))
							.unwrap();
					}
					ServerWsEvent::Response { response, password } => {
						let encrypted_response_message =
							response.0.expect("client failed to receive response");
						let encrypted_response =
							EncryptedResponse::decrypt(encrypted_response_message, &password)
								.expect("client failed to read encrypted response");

						match encrypted_response {
							EncryptedResponse::Database {
								database,
								last_modified_time,
							} => {
								if !downloaded_first_empty_server_database {
									assert_eq!(database, Database::default().to_serialized());
									downloaded_first_empty_server_database = true;

									message_sender
										.as_mut()
										.unwrap()
										.send(ServerWsMessage::Request(Request::UpdateDatabase {
											database: actual_testing_db.clone().to_serialized(),
											last_modified_time: *actual_testing_db
												.last_changed_time(),
										}))
										.unwrap();
								} else if uploaded_actual_testing_db_to_server {
									assert_eq!(
										last_modified_time,
										*actual_testing_db.last_changed_time()
									);
									assert_eq!(database, actual_testing_db.to_serialized());
									// closes client listen thread
									message_sender.as_mut().unwrap().send(ServerWsMessage::CloseSubscription).unwrap();
									return;
								} else {
									panic!("should not have received the db a second time, should only be the first empty db!");
								}
							}
							EncryptedResponse::DatabaseUpdated => {
								if !downloaded_first_empty_server_database {
									panic!("should not have gotten db updated response before actually downloading the empty db first!");
								} else {
									uploaded_actual_testing_db_to_server = true;
									// download db from server a second time to confirm that the same generated db is sent back by server
									message_sender
										.as_mut()
										.unwrap()
										.send(ServerWsMessage::Request(Request::DownloadDatabase))
										.unwrap();
								}
							}
							EncryptedResponse::ModifiedDate(_modified_date) => panic!("client received server db modified date eventhough the client never requested it"),
						}
					}
					ServerWsEvent::Error(error_str) => panic!("client listen error: {error_str}"),
					ServerWsEvent::Disconnected => {
						panic!("clients connection to server disconnected")
					}
				},
				None => panic!(
					"client ws connection seems to have closed (server ws event receiver closed)!"
				),
			}
		}
	});
}

fn generate_testing_db() -> Database {
	let mut db = Database::default();

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

		db.modify(|projects| projects.insert(ProjectId::generate(), project));
	}

	db
}
