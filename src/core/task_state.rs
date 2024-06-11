use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
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
