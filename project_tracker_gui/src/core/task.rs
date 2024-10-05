use crate::core::TaskTagId;
use iced::{widget::{container::Id, markdown}, advanced::widget};
use iced_aw::date_picker::Date;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

#[derive(Clone, Debug, Serialize)]
pub struct Task {
	name: String,
	pub needed_time_minutes: Option<usize>,
	pub due_date: Option<SerializableDate>,
	pub tags: HashSet<TaskTagId>,

	#[serde(skip_serializing, default = "Id::unique")]
	pub dropzone_id: Id,

	#[serde(skip_serializing, default = "widget::Id::unique")]
	pub droppable_id: widget::Id,

	#[serde(skip_serializing)]
	markdown_items: Vec<markdown::Item>,
}

impl Task {
	pub fn new(name: String, needed_time_minutes: Option<usize>, due_date: Option<SerializableDate>, tags: HashSet<TaskTagId>) -> Self {
		let markdown_items = markdown::parse(&name).collect();

		Self {
			name,
			needed_time_minutes,
			due_date,
			tags,
			dropzone_id: Id::unique(),
			droppable_id: widget::Id::unique(),
			markdown_items,
		}
	}

	pub fn name(&self) -> &String { &self.name }
	pub fn markdown_items(&self) -> &Vec<markdown::Item> { &self.markdown_items }

	pub fn set_name(&mut self, new_name: String) {
		self.name = new_name;
		self.markdown_items = markdown::parse(&self.name).collect();
	}

	pub fn has_same_content_as(&self, other: &Task) -> bool {
		self.name == other.name
			&& self.needed_time_minutes == other.needed_time_minutes
			&& self.due_date == other.due_date
			&& self.tags == other.tags
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

impl PartialEq for Task {
	fn eq(&self, other: &Self) -> bool {
		self.name.eq(&other.name) &&
		self.needed_time_minutes.eq(&other.needed_time_minutes) &&
		self.due_date.eq(&other.due_date) &&
		self.tags.eq(&other.tags)
	}
}
impl Eq for Task {}

impl<'de> Deserialize<'de> for Task {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>
	{
		#[derive(Deserialize)]
		struct SerializedTask {
			pub name: String,
			pub needed_time_minutes: Option<usize>,
			pub due_date: Option<SerializableDate>,
			pub tags: HashSet<TaskTagId>,
		}

		let serialized_task = SerializedTask::deserialize(deserializer)?;
		Ok(Task::new(
			serialized_task.name,
			serialized_task.needed_time_minutes,
			serialized_task.due_date,
			serialized_task.tags
		))
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
