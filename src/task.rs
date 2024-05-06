use iced::{widget::text, Element};
use serde::{Serialize, Deserialize};
use crate::project_tracker::UiMessage;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub enum TaskState {
	#[default]
	Todo,
	InProgress,
	Done,
}

impl TaskState {
	pub fn is_done(&self) -> bool {
		*self == TaskState::Done
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
	pub name: String,
	pub state: TaskState,
}

impl Task {
	pub fn new(name: String, state: TaskState) -> Self {
		Self {
			name,
			state,
		}
	}

	pub fn is_done(&self) -> bool {
		self.state.is_done()
	}

	pub fn view(&self) -> Element<UiMessage> {
		text(format!("({:?}) {}", self.state, self.name)).into()
	}
}
