use std::collections::BTreeSet;
use iced::widget::container::Id;
use iced_aw::date_picker::Date;
use serde::{Serialize, Deserialize};
use crate::core::{TaskState, TaskTagId};

pub type TaskId = usize;

pub fn generate_task_id() -> TaskId {
	rand::random()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
	pub name: String,
	pub state: TaskState,
	pub needed_time_minutes: Option<usize>,
	pub due_date: Option<SerializableDate>,
	pub tags: BTreeSet<TaskTagId>,

	#[serde(skip, default = "Id::unique")]
	pub dropzone_id: Id,
}

impl Task {
	pub fn new(name: String, state: TaskState, tags: BTreeSet<TaskTagId>) -> Self {
		Self {
			name,
			state,
			needed_time_minutes: None,
			due_date: None,
			tags,
			dropzone_id: Id::unique(),
		}
	}

	pub fn is_done(&self) -> bool {
		self.state.is_done()
	}

	pub fn is_todo(&self) -> bool {
		self.state.is_todo()
	}

	pub fn matches_filter(&self, filter: &BTreeSet<TaskTagId>) -> bool {
		for tag_id in filter.iter() {
			if !self.tags.contains(tag_id) {
				return false;
			}
		}
		true
	}
}


#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SerializableDate {
	pub year: i32,
	pub month: u32,
	pub day: u32,
}

impl From<SerializableDate> for Date {
	fn from(value: SerializableDate) -> Self {
		Self::from_ymd(value.year, value.month, value.day)
	}
}

impl From<Date> for SerializableDate {
	fn from(value: Date) -> Self {
		Self {
			year: value.year,
			month: value.month,
			day: value.day,
		}
	}
}