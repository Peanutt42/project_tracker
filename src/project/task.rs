use iced::{widget::{text, row}, Element, Alignment};
use iced_aw::{badge, BadgeStyles};
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

	pub fn view(&self) -> Element<UiMessage> {
		match self {
			TaskState::Todo => badge(text("Todo").size(15))
				.style(BadgeStyles::Default)
				.into(),
			TaskState::InProgress => badge(text("In Progress").size(15))
				.style(BadgeStyles::Warning)
				.into(),
			TaskState::Done => badge(text("Done").size(15))
				.style(BadgeStyles::Success)
				.into(),
		}
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
		row![
			self.state.view(),
			text(&self.name)
		]
		.spacing(10)
		.align_items(Alignment::Center)
		.into()
	}
}
