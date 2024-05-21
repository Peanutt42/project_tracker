use iced::{Alignment, theme, widget::{button, row, svg, text, Button}, Length};
use crate::{
	pages::{ProjectPageMessage, SidebarPageMessage}, project_tracker::UiMessage, styles::{BlackWhiteSvgStyle, GreenButtonStyle, GreenSvgStyle, ProjectPreviewButtonStyle, TransparentButtonStyle, ICON_SIZE, LARGE_ICON_SIZE, LARGE_TEXT_SIZE, SMALL_SPACING_AMOUNT, SPACING_AMOUNT}
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
		.align_items(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.on_press(SidebarPageMessage::OpenCreateNewProject.into())
	.style(theme::Button::custom(TransparentButtonStyle))
}

pub fn create_new_task_button() -> Button<'static, UiMessage> {
	let add_task_svg = svg::Handle::from_memory(include_bytes!("../../assets/add_task.svg"));

	button(
		row![
			svg(add_task_svg)
			.width(ICON_SIZE)
			.height(ICON_SIZE)
			.style(theme::Svg::Custom(Box::new(GreenSvgStyle))),

			text("New Task")
		]
		.align_items(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.on_press(ProjectPageMessage::OpenCreateNewTask.into())
	.style(theme::Button::custom(TransparentButtonStyle))
}

pub fn cancel_button() -> Button<'static, UiMessage> {
	let cancel_svg = svg::Handle::from_memory(include_bytes!("../../assets/cancel.svg"));

	button(
		svg(cancel_svg)
			.width(ICON_SIZE)
			.height(ICON_SIZE)
			.style(theme::Svg::Custom(Box::new(BlackWhiteSvgStyle)))
	)
	.style(theme::Button::custom(GreenButtonStyle))
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
	let overview_svg = svg::Handle::from_memory(include_bytes!("../../assets/overview.svg"));

	button(
		row![
			svg(overview_svg)
				.width(LARGE_ICON_SIZE)
				.width(LARGE_ICON_SIZE)
				.style(theme::Svg::Custom(Box::new(BlackWhiteSvgStyle))),

			text("Overview")
				.size(LARGE_TEXT_SIZE)
				.width(Length::Fill)
		]
		.align_items(Alignment::Center)
		.spacing(SPACING_AMOUNT)
		.width(Length::Fill)
	)
	.width(Length::Fill)
	.on_press(UiMessage::OpenOverview)
	.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }))
}