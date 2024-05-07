use iced::{alignment::Horizontal, widget::{button, row, text, svg, Button, Text, TextInput}, Background, Border, Color, Length, Shadow, Theme, Vector};
use iced_aw::Card;
use crate::project_tracker::UiMessage;

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

	pub fn view(&self) -> Option<Card<UiMessage>> {
		if self.opened {
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
					.on_press(UiMessage::CreateProject(self.project_name.clone())),

					button(
						text("Cancel")
							.horizontal_alignment(Horizontal::Center)
					)
					.width(Length::Fill)
					.on_press(UiMessage::CreateNewProjectModalMessage(CreateNewProjectModalMessage::Close)),
				]
			)
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
	.style(iced::theme::Button::Custom(Box::new(CreateNewProjectButtonStyle)))
}

struct CreateNewProjectButtonStyle;

impl button::StyleSheet for CreateNewProjectButtonStyle {
	type Style = Theme;

	fn active(&self, _style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 0.75, 0.0))),
			border: Border::with_radius(32.0),
			shadow: Shadow {
				color: Color::BLACK,
				offset: Vector::default(),
				blur_radius: 20.0,
			},
			..Default::default()
		}
	}

	fn hovered(&self, _style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 1.0, 0.0))),
			border: Border::with_radius(32.0),
			shadow: Shadow {
				color: Color::BLACK,
				offset: Vector::default(),
				blur_radius: 30.0,
			},
			..Default::default()
		}
	}

	fn pressed(&self, _style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 0.5, 0.0))),
			border: Border::with_radius(32.0),
			shadow: Shadow {
				color: Color::BLACK,
				offset: Vector::default(),
				blur_radius: 40.0,
			},
			..Default::default()
		}
	}
}
/*
struct RoundCreateNewProjectButtonStyle;

impl button::StyleSheet for RoundCreateNewProjectButtonStyle {

}
*/
