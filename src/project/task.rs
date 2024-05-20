use iced::{widget::{row, checkbox, text}, Color, theme, Element};
use serde::{Serialize, Deserialize};
use crate::project_tracker::UiMessage;
use crate::styles::GreenCheckboxStyle;
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
						theme::Text::Color(Color::from_rgb(0.5, 0.5, 0.5))
					}
					else {
						theme::Text::Default
					}
				),
		]
		.align_items(iced::Alignment::Start)
		.into()
	}
}
