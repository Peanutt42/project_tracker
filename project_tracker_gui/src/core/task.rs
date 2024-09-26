use std::collections::HashSet;
use iced::widget::container::Id;
use iced_aw::date_picker::Date;
use serde::{Serialize, Deserialize};
use crate::core::TaskTagId;

pub type TaskId = usize;

pub fn generate_task_id() -> TaskId {
	rand::random()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TaskType {
	Todo,
	Done,
	SourceCodeTodo,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
	pub name: String,
	pub needed_time_minutes: Option<usize>,
	pub due_date: Option<SerializableDate>,
	pub tags: HashSet<TaskTagId>,

	#[serde(skip, default = "Id::unique")]
	pub dropzone_id: Id,
}

impl Task {
	pub fn new(name: String, tags: HashSet<TaskTagId>) -> Self {
		Self {
			name,
			needed_time_minutes: None,
			due_date: None,
			tags,
			dropzone_id: Id::unique(),
		}
	}

	pub fn has_same_content_as(&self, other: &Task) -> bool {
		self.name == other.name &&
		self.needed_time_minutes == other.needed_time_minutes &&
		self.due_date == other.due_date &&
		self.tags == other.tags
	}

	pub fn matches_filter(&self, filter: &HashSet<TaskTagId>) -> bool {
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