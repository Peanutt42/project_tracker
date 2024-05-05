use iced::{widget::text, Element};
use crate::project_tracker::UiMessage;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
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
