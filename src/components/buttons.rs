use iced::{theme, widget::{button, container, row, text, Button}, Alignment, Element, Length, Padding};
use iced_aw::core::icons::bootstrap::{icon_to_text, Bootstrap};
use crate::{
	pages::{ProjectPageMessage, SidebarPageMessage}, core::ProjectId, project_tracker::UiMessage, styles::{GreenButtonStyle, ProjectPreviewButtonStyle, SecondaryButtonStyle, TransparentButtonStyle, GREEN_TEXT_STYLE, LARGE_TEXT_SIZE, SMALL_SPACING_AMOUNT, SPACING_AMOUNT}
};

pub fn create_new_project_button() -> Button<'static, UiMessage> {
	button(
		container(
			icon_to_text(Bootstrap::PlusSquareFill)
				.size(LARGE_TEXT_SIZE * 1.7)
				.style(GREEN_TEXT_STYLE)
		)
		.width(Length::Fill)
		.height(Length::Fill)
		.center_x()
		.center_y()
	)
	.width(LARGE_TEXT_SIZE * 2.75)
	.height(LARGE_TEXT_SIZE * 2.75)
	.on_press(SidebarPageMessage::OpenCreateNewProject.into())
	.style(theme::Button::custom(TransparentButtonStyle))
}

pub fn create_new_task_button() -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::PlusCircle)
				.size(LARGE_TEXT_SIZE)
				.style(GREEN_TEXT_STYLE),

			text("New Task")
		]
		.align_items(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.on_press(ProjectPageMessage::OpenCreateNewTask.into())
	.style(theme::Button::custom(TransparentButtonStyle))
}

pub fn cancel_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
		.style(theme::Button::custom(GreenButtonStyle))
}

fn context_menu_button(content: impl Into<Element<'static, UiMessage>>) -> Button<'static, UiMessage>{
	button(content)
		.style(theme::Button::custom(SecondaryButtonStyle))
}

pub fn delete_project_button(project_id: ProjectId) -> Button<'static, UiMessage> {
	context_menu_button(
		row![
			icon_to_text(Bootstrap::Trash),
			text("Delete")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_items(Alignment::Center)
	)
	.on_press(UiMessage::DeleteProject(project_id))
}

pub fn move_project_up_button(project_id: ProjectId) -> Button<'static, UiMessage> {
	context_menu_button(
		row![
			icon_to_text(Bootstrap::ArrowUp),
			text("Move up")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_items(Alignment::Center)
	)
	.on_press(UiMessage::MoveProjectUp(project_id))
}

pub fn move_project_down_button(project_id: ProjectId) -> Button<'static, UiMessage> {
	context_menu_button(
		row![
			icon_to_text(Bootstrap::ArrowDown),
			text("Move down")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_items(Alignment::Center)
	)
	.on_press(UiMessage::MoveProjectDown(project_id))
}

pub fn settings_button() -> Button<'static, UiMessage> {
	button(
		container(
			icon_to_text(Bootstrap::Gear)
				.size(LARGE_TEXT_SIZE)
		)
		.padding(Padding{ left: 2.5, right: 2.5, top: 0.0, bottom: 0.0 })
	)
	.on_press(UiMessage::OpenSettings)
	.style(theme::Button::Secondary)
}

pub fn overview_button(selected: bool) -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::List)
				.size(LARGE_TEXT_SIZE),

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