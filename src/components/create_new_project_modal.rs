use iced::{Length, widget::{row, Button, Text, TextInput}, alignment::Horizontal};
use iced_aw::Card;
use crate::project_tracker::UiMessage;

#[derive(Clone, Debug)]
pub struct CreateNewProjectModal {
	opened: bool,
	pub project_name: String,
}

impl CreateNewProjectModal {
	pub fn new() -> Self {
		Self {
			opened: false,
			project_name: String::new(),
		}
	}

	pub fn view(&self) -> Option<Card<UiMessage>> {
		if self.opened {
			Some(Card::new(
				Text::new("Create Project"),
				TextInput::new("Project name", &self.project_name)
					.on_input(|new_project_name| UiMessage::ChangeCreateNewProjectName(new_project_name))
			)
			.foot(
				row![
					Button::new(
						Text::new("Create")
							.horizontal_alignment(Horizontal::Center)
					)
					.width(Length::Fill)
					.on_press(UiMessage::CreateProject(self.project_name.clone())),

					Button::new(
						Text::new("Cancel")
							.horizontal_alignment(Horizontal::Center)
					)
					.width(Length::Fill)
					.on_press(UiMessage::CloseCreateNewProjectModal),
				]
			)
			.max_width(400.0)
			)
		}
		else {
			None
		}
	}

	pub fn open(&mut self) {
		self.opened = true;
		self.project_name.clear();
	}

	pub fn close(&mut self) {
		self.opened = false;
		self.project_name.clear();
	}
}
