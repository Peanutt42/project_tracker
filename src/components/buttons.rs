use iced::{alignment::Horizontal, theme, widget::{button, container, row, text, Button}, Alignment, Length};
use iced_aw::core::icons::bootstrap::{icon_to_text, Bootstrap};
use crate::{
	core::{ProjectId, TaskId}, pages::{ProjectPageMessage, SidebarPageMessage}, project_tracker::UiMessage, styles::{DangerousButtonStyle, DeleteButtonStyle, InvisibleButtonStyle, ProjectContextButtonStyle, ProjectPreviewButtonStyle, ThemeModeButtonStyle, TransparentButtonStyle, BOLD_FONT, GREEN_TEXT_STYLE, LARGE_TEXT_SIZE, SMALL_SPACING_AMOUNT, SPACING_AMOUNT}, theme_mode::ThemeMode
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

pub fn cancel_create_project_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
		.on_press(SidebarPageMessage::CloseCreateNewProject.into())
		.style(theme::Button::custom(ProjectContextButtonStyle))
}

pub fn edit_project_button(project_id: ProjectId, visible: bool) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Pencil)
	)
	.on_press(SidebarPageMessage::EditProject(project_id).into())
	.style(if visible { theme::Button::custom(ProjectContextButtonStyle) } else { theme::Button::custom(InvisibleButtonStyle) })
}

pub fn edit_task_button(task_id: TaskId, visible: bool) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Pencil)
	)
	.on_press(ProjectPageMessage::EditTask(task_id).into())
	.style(if visible { theme::Button::custom(ProjectContextButtonStyle) } else { theme::Button::custom(InvisibleButtonStyle) })
}

pub fn delete_project_button(project_id: ProjectId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Trash)
	)
	.on_press(UiMessage::DeleteProject(project_id))
	.style(theme::Button::custom(DeleteButtonStyle))
}

pub fn delete_task_button(project_id: ProjectId, task_id: TaskId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Trash)
	)
	.on_press(UiMessage::DeleteTask{ project_id, task_id })
	.style(theme::Button::custom(DeleteButtonStyle))
}

pub fn move_project_up_button(project_id: ProjectId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::ArrowUp),
	)
	.on_press(UiMessage::MoveProjectUp(project_id))
	.style(theme::Button::custom(ProjectContextButtonStyle))
}

pub fn move_task_up_button(project_id: ProjectId, task_id: TaskId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::ArrowUp),
	)
	.on_press(UiMessage::MoveTaskUp{ project_id, task_id })
	.style(theme::Button::custom(ProjectContextButtonStyle))
}

pub fn move_project_down_button(project_id: ProjectId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::ArrowDown),
	)
	.on_press(UiMessage::MoveProjectDown(project_id))
	.style(theme::Button::custom(ProjectContextButtonStyle))
}

pub fn move_task_down_button(project_id: ProjectId, task_id: TaskId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::ArrowDown),
	)
	.on_press(UiMessage::MoveTaskDown{ project_id, task_id })
	.style(theme::Button::custom(ProjectContextButtonStyle))
}

pub fn dangerous_button(label: &str) -> Button<'static, UiMessage> {
	button(
		text(label)
			.font(BOLD_FONT)
	)
	.style(theme::Button::custom(DangerousButtonStyle))
}

pub fn theme_mode_button(theme_mode: ThemeMode, current_theme_mode: ThemeMode) -> Button<'static, UiMessage> {
	button(text(format!("{:?}", theme_mode)).horizontal_alignment(Horizontal::Center))
		.style(theme::Button::custom(ThemeModeButtonStyle{ selected: theme_mode == current_theme_mode }))
		.width(60.0)
		.on_press(UiMessage::SetThemeMode(theme_mode))
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

pub fn settings_button(selected: bool) -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::Gear)
				.size(LARGE_TEXT_SIZE),

			text("Settings")
				.size(LARGE_TEXT_SIZE)
				.width(Length::Fill)
		]
		.align_items(Alignment::Center)
		.spacing(SPACING_AMOUNT)
		.width(Length::Fill)
	)
	.width(Length::Fill)
	.on_press(UiMessage::OpenSettings)
	.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }))
}
