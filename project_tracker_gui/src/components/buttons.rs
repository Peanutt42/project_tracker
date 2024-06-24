use std::path::PathBuf;
use iced::{alignment::Horizontal, theme, widget::{button, row, text, Button}, Alignment, Length};
use iced_aw::core::icons::bootstrap::{icon_to_text, Bootstrap};
use crate::{
	core::{DatabaseMessage, PreferenceMessage, ProjectId, ProjectMessage, TaskId}, pages::{ProjectPageMessage, SidebarPageMessage}, project_tracker::UiMessage, styles::{DangerousButtonStyle, DeleteButtonStyle, DeleteDoneTasksButtonStyle, InvisibleButtonStyle, ProjectContextButtonStyle, ProjectPreviewButtonStyle, ThemeModeButtonStyle, TransparentButtonStyle, BOLD_FONT, DISABLED_GREEN_TEXT_STYLE, GREEN_TEXT_STYLE, LARGE_TEXT_SIZE, SMALL_SPACING_AMOUNT, SPACING_AMOUNT}, theme_mode::ThemeMode
};

use super::ConfirmModalMessage;

pub fn create_new_project_button(enabled: bool) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::PlusSquareFill)
			.size(LARGE_TEXT_SIZE * 1.7)
			.horizontal_alignment(iced::alignment::Horizontal::Center)
			.style(if enabled { GREEN_TEXT_STYLE } else { DISABLED_GREEN_TEXT_STYLE })
	)
	.on_press_maybe(if enabled {
		Some(SidebarPageMessage::OpenCreateNewProject.into())
	}
	else {
		None
	})
	.width(LARGE_TEXT_SIZE * 2.715)
	.height(LARGE_TEXT_SIZE * 2.715)
	.style(theme::Button::custom(TransparentButtonStyle))
}

pub fn create_new_task_button(enabled: bool) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::PlusCircleFill)
			.size(LARGE_TEXT_SIZE * 1.7)
			.horizontal_alignment(iced::alignment::Horizontal::Center)
			.style(if enabled { GREEN_TEXT_STYLE } else { DISABLED_GREEN_TEXT_STYLE })
	)
	.width(LARGE_TEXT_SIZE * 2.715)
	.height(LARGE_TEXT_SIZE * 2.715)
	.on_press_maybe(if enabled {
		Some(ProjectPageMessage::OpenCreateNewTask.into())
	}
	else {
		None
	})
	.style(theme::Button::custom(TransparentButtonStyle))
}

pub fn cancel_create_project_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
		.on_press(SidebarPageMessage::CloseCreateNewProject.into())
		.style(theme::Button::custom(ProjectContextButtonStyle))
}

pub fn cancel_create_task_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
		.on_press(ProjectPageMessage::CloseCreateNewTask.into())
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
	.on_press(ConfirmModalMessage::Open {
		title: "Delete Project".to_string(),
		on_confirmed: Box::new(DatabaseMessage::DeleteProject(project_id).into()),
	}.into())
	.style(theme::Button::custom(DeleteButtonStyle))
}

pub fn delete_task_button(project_id: ProjectId, task_id: TaskId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Trash)
	)
	.on_press(DatabaseMessage::ProjectMessage { project_id, task_id, message: ProjectMessage::DeleteTask }.into())
	.style(theme::Button::custom(DeleteButtonStyle))
}

pub fn delete_all_done_tasks_button(project_id: ProjectId) -> Button<'static, UiMessage> {
	button(row![
		icon_to_text(Bootstrap::Trash),
		text("Delete done tasks")
	])
	.on_press(DatabaseMessage::DeleteDoneTasks(project_id).into())
	.style(theme::Button::custom(DeleteDoneTasksButtonStyle))
}

pub fn move_project_up_button(project_id: ProjectId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::ArrowUp),
	)
	.on_press(DatabaseMessage::MoveProjectUp(project_id).into())
	.style(theme::Button::custom(ProjectContextButtonStyle))
}

pub fn move_task_up_button(project_id: ProjectId, task_id: TaskId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::ArrowUp),
	)
	.on_press(DatabaseMessage::ProjectMessage { project_id, task_id, message: ProjectMessage::MoveTaskUp }.into())
	.style(theme::Button::custom(ProjectContextButtonStyle))
}

pub fn move_project_down_button(project_id: ProjectId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::ArrowDown),
	)
	.on_press(DatabaseMessage::MoveProjectDown(project_id).into())
	.style(theme::Button::custom(ProjectContextButtonStyle))
}

pub fn move_task_down_button(project_id: ProjectId, task_id: TaskId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::ArrowDown),
	)
	.on_press(DatabaseMessage::ProjectMessage { project_id, task_id, message: ProjectMessage::MoveTaskDown }.into())
	.style(theme::Button::custom(ProjectContextButtonStyle))
}

pub fn show_done_tasks_button(show: bool, done_task_len: usize) -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(if show { Bootstrap::CaretDownFill } else { Bootstrap::CaretRightFill }),
			text(format!("{} done ({done_task_len})", if show { "Hide" } else { "Show" })),
		]
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.on_press(ProjectPageMessage::ShowDoneTasks(!show).into())
	.style(theme::Button::Secondary)
}

pub fn dangerous_button(label: &str, on_press: impl Into<UiMessage>) -> Button<'static, UiMessage> {
	button(
		text(label)
			.font(BOLD_FONT)
	)
	.style(theme::Button::custom(DangerousButtonStyle))
	.on_press(ConfirmModalMessage::Open {
		title: label.to_string(),
		on_confirmed: Box::new(on_press.into())
	}.into())
}

pub fn theme_mode_button(theme_mode: ThemeMode, current_theme_mode: ThemeMode) -> Button<'static, UiMessage> {
	button(text(format!("{:?}", theme_mode)).horizontal_alignment(Horizontal::Center))
		.style(theme::Button::custom(ThemeModeButtonStyle{ selected: theme_mode == current_theme_mode }))
		.width(60.0)
		.on_press(PreferenceMessage::SetThemeMode(theme_mode).into())
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

pub fn open_location_button(filepath: Option<PathBuf>) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Folder)
	)
	.on_press_maybe(filepath.map(UiMessage::OpenFolderLocation))
	.style(theme::Button::Secondary)
}