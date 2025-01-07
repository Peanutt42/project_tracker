use indexmap::IndexMap;
use project_tracker::{
	Database, OrderedHashMap, Project, ProjectId, SerializableColor, SerializableDate, SortMode,
	Task, TaskId, TaskTag, TaskTagId, TimeSpend,
};
use serde::{Deserialize, Serialize};
use std::{
	collections::{HashMap, HashSet},
	path::PathBuf,
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
	let mut args = std::env::args();

	let Some(filepath_str) = args.nth(1) else {
		eprintln!("usage: migrate_db_keys_to_uuid /path/to/db.json");
		return;
	};

	let filepath = PathBuf::from(&filepath_str);
	let old_db: OldU64KeySerializedDatabase = serde_json::from_str(
		&tokio::fs::read_to_string(&filepath)
			.await
			.expect("failed to read old json database file"),
	)
	.expect("failed to load old json database");
	let mut migrated_db_filepath = filepath;
	migrated_db_filepath.set_file_name("migrated_database.json");

	let mut u64_project_id_uuid_map = HashMap::new();
	let mut u64_task_id_uuid_map = HashMap::new();
	let mut u64_task_tag_id_uuid_map = HashMap::new();

	let mut new_db = Database::default();
	for (project_id, old_project) in old_db.projects.iter() {
		let project_uuid = u64_project_id_uuid_map
			.entry(project_id)
			.or_insert(Uuid::new_v4());

		let mut new_task_tags = OrderedHashMap::<TaskTagId, TaskTag>::new();

		for (old_task_tag_id, task_tag) in old_project.task_tags.iter() {
			let task_tag_uuid = u64_task_tag_id_uuid_map
				.entry(old_task_tag_id)
				.or_insert(Uuid::new_v4());
			new_task_tags.insert(TaskTagId(*task_tag_uuid), task_tag.clone());
		}

		let mut new_todo_tasks = OrderedHashMap::<TaskId, Task>::new();
		let mut new_done_tasks = IndexMap::<TaskId, Task>::new();
		let mut new_source_code_tasks = IndexMap::<TaskId, Task>::new();

		let mut migrate_old_task =
			|old_task_id: OldU64KeyTaskId, old_task: &OldU64KeyTask| -> (TaskId, Task) {
				let task_uuid = u64_task_id_uuid_map
					.entry(old_task_id)
					.or_insert(Uuid::new_v4());

				let mut new_tags = HashSet::<TaskTagId>::new();
				for old_task_tag_id in old_task.tags.iter() {
					let task_tag_uuid = u64_task_tag_id_uuid_map
						.entry(*old_task_tag_id)
						.or_insert(Uuid::new_v4());
					new_tags.insert(TaskTagId(*task_tag_uuid));
				}

				let new_task = Task {
					name: old_task.name.clone(),
					description: old_task.description.clone(),
					needed_time_minutes: old_task.needed_time_minutes,
					time_spend: old_task.time_spend.clone(),
					due_date: old_task.due_date,
					tags: new_tags,
				};

				(TaskId(*task_uuid), new_task)
			};

		for (old_task_id, old_task) in old_project.todo_tasks.iter() {
			let (task_uuid, new_task) = migrate_old_task(old_task_id, old_task);
			new_todo_tasks.insert(task_uuid, new_task);
		}
		for (old_task_id, old_task) in old_project.done_tasks.iter() {
			let (task_uuid, new_task) = migrate_old_task(*old_task_id, old_task);
			new_done_tasks.insert(task_uuid, new_task);
		}
		for (old_task_id, old_task) in old_project.source_code_todos.iter() {
			let (task_uuid, new_task) = migrate_old_task(*old_task_id, old_task);
			new_source_code_tasks.insert(task_uuid, new_task);
		}

		let new_project = Project {
			name: old_project.name.clone(),
			color: old_project.color,
			sort_mode: old_project.sort_mode,
			task_tags: new_task_tags,
			todo_tasks: new_todo_tasks,
			done_tasks: new_done_tasks,
			source_code_todos: new_source_code_tasks,
			source_code_directory: old_project.source_code_directory.clone(),
		};

		new_db.modify(|projects| {
			projects.insert(ProjectId(*project_uuid), new_project);
		});
	}

	Database::export_as_json(
		migrated_db_filepath.clone(),
		new_db
			.to_json()
			.expect("failed to serialize migrated db as json"),
	)
	.await
	.expect("failed to export migrated db as json and save to file");

	println!(
		"successfully migrated json db to uuid, new db is in '{}'",
		migrated_db_filepath.display()
	);
}

pub type OldU64KeyProjectId = u64;
pub type OldU64KeyTaskId = u64;
pub type OldU64KeyTaskTagId = u64;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OldU64KeySerializedDatabase {
	projects: OrderedHashMap<OldU64KeyProjectId, OldU64KeyProject>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OldU64KeyProject {
	pub name: String,
	pub color: SerializableColor,
	pub sort_mode: SortMode,
	pub task_tags: OrderedHashMap<OldU64KeyTaskTagId, TaskTag>,
	pub todo_tasks: OrderedHashMap<OldU64KeyTaskId, OldU64KeyTask>,
	#[serde(with = "indexmap::map::serde_seq")]
	pub done_tasks: IndexMap<OldU64KeyTaskId, OldU64KeyTask>,
	#[serde(with = "indexmap::map::serde_seq")]
	pub source_code_todos: IndexMap<OldU64KeyTaskId, OldU64KeyTask>,
	#[serde(default)]
	pub source_code_directory: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OldU64KeyTask {
	name: String,
	description: String,
	#[serde(default)]
	pub needed_time_minutes: Option<usize>,
	#[serde(default)]
	pub time_spend: Option<TimeSpend>,
	#[serde(default)]
	pub due_date: Option<SerializableDate>,
	pub tags: HashSet<OldU64KeyTaskTagId>,
}
