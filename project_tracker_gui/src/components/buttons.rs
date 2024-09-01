use std::path::PathBuf;
use iced::{alignment::{Horizontal, Vertical}, theme, widget::{button, container, row, text, tooltip, tooltip::Position, Button}, Alignment, Border, Color, Element, Length};
use iced_aw::{core::icons::bootstrap::{icon_to_text, Bootstrap}, quad::Quad, widgets::InnerBounds, Spinner};
use crate::{
	components::{date_text, ConfirmModalMessage, ManageTaskTagsModalMessage, SettingsModalMessage}, core::{DatabaseMessage, DateFormatting, PreferenceMessage, ProjectId, SerializableDate, TaskId, TaskTag, TaskTagId}, pages::{ProjectPageMessage, SidebarPageMessage}, project_tracker::UiMessage, styles::{disabled_primary_text_color, primary_text_color, ColorPaletteButtonStyle, DangerousButtonStyle, DeleteButtonStyle, DeleteDoneTasksButtonStyle, InvisibleButtonStyle, ProjectPreviewButtonStyle, SecondaryButtonStyle, SelectionListButtonStyle, TaskTagButtonStyle, TooltipContainerStyle, GAP, LARGE_TEXT_SIZE, SMALL_HORIZONTAL_PADDING, SMALL_SPACING_AMOUNT, SMALL_TEXT_SIZE, SPACING_AMOUNT}, theme_mode::ThemeMode
};

pub fn create_new_project_button(enabled: bool) -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::PlusSquareFill)
				.style(if enabled { primary_text_color() } else { disabled_primary_text_color() }),
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
	.style(theme::Button::custom(SecondaryButtonStyle::default()))
}

pub fn create_new_task_button(enabled: bool) -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::PlusCircleFill)
				.style(if enabled { primary_text_color() } else { disabled_primary_text_color() }),
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
	.style(theme::Button::custom(SecondaryButtonStyle::default()))
}

pub fn cancel_create_project_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
		.on_press(SidebarPageMessage::CloseCreateNewProject.into())
		.style(theme::Button::custom(SecondaryButtonStyle::default()))
}

pub fn cancel_create_task_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
		.on_press(ProjectPageMessage::CloseCreateNewTask.into())
		.style(theme::Button::custom(SecondaryButtonStyle::ONLY_ROUND_RIGHT))
}

pub fn delete_project_button(project_id: ProjectId, project_name: &str) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Trash)
	)
	.on_press(ConfirmModalMessage::open(format!("Delete Project '{project_name}'?"), DatabaseMessage::DeleteProject(project_id)))
	.style(theme::Button::custom(DeleteButtonStyle{ round_left: true, round_right: true }))
}

pub fn delete_task_button(project_id: ProjectId, task_id: TaskId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Trash)
	)
	.on_press(DatabaseMessage::DeleteTask { project_id, task_id }.into())
	.style(theme::Button::custom(DeleteButtonStyle{ round_left: false, round_right: true }))
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
	.style(theme::Button::custom(SecondaryButtonStyle::default()))
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

pub fn theme_mode_button(theme_mode: ThemeMode, current_theme_mode: ThemeMode, round_left: bool, round_right: bool) -> Button<'static, UiMessage> {
	button(
		text(format!("{:?}", theme_mode))
			.horizontal_alignment(Horizontal::Center)
	)
	.style(theme::Button::custom(SelectionListButtonStyle{
		selected: theme_mode == current_theme_mode,
		round_left,
		round_right
	}))
	.width(80.0)
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
	.style(theme::Button::custom(SecondaryButtonStyle::default()))
}

pub fn select_synchronization_filepath_button() -> Element<'static, UiMessage> {
	tooltip(
		button(
			icon_to_text(Bootstrap::Folder)
		)
		.on_press(SettingsModalMessage::BrowseSynchronizationFilepath.into())
		.style(theme::Button::custom(SecondaryButtonStyle::default())),

		text("Select file").size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(GAP)
	.style(theme::Container::Custom(Box::new(TooltipContainerStyle)))
	.into()
}

pub fn clear_synchronization_filepath_button() -> Element<'static, UiMessage> {
	tooltip(
		button(icon_to_text(Bootstrap::XLg))
			.on_press(PreferenceMessage::SetSynchronizationFilepath(None).into())
			.style(theme::Button::custom(SecondaryButtonStyle::default())),

		text("Clear").size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(GAP)
	.style(theme::Container::Custom(Box::new(TooltipContainerStyle)))
	.into()
}

pub fn date_formatting_button(format: &DateFormatting, selected_format: &DateFormatting, is_left: bool) -> Button<'static, UiMessage> {
	button(
		text(format.as_str()).horizontal_alignment(Horizontal::Center)
	)
	.width(120.0)
	.on_press(SettingsModalMessage::SetDateFormatting(*format).into())
	.style(theme::Button::custom(SelectionListButtonStyle {
		selected: *selected_format == *format,
		round_left: is_left,
		round_right: !is_left,
	}))
}

pub fn copy_to_clipboard_button(copied_text: String) -> Element<'static, UiMessage> {
	tooltip(
		button(
			icon_to_text(Bootstrap::Clipboard)
		)
		.on_press(UiMessage::CopyToClipboard(copied_text))
		.style(theme::Button::custom(SecondaryButtonStyle::default())),

		text("Copy to clipboard")
			.size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(GAP)
	.style(theme::Container::Custom(Box::new(TooltipContainerStyle)))
	.into()
}

pub fn toggle_sidebar_button() -> Element<'static, UiMessage> {
	tooltip(
		button(
			icon_to_text(Bootstrap::LayoutSidebar)
		)
		.on_press(PreferenceMessage::ToggleShowSidebar.into())
		.style(theme::Button::custom(SecondaryButtonStyle::default())),

		text("Toggle sidebar (Ctrl + B)").size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(GAP)
	.style(theme::Container::Custom(Box::new(TooltipContainerStyle)))
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

pub fn task_tag_button(task_tag: &TaskTag, toggled: bool, round_bottom: bool, tall: bool) -> Button<'static, UiMessage> {
	let button = button(
		text(&task_tag.name)
	)
	.style(theme::Button::custom(TaskTagButtonStyle{
		color: task_tag.color.into(),
		toggled,
		round_bottom
	}));

	if tall {
		button
	}
	else {
		button.padding(SMALL_HORIZONTAL_PADDING)
	}
}

pub fn manage_task_tags_button(project_id: ProjectId) -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::BookmarkFill)
				.style(theme::Text::Default),
			text("Manage")
		]
		.align_items(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.on_press(ManageTaskTagsModalMessage::Open { project_id }.into())
	.style(theme::Button::custom(SecondaryButtonStyle::default()))
}

pub fn create_new_task_tags_button() -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::BookmarkPlusFill)
				.style(primary_text_color()),
			text("Create new")
		]
		.align_items(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.on_press(ManageTaskTagsModalMessage::OpenCreateNewTaskTag.into())
	.style(theme::Button::custom(SecondaryButtonStyle::default()))
}

pub fn cancel_create_new_task_tag_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
			.on_press(ManageTaskTagsModalMessage::CloseCreateNewTaskTag.into())
			.style(theme::Button::custom(SecondaryButtonStyle::ONLY_ROUND_RIGHT))
}

pub fn delete_task_tag_button(task_tag_id: TaskTagId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Trash)
	)
	.on_press(ManageTaskTagsModalMessage::DeleteTaskTag(task_tag_id).into())
	.style(theme::Button::custom(DeleteButtonStyle{ round_left: true, round_right: true }))
}

pub fn clear_task_needed_time_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::XLg)
	)
	.on_press(ProjectPageMessage::ClearTaskNeededTime.into())
	.style(theme::Button::custom(SecondaryButtonStyle {
		round_right_bottom: true,
		..SecondaryButtonStyle::NO_ROUNDING
	}))
}

pub fn clear_task_due_date_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::XLg)
	)
	.padding(SMALL_HORIZONTAL_PADDING)
	.on_press(ProjectPageMessage::ClearTaskDueDate.into())
	.style(theme::Button::custom(SecondaryButtonStyle {
		round_right_bottom: true,
		..SecondaryButtonStyle::NO_ROUNDING
	}))
}

pub fn add_due_date_button() -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::CalendarCheck),
			text("Add due date")
		]
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.padding(SMALL_HORIZONTAL_PADDING)
	.on_press(ProjectPageMessage::EditTaskDueDate.into())
	.style(theme::Button::custom(SecondaryButtonStyle::ONLY_ROUND_BOTTOM))
}

pub fn edit_due_date_button(due_date: &SerializableDate, date_formatting: DateFormatting) -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::CalendarCheck),
			date_text(due_date, date_formatting)
		]
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.padding(SMALL_HORIZONTAL_PADDING)
	.on_press(ProjectPageMessage::EditTaskDueDate.into())
	.style(theme::Button::custom(SecondaryButtonStyle {
		round_left_bottom: true,
		..SecondaryButtonStyle::NO_ROUNDING
	}))
}

pub fn color_palette_item_button(color: Color, selected: bool, on_press: UiMessage) -> Button<'static, UiMessage> {
	button(
		Quad {
			width: Length::Fixed(25.0),
			height: Length::Fixed(25.0),
			inner_bounds: InnerBounds::Ratio(0.8, 0.8),
			quad_color: color.into(),
			quad_border: Border::with_radius(f32::MAX),
			bg_color: None,
			..Default::default()
		}
	)
	.on_press(on_press)
	.style(theme::Button::custom(ColorPaletteButtonStyle{ selected }))
}

pub fn confirm_ok_button(on_confirmed: &UiMessage) -> Button<'static, UiMessage> {
	button(
		text("Ok")
			.horizontal_alignment(Horizontal::Center)
	)
	.width(Length::Fill)
	.style(theme::Button::custom(DangerousButtonStyle))
	.on_press(UiMessage::ConfirmModalConfirmed(Box::new(on_confirmed.clone())))
}

pub fn confirm_cancel_button() -> Button<'static, UiMessage> {
	button(
		text("Cancel")
			.horizontal_alignment(Horizontal::Center)
	)
	.width(Length::Fill)
	.style(theme::Button::custom(SecondaryButtonStyle::default()))
	.on_press(ConfirmModalMessage::Close.into())
}