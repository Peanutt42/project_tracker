use iced::{widget::{row, checkbox, text}, Alignment, theme, Element};
use serde::{Serialize, Deserialize};
use crate::project_tracker::UiMessage;
use crate::styles::{GreenCheckboxStyle, GREY};
use crate::project::TaskState;

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

	pub fn view<'a>(&'a self, project_name: &'a str) -> Element<UiMessage> {
		row![
			checkbox("", self.state.is_done())
			.on_toggle(|checked| {
				UiMessage::SetTaskState {
					project_name: project_name.to_string(),
					task_name: self.name.clone(),
					task_state:
						if checked {
							TaskState::Done
						}
						else {
							TaskState::Todo
						},
				}
			})
			.style(theme::Checkbox::Custom(Box::new(GreenCheckboxStyle))),

			text(&self.name)
				.style(
					if self.is_done() {
						theme::Text::Color(GREY)
					}
					else {
						theme::Text::Default
					}
				),
		]
		.align_items(Alignment::Start)
		.into()
	}
}
