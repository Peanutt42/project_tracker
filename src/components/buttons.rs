use iced::{theme, Length, alignment::{Horizontal, Vertical}, widget::{button, row, svg, text, Button}};
use crate::{
	components::{CreateNewProjectModalMessage, CreateNewTaskModalMessage}, project_tracker::UiMessage,
	styles::{GreenSvgStyle, ProjectPreviewButtonStyle, TransparentButtonStyle, BlackWhiteSvgStyle}
};

pub fn create_new_project_button() -> Button<'static, UiMessage> {
	let add_project_svg = svg::Handle::from_memory(include_bytes!("../../assets/add_project.svg"));

	button(
		row![
			svg(add_project_svg)
				.width(32)
				.height(32)
				.style(theme::Svg::Custom(Box::new(GreenSvgStyle))),

			text("New Project")
		]
		.align_items(iced::Alignment::Center)
		.spacing(5)
	)
	.on_press(CreateNewProjectModalMessage::Open.into())
	.style(theme::Button::custom(TransparentButtonStyle))
}

pub fn create_new_task_button() -> Button<'static, UiMessage> {
	let add_task_svg = svg::Handle::from_memory(include_bytes!("../../assets/add_task.svg"));

	button(
		svg(add_task_svg)
			.width(32)
			.height(32)
			.style(theme::Svg::Custom(Box::new(GreenSvgStyle)))
	)
	.on_press(CreateNewTaskModalMessage::Open.into())
	.style(theme::Button::custom(TransparentButtonStyle))
}

pub fn settings_button() -> Button<'static, UiMessage> {
	let settings_svg = svg::Handle::from_memory(include_bytes!("../../assets/settings.svg"));
	
	button(
		svg(settings_svg)
			.width(32)
			.height(32)
			.style(theme::Svg::Custom(Box::new(BlackWhiteSvgStyle)))
	)
	.on_press(UiMessage::OpenSettings)
	.style(theme::Button::Secondary)
}

pub fn overview_button(selected: bool) -> Button<'static, UiMessage> {
	button(
		text("Overview")
			.size(22)
			.width(Length::Fill)
			.horizontal_alignment(Horizontal::Center)
			.vertical_alignment(Vertical::Center)
	)
	.width(Length::Fill)
	.on_press(UiMessage::OpenOverview)
	.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }))
}