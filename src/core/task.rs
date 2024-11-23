use crate::core::{TaskTagId, SerializableDate};
use iced::{widget::{container::Id, markdown}, advanced::widget};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, time::{Duration, Instant}};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub usize);

impl TaskId {
	pub fn generate() -> Self {
		Self(rand::random())
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TaskType {
	Todo,
	Done,
	SourceCodeTodo,
}

impl TaskType {
	pub fn is_done(&self) -> bool {
		matches!(self, Self::Done)
	}
}

#[derive(Clone, Debug, Serialize)]
pub struct TimeSpend {
	offset_seconds: f32, // seconds spend before stopping the time
	#[serde(skip)]
	tracking_time_start: Option<Instant>,
}

impl TimeSpend {
	pub fn new(seconds: f32) -> Self {
		Self {
			offset_seconds: seconds,
			tracking_time_start: None,
		}
	}

	pub fn get_duration(&self) -> Duration {
		match &self.tracking_time_start {
			Some(tracking_time_start) => Duration::from_secs_f32(self.offset_seconds) + Instant::now().duration_since(*tracking_time_start),
			None => Duration::from_secs_f32(self.offset_seconds)
		}
	}

	pub fn get_seconds(&self) -> f32 {
		match &self.tracking_time_start {
			Some(tracking_time_start) => self.offset_seconds + Instant::now().duration_since(*tracking_time_start).as_secs_f32(),
			None => self.offset_seconds
		}
	}

	pub fn start(&mut self) {
		self.stop();

		self.tracking_time_start = Some(Instant::now());
	}

	pub fn stop(&mut self) {
		if let Some(tracking_time_start) = &self.tracking_time_start {
			self.offset_seconds += Instant::now().duration_since(*tracking_time_start).as_secs_f32();
			self.tracking_time_start = None;
		}
	}
}

#[derive(Clone, Debug)]
pub struct Task {
	name: String,
	description: String,
	pub needed_time_minutes: Option<usize>,
	pub time_spend: Option<TimeSpend>,
	pub due_date: Option<SerializableDate>,
	pub tags: HashSet<TaskTagId>,

	pub dropzone_id: Id,
	pub droppable_id: widget::Id,
	description_markdown_items: Vec<markdown::Item>,
}

impl Task {
	pub fn new(name: String, description: String, needed_time_minutes: Option<usize>, time_spend: Option<TimeSpend>, due_date: Option<SerializableDate>, tags: HashSet<TaskTagId>) -> Self {
		let description_markdown_items = markdown::parse(&description).collect();

		Self {
			name,
			description,
			needed_time_minutes,
			time_spend,
			due_date,
			tags,
			dropzone_id: Id::unique(),
			droppable_id: widget::Id::unique(),
			description_markdown_items,
		}
	}

	pub fn name(&self) -> &String { &self.name }
	pub fn description(&self) -> &String { &self.description }
	pub fn description_markdown_items(&self) -> &Vec<markdown::Item> { &self.description_markdown_items }

	pub fn set_name(&mut self, new_name: String) {
		self.name = new_name;
	}

	pub fn set_description(&mut self, new_description: String) {
		self.description = new_description;
		self.description_markdown_items = markdown::parse(&self.description).collect();
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


#[derive(Serialize, Deserialize)]
struct SerializedTask {
	pub name: String,
	pub description: String,
	#[serde(default)]
	pub needed_time_minutes: Option<usize>,
	#[serde(default)]
	pub time_spend_seconds: Option<f32>,
	#[serde(default)]
	pub due_date: Option<SerializableDate>,
	pub tags: HashSet<TaskTagId>,
}

impl Serialize for Task {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
		let serialized_task = SerializedTask {
			name: self.name.clone(),
			description: self.description.clone(),
			needed_time_minutes: self.needed_time_minutes,
			time_spend_seconds: self.time_spend.as_ref().map(|time_spend| time_spend.get_seconds()),
			due_date: self.due_date,
			tags: self.tags.clone(),
		};
		serialized_task.serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for Task {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>
	{
		let serialized_task = SerializedTask::deserialize(deserializer)?;
		Ok(Task::new(
			serialized_task.name,
			serialized_task.description,
			serialized_task.needed_time_minutes,
			serialized_task.time_spend_seconds.map(TimeSpend::new),
			serialized_task.due_date,
			serialized_task.tags
		))
	}
}