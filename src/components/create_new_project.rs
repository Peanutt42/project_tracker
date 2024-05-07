use iced::{theme, alignment::Horizontal, widget::{button, row, text, svg, Button, Text, TextInput}, Length};
use iced_aw::{Card, CardStyles};
use crate::project_tracker::UiMessage;
use crate::components::{GreenRoundButtonStyle, GreenButtonStyle};

#[derive(Debug, Clone)]
pub enum CreateNewProjectModalMessage {
	Open,
	Close,
	ChangeProjectName(String),
}

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

	pub fn update(&mut self, message: CreateNewProjectModalMessage) {
		match message {
			CreateNewProjectModalMessage::Open => {
				self.opened = true;
				self.project_name.clear();
			},
			CreateNewProjectModalMessage::Close => {
				self.opened = false;
				self.project_name.clear();
			},
			CreateNewProjectModalMessage::ChangeProjectName(new_name) => self.project_name = new_name,
		}
	}

	pub fn view(&self, dark_mode: bool) -> Option<Card<UiMessage>> {
		if self.opened {
			let card_style = if dark_mode {
				CardStyles::Dark
			}
			else {
				CardStyles::Light
			};

			Some(Card::new(
				Text::new("Create Project"),
				TextInput::new("Project name", &self.project_name)
        			.on_input(|new_name| UiMessage::CreateNewProjectModalMessage(CreateNewProjectModalMessage::ChangeProjectName(new_name)))
					.on_submit(UiMessage::CreateProject(self.project_name.clone()))
			)
			.foot(
				row![
					button(
						text("Create")
							.horizontal_alignment(Horizontal::Center)
					)
					.width(Length::Fill)
					.style(theme::Button::Custom(Box::new(GreenButtonStyle)))
					.on_press(UiMessage::CreateProject(self.project_name.clone())),

					button(
						text("Cancel")
							.horizontal_alignment(Horizontal::Center)
					)
					.width(Length::Fill)
					.style(theme::Button::Secondary)
					.on_press(UiMessage::CreateNewProjectModalMessage(CreateNewProjectModalMessage::Close)),
				]
			)
			.style(card_style)
			.max_width(400.0))
		}
		else {
			None
		}
	}
}


pub fn create_new_project_button() -> Button<'static, UiMessage> {
	let add_project_svg = svg::Handle::from_path(format!("{}/assets/add_project.svg", env!("CARGO_MANIFEST_DIR")));

	button(
		svg(add_project_svg)
			.width(32)
			.height(32)
	)
	.on_press(UiMessage::CreateNewProjectModalMessage(CreateNewProjectModalMessage::Open))
	.style(iced::theme::Button::Custom(Box::new(GreenRoundButtonStyle)))
}
