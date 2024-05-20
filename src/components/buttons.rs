use iced::{alignment::{Horizontal, Vertical}, theme, widget::{button, container, row, svg, text, Button}, Length, Padding};
use crate::{
	pages::{ProjectPageMessage, SidebarPageMessage}, project_tracker::UiMessage, styles::{BlackWhiteSvgStyle, GreenSvgStyle, ProjectPreviewButtonStyle, TransparentButtonStyle, ICON_SIZE, SMALL_SPACING_AMOUNT}
};

pub fn create_new_project_button() -> Button<'static, UiMessage> {
	let add_project_svg = svg::Handle::from_memory(include_bytes!("../../assets/add_project.svg"));

	button(
		row![
			svg(add_project_svg)
				.width(ICON_SIZE)
				.height(ICON_SIZE)
				.style(theme::Svg::Custom(Box::new(GreenSvgStyle))),

			text("New Project")
		]
		.align_items(iced::Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.on_press(SidebarPageMessage::OpenCreateNewProject.into())
	.style(theme::Button::custom(TransparentButtonStyle))
}

pub fn create_new_task_button() -> Button<'static, UiMessage> {
	let add_task_svg = svg::Handle::from_memory(include_bytes!("../../assets/add_task.svg"));

	button(
		container(svg(add_task_svg)
			.width(ICON_SIZE)
			.height(ICON_SIZE)
			.style(theme::Svg::Custom(Box::new(GreenSvgStyle))))
				.padding(Padding::new(2.5))
	)
	.on_press(ProjectPageMessage::OpenCreateNewTask.into())
	.style(theme::Button::custom(TransparentButtonStyle))
}

pub fn settings_button() -> Button<'static, UiMessage> {
	let settings_svg = svg::Handle::from_memory(include_bytes!("../../assets/settings.svg"));
	
	button(
		svg(settings_svg)
			.width(ICON_SIZE)
			.height(ICON_SIZE)
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