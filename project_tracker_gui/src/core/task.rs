use std::collections::BTreeSet;
use iced::widget::container::Id;
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
	pub tags: BTreeSet<TaskTagId>,

	#[serde(skip, default = "Id::unique")]
	pub dropzone_id: Id,
}

impl Task {
	pub fn new(name: String, state: TaskState) -> Self {
		Self {
			name,
			state,
			tags: BTreeSet::new(),
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