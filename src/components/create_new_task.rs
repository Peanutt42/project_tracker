use iced::{theme, widget::{button, row, svg, text, Button, TextInput}, Length, alignment::{Horizontal, Vertical}};
use iced_aw::{CardStyles, Card};

use crate::{project_tracker::UiMessage, task::{Task, TaskState}};
use crate::components::{GreenRoundButtonStyle, GreenButtonStyle};

#[derive(Debug, Clone)]
pub enum CreateNewTaskModalMessage {
	Open,
	Close,
	ChangeTaskName(String),
}

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

	pub fn update(&mut self, message: CreateNewTaskModalMessage) {
		match message {
			CreateNewTaskModalMessage::Open => {
				self.opened = true;
				self.task_name.clear();
			},
			CreateNewTaskModalMessage::Close => {
				self.opened = false;
				self.task_name.clear();
			},
			CreateNewTaskModalMessage::ChangeTaskName(new_name) => self.task_name = new_name,
		}
	}

	pub fn view(&self, project_name: String, dark_mode: bool) -> Option<Card<UiMessage>> {
		if self.opened {
			let card_style = if dark_mode {
				CardStyles::Dark
			}
			else {
				CardStyles::Light
			};

			Some(Card::new(
				text("Create new task")
					.vertical_alignment(Vertical::Center),
				TextInput::new("task name", &self.task_name)
					.on_input(|new_name| UiMessage::CreateNewTaskModalMessage(CreateNewTaskModalMessage::ChangeTaskName(new_name)))
					.on_submit(UiMessage::CreateTask {
						project_name: project_name.clone(),
						task: Task::new(self.task_name.clone(), TaskState::Todo)
					})
			)
			.foot(
				row![
					button(
						text("Create")
							.horizontal_alignment(Horizontal::Center)
							.vertical_alignment(Vertical::Center)
					)
						.width(Length::Fill)
						.style(theme::Button::Custom(Box::new(GreenButtonStyle)))
						.on_press(UiMessage::CreateTask {
							project_name: project_name.clone(),
							task: Task::new(self.task_name.clone(), TaskState::Todo)
						}),

					button(
						text("Cancel")
							.horizontal_alignment(Horizontal::Center)
							.vertical_alignment(Vertical::Center)
					)
						.width(Length::Fill)
						.style(theme::Button::Secondary)
						.on_press(UiMessage::CreateNewTaskModalMessage(CreateNewTaskModalMessage::Close))
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

pub fn create_new_task_button() -> Button<'static, UiMessage> {
	let add_task_svg = svg::Handle::from_path(format!("{}/assets/add_task.svg", env!("CARGO_MANIFEST_DIR")));

	button(
		svg(add_task_svg)
			.width(32)
			.height(32)
	)
	.on_press(UiMessage::CreateNewTaskModalMessage(CreateNewTaskModalMessage::Open))
	.style(theme::Button::Custom(Box::new(GreenRoundButtonStyle)))
}
