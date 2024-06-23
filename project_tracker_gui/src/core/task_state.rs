use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
	#[default]
	Todo,
	Done,
}

impl TaskState {
	pub fn is_done(&self) -> bool {
		*self == TaskState::Done
	}

	pub fn is_todo(&self) -> bool {
		*self == TaskState::Todo
	}
}
