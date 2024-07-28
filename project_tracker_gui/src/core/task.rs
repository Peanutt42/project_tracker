use iced::widget::container::Id;
use serde::{Serialize, Deserialize};
use crate::core::TaskState;

pub type TaskId = usize;

pub fn generate_task_id() -> TaskId {
	rand::random()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
	pub name: String,
	pub state: TaskState,

	#[serde(skip, default = "Id::unique")]
	pub dropzone_id: Id,
}

impl Task {
	pub fn new(name: String, state: TaskState) -> Self {
		Self {
			name,
			state,
			dropzone_id: Id::unique(),
		}
	}

	pub fn is_done(&self) -> bool {
		self.state.is_done()
	}

	pub fn is_todo(&self) -> bool {
		self.state.is_todo()
	}
}