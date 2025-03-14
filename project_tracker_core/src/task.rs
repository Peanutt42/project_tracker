use crate::{SerializableDate, TaskTagId};
use humantime::format_duration;
use serde::{Deserialize, Serialize};
use std::{
	collections::{BTreeSet, HashSet},
	hash::Hash,
	time::{Duration, Instant},
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
	pub fn generate() -> Self {
		Self(Uuid::new_v4())
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

/// Note: hash implementation of 'TimeSpend' ignores 'tracking_time_start' and only uses u64 resolution of 'offset_seconds'
/// this is needed for server synchronization!
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeSpend {
	/// seconds spend before stopping the time
	offset_seconds: f32,
	/// only used for tracking time since tracking start by client, ignored by server
	#[serde(skip)]
	tracking_time_start: Option<Instant>,
}

impl PartialEq for TimeSpend {
	fn eq(&self, other: &Self) -> bool {
		(self.offset_seconds as u64) == (other.offset_seconds as u64)
	}
}
impl Eq for TimeSpend {}
impl Hash for TimeSpend {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		(self.offset_seconds as u64).hash(state);
	}
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
			Some(tracking_time_start) => {
				Duration::from_secs_f32(self.offset_seconds)
					+ Instant::now().duration_since(*tracking_time_start)
			}
			None => Duration::from_secs_f32(self.offset_seconds),
		}
	}

	pub fn get_seconds(&self) -> f32 {
		match &self.tracking_time_start {
			Some(tracking_time_start) => {
				self.offset_seconds
					+ Instant::now()
						.duration_since(*tracking_time_start)
						.as_secs_f32()
			}
			None => self.offset_seconds,
		}
	}

	pub fn start(&mut self) {
		self.stop();

		self.tracking_time_start = Some(Instant::now());
	}

	pub fn stop(&mut self) {
		if let Some(tracking_time_start) = &self.tracking_time_start {
			self.offset_seconds += Instant::now()
				.duration_since(*tracking_time_start)
				.as_secs_f32();
			self.tracking_time_start = None;
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Task {
	pub name: String,
	pub description: String,
	#[serde(default)]
	pub needed_time_minutes: Option<usize>,
	#[serde(default)]
	pub time_spend: Option<TimeSpend>,
	#[serde(default)]
	pub due_date: Option<SerializableDate>,
	pub tags: BTreeSet<TaskTagId>,
}

impl Task {
	pub fn new(
		name: String,
		description: String,
		needed_time_minutes: Option<usize>,
		time_spend: Option<TimeSpend>,
		due_date: Option<SerializableDate>,
		tags: BTreeSet<TaskTagId>,
	) -> Self {
		Self {
			name,
			description,
			needed_time_minutes,
			time_spend,
			due_date,
			tags,
		}
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

pub fn round_duration_to_seconds(duration: Duration) -> Duration {
	Duration::from_secs(duration.as_secs())
}

pub fn round_duration_to_minutes(duration: Duration) -> Duration {
	Duration::from_secs(60 * (duration.as_secs() / 60))
}

pub fn duration_to_minutes(duration: Duration) -> usize {
	duration.as_secs() as usize / 60
}

pub fn parse_duration_from_str(string: &str) -> Option<Duration> {
	humantime::parse_duration(string).ok()
}

pub fn duration_str(duration: Duration) -> String {
	format_duration(duration).to_string()
}
