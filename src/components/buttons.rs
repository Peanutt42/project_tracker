use iced::{theme, Length, alignment::{Horizontal, Vertical}, widget::{button, svg, text, Button}};
use crate::{
	project_tracker::UiMessage,
	components::{CreateNewProjectModalMessage, CreateNewTaskModalMessage},
	styles::{GreenCircleButtonStyle, ProjectPreviewButtonStyle},
};

pub fn create_new_project_button() -> Button<'static, UiMessage> {
	let add_project_svg = svg::Handle::from_memory(include_bytes!("../../assets/add_project.svg"));

	button(
		svg(add_project_svg)
			.width(28)
			.height(28)
	)
	.on_press(CreateNewProjectModalMessage::Open.into())
	.style(theme::Button::custom(GreenCircleButtonStyle))
}

pub fn create_new_task_button() -> Button<'static, UiMessage> {
	let add_task_svg = svg::Handle::from_memory(include_bytes!("../../assets/add_task.svg"));

	button(
		svg(add_task_svg)
			.width(28)
			.height(28)
	)
	.on_press(CreateNewTaskModalMessage::Open.into())
	.style(theme::Button::custom(GreenCircleButtonStyle))
}

pub fn overview_button(selected: bool) -> Button<'static, UiMessage> {
	button(
		text("Todo Overview")
			.size(25)
			.width(Length::Fill)
			.horizontal_alignment(Horizontal::Center)
			.vertical_alignment(Vertical::Center)
	)
	.width(Length::Fill)
	.on_press(UiMessage::OpenOverview)
	.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }))
}