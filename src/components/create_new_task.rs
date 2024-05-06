use iced::{theme, widget::{button, row, svg, text, Button, TextInput}, Background, Border, Color, Shadow, Theme, Vector};
use iced_aw::Card;

use crate::{project_tracker::UiMessage, task::{Task, TaskState}};

#[derive(Debug, Clone)]
pub struct CreateNewTaskModal {
	opened: bool,
	pub task_name: String,
}

impl CreateNewTaskModal {
	pub fn new() -> Self {
		Self {
			opened: false,
			task_name: String::new(),
		}
	}

	pub fn view(&self, project_name: String) -> Option<Card<UiMessage>> {
		if self.opened {
			Some(Card::new(
				text("Create new task"),
				TextInput::new("task name", &self.task_name)
					.on_input(UiMessage::ChangeCreateNewTaskName)
					.on_submit(UiMessage::CreateTask {
						project_name: project_name.clone(),
						task: Task::new(self.task_name.clone(), TaskState::Todo)
					})
			)
			.foot(
				row![
					button(text("Create"))
						.on_press(UiMessage::CreateTask {
							project_name: project_name.clone(),
							task: Task::new(self.task_name.clone(), TaskState::Todo)
						}),
					button(text("Cancel"))
						.on_press(UiMessage::CloseCreateNewTaskModal)
				]
			)
			.max_width(400.0))
		}
		else {
			None
		}
	}

	pub fn open(&mut self) {
		self.opened = true;
		self.task_name.clear();
	}

	pub fn close(&mut self) {
		self.opened = false;
		self.task_name.clear();
	}
}

pub fn create_new_task_button() -> Button<'static, UiMessage> {
	let add_task_svg = svg::Handle::from_path(format!("{}/assets/add_task.svg", env!("CARGO_MANIFEST_DIR")));

	button(
		svg(add_task_svg)
			.width(32)
			.height(32)
	)
	.on_press(UiMessage::OpenCreateNewTaskModal)
	.style(theme::Button::Custom(Box::new(CreateNewTaskButtonStyle)))
}

struct CreateNewTaskButtonStyle;

impl button::StyleSheet for CreateNewTaskButtonStyle {
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
