#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

mod database;
pub use database::{
	get_last_modification_date_time, Database, DatabaseMessage, LoadDatabaseError,
	LoadDatabaseResult, SaveDatabaseError, SerializedDatabase, SyncDatabaseResult,
};

mod ordered_hash_map;
pub use ordered_hash_map::OrderedHashMap;

mod project;
pub use project::{Project, ProjectId, SerializableColor, SortMode};

mod task;
pub use task::{
	duration_str, duration_to_minutes, parse_duration_from_str, round_duration_to_minutes,
	round_duration_to_seconds, Task, TaskId, TaskType, TimeSpend,
};

mod date;
pub use date::SerializableDate;

mod task_tag;
pub use task_tag::{TaskTag, TaskTagId};
