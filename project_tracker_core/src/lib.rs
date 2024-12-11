#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

mod database;
pub use database::{Database, DatabaseMessage, LoadDatabaseResult, LoadDatabaseError, SaveDatabaseError, SyncDatabaseResult, get_last_modification_date_time};

mod ordered_hash_map;
pub use ordered_hash_map::OrderedHashMap;

mod project;
pub use project::{Project, ProjectId, SerializableColor, SortMode};

mod task;
pub use task::{Task, TaskId, TaskType, TimeSpend};

mod date;
pub use date::SerializableDate;

mod task_tag;
pub use task_tag::{TaskTag, TaskTagId};