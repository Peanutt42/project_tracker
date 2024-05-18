use iced::{theme, widget::{button, svg, Button}};
use crate::{
	project_tracker::UiMessage,
	components::{CreateNewProjectModalMessage, CreateNewTaskModalMessage},
	styles::GreenCircleButtonStyle,
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
