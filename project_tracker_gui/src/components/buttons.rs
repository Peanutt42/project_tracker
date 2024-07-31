use std::path::PathBuf;
use iced::{alignment::Horizontal, theme, widget::{container, button, row, text, tooltip, tooltip::Position, Button}, Alignment, alignment::Vertical, Element, Length};
use iced_aw::{Spinner, core::icons::bootstrap::{icon_to_text, Bootstrap}};
use crate::{
	components::{ConfirmModalMessage, SettingsModalMessage}, core::{DatabaseMessage, PreferenceMessage, ProjectId, TaskId, TaskTag}, pages::{ProjectPageMessage, SidebarPageMessage}, project_tracker::UiMessage, styles::{DangerousButtonStyle, DeleteButtonStyle, DeleteDoneTasksButtonStyle, InvisibleButtonStyle, CancelButtonStyle, ProjectPreviewButtonStyle, RoundedContainerStyle, RoundedSecondaryButtonStyle, TaskTagButtonStyle, ThemeModeButtonStyle, DISABLED_GREEN_TEXT_STYLE, GREEN_TEXT_STYLE, LARGE_TEXT_SIZE, SMALL_SPACING_AMOUNT, SMALL_TEXT_SIZE, SPACING_AMOUNT}, theme_mode::ThemeMode
};

pub fn create_new_project_button(enabled: bool) -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::PlusSquareFill)
				.style(if enabled { GREEN_TEXT_STYLE } else { DISABLED_GREEN_TEXT_STYLE }),
			text("New project")
		]
		.align_items(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.on_press_maybe(
		if enabled {
			Some(SidebarPageMessage::OpenCreateNewProject.into())
		}
		else {
			None
		}
	)
	.style(theme::Button::custom(RoundedSecondaryButtonStyle))
}

pub fn create_new_task_button(enabled: bool) -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::PlusCircleFill)
				.style(if enabled { GREEN_TEXT_STYLE } else { DISABLED_GREEN_TEXT_STYLE }),
			text("New task")
		]
		.align_items(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.on_press_maybe(
		if enabled {
			Some(ProjectPageMessage::OpenCreateNewTask.into())
		}
		else {
			None
		}
	)
	.style(theme::Button::custom(RoundedSecondaryButtonStyle))
}

pub fn cancel_create_project_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
		.on_press(SidebarPageMessage::CloseCreateNewProject.into())
		.style(theme::Button::custom(CancelButtonStyle{ round_left: true, round_right: true }))
}

pub fn cancel_create_task_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
		.on_press(ProjectPageMessage::CloseCreateNewTask.into())
		.style(theme::Button::custom(CancelButtonStyle{ round_left: false, round_right: true }))
}

pub fn delete_project_button(project_id: ProjectId, project_name: &str) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Trash)
	)
	.on_press(ConfirmModalMessage::open(format!("Delete Project '{project_name}'?"), DatabaseMessage::DeleteProject(project_id)))
	.style(theme::Button::custom(DeleteButtonStyle))
}

pub fn delete_task_button(project_id: ProjectId, task_id: TaskId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Trash)
	)
	.on_press(DatabaseMessage::DeleteTask { project_id, task_id }.into())
	.style(theme::Button::custom(DeleteButtonStyle))
}

pub fn delete_all_done_tasks_button(project_id: ProjectId, project_name: &str) -> Button<'static, UiMessage> {
	button(row![
		icon_to_text(Bootstrap::Trash),
		text("Delete done tasks")
	])
	.on_press(ConfirmModalMessage::open(format!("Delete all done tasks of project '{project_name}'?"), DatabaseMessage::DeleteDoneTasks(project_id)))
	.style(theme::Button::custom(DeleteDoneTasksButtonStyle))
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

pub fn dangerous_button(icon: Bootstrap, text: &'static str, confirm_label: Option<String>, on_press: impl Into<UiMessage>) -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(icon),
			iced::widget::text(text)
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_items(Alignment::Center)
	)
	.style(theme::Button::custom(DangerousButtonStyle))
	.on_press(if let Some(label) = confirm_label {
		ConfirmModalMessage::open(label, on_press)
	}
	else {
		on_press.into()
	})
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

pub fn settings_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Gear)
		    .horizontal_alignment(Horizontal::Center)
			.size(LARGE_TEXT_SIZE)
	)
	.width(Length::Fixed(35.0))
	.on_press(SettingsModalMessage::Open.into())
	.style(theme::Button::custom(RoundedSecondaryButtonStyle))
}

pub fn open_location_button(filepath: Option<PathBuf>) -> Element<'static, UiMessage> {
	tooltip(
		button(
			icon_to_text(Bootstrap::Folder)
		)
		.on_press_maybe(filepath.map(UiMessage::OpenFolderLocation))
		.style(theme::Button::custom(RoundedSecondaryButtonStyle)),

		text("Open folder location")
			.size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(10)
	.style(theme::Container::Custom(Box::new(RoundedContainerStyle)))
	.into()
}

pub fn copy_to_clipboard_button(copied_text: String) -> Element<'static, UiMessage> {
	tooltip(
		button(
			icon_to_text(Bootstrap::Clipboard)
		)
		.on_press(UiMessage::CopyToClipboard(copied_text))
		.style(theme::Button::custom(RoundedSecondaryButtonStyle)),

		text("Copy to clipboard")
			.size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(10)
	.style(theme::Container::Custom(Box::new(RoundedContainerStyle)))
	.into()
}

pub fn toggle_sidebar_button() -> Element<'static, UiMessage> {
	tooltip(
		button(
			icon_to_text(Bootstrap::LayoutSidebar)
		)
		.on_press(PreferenceMessage::ToggleShowSidebar.into())
		.style(theme::Button::custom(RoundedSecondaryButtonStyle)),

		text("Toggle sidebar (Ctrl + B)").size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(10)
	.style(theme::Container::Custom(Box::new(RoundedContainerStyle)))
	.into()
}

// only for layout purposes
pub fn invisible_toggle_sidebar_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::LayoutSidebar)
	)
	.style(theme::Button::custom(InvisibleButtonStyle))
}

pub fn sync_database_button(synchronizing: bool, synchronization_filepath: Option<PathBuf>) -> Element<'static, UiMessage> {
	button(
		row![
			container(
				if synchronizing {
					Element::new(
						Spinner::new()
							.width(Length::Fixed(16.0))
							.height(Length::Fixed(16.0))
							.circle_radius(2.0)
					)
				}
				else {
					icon_to_text(Bootstrap::ArrowClockwise)
						.vertical_alignment(Vertical::Center)
						.into()
				}
			)
			.center_y(),

			text("Synchronize")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_items(Alignment::Center)
	)
	.on_press_maybe(synchronization_filepath.map(|filepath| DatabaseMessage::Sync(filepath).into()))
	.style(theme::Button::custom(DangerousButtonStyle))
	.into()
}

pub fn task_tag_button(task_tag: &TaskTag, toggled: bool) -> Button<'static, UiMessage> {
	button(
		text(&task_tag.name)
	)
	.style(theme::Button::custom(TaskTagButtonStyle{
		color: task_tag.color.into(),
		toggled,
	}))
}

pub fn create_new_label_button() -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::BookmarkPlusFill)
				.style(GREEN_TEXT_STYLE),
			text("Create new")
		]
		.align_items(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.on_press(ProjectPageMessage::OpenCreateNewTaskTag.into())
	.style(theme::Button::custom(RoundedSecondaryButtonStyle))
}

pub fn cancel_create_new_task_tag_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
		.on_press(ProjectPageMessage::CloseCreateNewTaskTag.into())
		.style(theme::Button::custom(CancelButtonStyle{ round_left: false, round_right: true }))
}