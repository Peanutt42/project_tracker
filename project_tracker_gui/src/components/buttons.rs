use std::path::PathBuf;
use iced::{alignment::{Horizontal, Vertical}, border::rounded, widget::{button, container, row, text, tooltip, tooltip::Position, Button}, Alignment, Color, Element, Length::{self, Fill}};
use iced_aw::{quad::Quad, widgets::InnerBounds, Spinner};
use crate::{
	components::{date_text, ConfirmModalMessage, ManageTaskTagsModalMessage, SettingTab, SettingsModalMessage}, core::{DatabaseMessage, DateFormatting, PreferenceMessage, ProjectId, SerializableDate, TaskId, TaskTag, TaskTagId}, pages::{format_stopwatch_duration, ProjectPageMessage, SidebarPageMessage, StopwatchPage, StopwatchPageMessage, STOPWATCH_TASK_DROPZONE_ID}, project_tracker::UiMessage, styles::{color_palette_button_style, dangerous_button_style, delete_button_style, delete_done_tasks_button_style, hidden_secondary_button_style, invisible_button_style, primary_button_style, project_preview_style, secondary_button_style, secondary_button_style_default, secondary_button_style_only_round_bottom, secondary_button_style_only_round_right, selection_list_button_style, settings_tab_button_style, task_tag_button_style, timer_button_style, tooltip_container_style, GAP, LARGE_TEXT_SIZE, SMALL_HORIZONTAL_PADDING, SMALL_SPACING_AMOUNT, SMALL_TEXT_SIZE, SPACING_AMOUNT}, theme_mode::ThemeMode,
	icons::{icon_to_text, Bootstrap}
};

fn icon_button(label: impl text::IntoFragment<'static>, icon: Bootstrap) -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(icon),
			text(label)
		]
		.align_y(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
	)
}

pub fn create_new_project_button(enabled: bool) -> Button<'static, UiMessage> {
	icon_button("New project", Bootstrap::PlusSquareFill)
		.on_press_maybe(
			if enabled {
				Some(SidebarPageMessage::OpenCreateNewProject.into())
			}
			else {
				None
			}
		)
		.style(primary_button_style)
}

pub fn create_new_task_button(enabled: bool) -> Button<'static, UiMessage> {
	icon_button("New task", Bootstrap::PlusCircleFill)
		.on_press_maybe(
			if enabled {
				Some(ProjectPageMessage::OpenCreateNewTask.into())
			}
			else {
				None
			}
		)
		.style(primary_button_style)
}

pub fn cancel_create_project_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
		.on_press(SidebarPageMessage::CloseCreateNewProject.into())
		.style(secondary_button_style_default)
}

pub fn cancel_create_task_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
		.on_press(ProjectPageMessage::CloseCreateNewTask.into())
		.style(secondary_button_style_only_round_right)
}

pub fn delete_project_button(project_id: ProjectId, project_name: &str) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Trash)
	)
	.on_press(ConfirmModalMessage::open(format!("Delete Project '{project_name}'?"), DatabaseMessage::DeleteProject(project_id)))
	.style(move |t, s| delete_button_style(t, s, true, true))
}

pub fn delete_task_button(project_id: ProjectId, task_id: TaskId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Trash)
	)
	.on_press(DatabaseMessage::DeleteTask { project_id, task_id }.into())
	.style(move |t, s| delete_button_style(t, s, false, true))
}

pub fn delete_all_done_tasks_button(project_id: ProjectId, project_name: &str) -> Button<'static, UiMessage> {
	button(
		row![
			icon_to_text(Bootstrap::Trash),
			text("Delete done tasks")
		]
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.on_press(ConfirmModalMessage::open(format!("Delete all done tasks of project '{project_name}'?"), DatabaseMessage::DeleteDoneTasks(project_id)))
	.style(delete_done_tasks_button_style)
}

pub fn show_done_tasks_button(show: bool, done_task_len: usize) -> Button<'static, UiMessage> {
	icon_button(
		format!("{} done ({done_task_len})", if show { "Hide" } else { "Show" }),
		if show { Bootstrap::CaretDownFill } else { Bootstrap::CaretRightFill }
	)
	.on_press(ProjectPageMessage::ShowDoneTasks(!show).into())
	.style(secondary_button_style_default)
}

pub fn dangerous_button(icon: Bootstrap, text: &'static str, confirm_label: Option<String>, on_press: impl Into<UiMessage>) -> Button<'static, UiMessage> {
	icon_button(text, icon)
		.style(dangerous_button_style)
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
			.align_x(Horizontal::Center)
	)
	.style(move |t, s| selection_list_button_style(t, s, theme_mode == current_theme_mode, round_left, round_right))
	.width(80.0)
	.on_press(PreferenceMessage::SetThemeMode(theme_mode).into())
}

pub fn stopwatch_button(stopwatch_page: &StopwatchPage, selected: bool) -> Element<'static, UiMessage> {
	let stopwatch_label = match stopwatch_page {
		StopwatchPage::Ticking { clock, elapsed_time, .. } => {
			Some(if clock.label().is_empty() {
				format_stopwatch_duration(elapsed_time.as_secs_f64().round_ties_even() as i64)
			}
			else {
				clock.label().to_string()
			})
		},
		_ => None
	};

	let stopwatch_ticking = matches!(stopwatch_page, StopwatchPage::Ticking { .. });

	container(
		button(
			row![
				icon_to_text(Bootstrap::Stopwatch)
					.size(LARGE_TEXT_SIZE),

				text("Stopwatch")
					.size(LARGE_TEXT_SIZE)
					.width(Fill)
			]
			.push_maybe(
				stopwatch_label.map(|stopwatch_label| {
					container(
						text(stopwatch_label)
							.size(SMALL_TEXT_SIZE)
					)
					.width(Fill)
					.align_x(Horizontal::Right)
				})
			)
			.width(Fill)
			.spacing(SPACING_AMOUNT)
			.align_y(Alignment::Center)
			.padding(SMALL_HORIZONTAL_PADDING)
		)
		.width(Fill)
		.on_press(UiMessage::OpenStopwatch)
		.style(move |t, s| project_preview_style(
			t,
			s,
			selected || stopwatch_ticking,
			if stopwatch_ticking {
				Some(Color::from_rgb(1.0, 0.0, 0.0))}
			else {
				None
			}
		))
	)
	.id(STOPWATCH_TASK_DROPZONE_ID.clone())
	.into()
}

pub fn settings_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Gear)
		    .align_x(Horizontal::Center)
			.size(LARGE_TEXT_SIZE)
	)
	.width(Length::Fixed(1.75 * LARGE_TEXT_SIZE))
	.on_press(SettingsModalMessage::Open.into())
	.style(secondary_button_style_default)
}

pub fn select_synchronization_filepath_button() -> Element<'static, UiMessage> {
	tooltip(
		button(
			icon_to_text(Bootstrap::Folder)
		)
		.on_press(SettingsModalMessage::BrowseSynchronizationFilepath.into())
		.style(secondary_button_style_default),

		text("Select file").size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn clear_synchronization_filepath_button() -> Element<'static, UiMessage> {
	tooltip(
		button(icon_to_text(Bootstrap::XLg))
			.on_press(PreferenceMessage::SetSynchronizationFilepath(None).into())
			.style(secondary_button_style_default),

		text("Clear").size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn date_formatting_button<'a>(format: &'a DateFormatting, selected_format: &'a DateFormatting, is_left: bool) -> Button<'a, UiMessage> {
	button(
		text(format.as_str()).align_x(Horizontal::Center)
	)
	.width(120.0)
	.on_press(SettingsModalMessage::SetDateFormatting(*format).into())
	.style(move |t, s| selection_list_button_style(t, s, *selected_format == *format, is_left, !is_left))
}

pub fn copy_to_clipboard_button(copied_text: String) -> Element<'static, UiMessage> {
	tooltip(
		button(
			icon_to_text(Bootstrap::Clipboard)
		)
		.on_press(UiMessage::CopyToClipboard(copied_text))
		.style(secondary_button_style_default),

		text("Copy to clipboard")
			.size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn toggle_sidebar_button() -> Element<'static, UiMessage> {
	tooltip(
		button(
			icon_to_text(Bootstrap::LayoutSidebar)
		)
		.on_press(PreferenceMessage::ToggleShowSidebar.into())
		.style(secondary_button_style_default),

		text("Toggle sidebar (Ctrl + B)").size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

// only for layout purposes
pub fn invisible_toggle_sidebar_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::LayoutSidebar)
	)
	.style(invisible_button_style)
}

pub fn import_database_button(importing: bool) -> Element<'static, UiMessage> {
	button(
		row![
			container(
				if importing {
					Element::new(
						Spinner::new()
							.width(Length::Fixed(16.0))
							.height(Length::Fixed(16.0))
							.circle_radius(2.0)
					)
				}
				else {
					icon_to_text(Bootstrap::Download)
						.align_y(Vertical::Center)
						.into()
				}
			)
			.center_y(Fill),

			text("Import")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_y(Alignment::Center)
	)
	.on_press_maybe(if importing {
		None
	}
	else {
		Some(UiMessage::ImportDatabaseDialog)
	})
	.style(dangerous_button_style)
	.into()
}

pub fn export_database_button(importing: bool) -> Element<'static, UiMessage> {
	button(
		row![
			container(
				if importing {
					Element::new(
						Spinner::new()
							.width(Length::Fixed(16.0))
							.height(Length::Fixed(16.0))
							.circle_radius(2.0)
					)
				}
				else {
					icon_to_text(Bootstrap::Upload)
						.align_y(Vertical::Center)
						.into()
				}
			)
			.center_y(Fill),

			text("Export")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_y(Alignment::Center)
	)
	.on_press_maybe(if importing {
		None
	}
	else {
		Some(UiMessage::ExportDatabaseDialog)
	})
	.style(dangerous_button_style)
	.into()
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
						.align_y(Vertical::Center)
						.into()
				}
			)
			.center_y(Fill),

			text("Synchronize")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_y(Alignment::Center)
	)
	.on_press_maybe(synchronization_filepath.map(UiMessage::SyncDatabase))
	.style(dangerous_button_style)
	.into()
}

pub fn task_tag_button(task_tag: &TaskTag, toggled: bool, round_bottom: bool, tall: bool) -> Button<UiMessage> {
	let button = button(
		text(&task_tag.name)
	)
	.style(move |t, s| task_tag_button_style(t, s, task_tag.color.into(), toggled, round_bottom));

	if tall {
		button
	}
	else {
		button.padding(SMALL_HORIZONTAL_PADDING)
	}
}

pub fn manage_task_tags_button(project_id: ProjectId) -> Element<'static, UiMessage> {
	tooltip(
		button(icon_to_text(Bootstrap::Bookmark))
			.on_press(ManageTaskTagsModalMessage::Open { project_id }.into())
			.style(secondary_button_style_default),

		text("Manage tags").size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn create_new_task_tags_button() -> Button<'static, UiMessage> {
	icon_button("Create new", Bootstrap::BookmarkPlusFill)
		.on_press(ManageTaskTagsModalMessage::OpenCreateNewTaskTag.into())
		.style(primary_button_style)
}

pub fn cancel_create_new_task_tag_button() -> Button<'static, UiMessage> {
	button(icon_to_text(Bootstrap::XLg))
			.on_press(ManageTaskTagsModalMessage::CloseCreateNewTaskTag.into())
			.style(secondary_button_style_only_round_right)
}

pub fn delete_task_tag_button(task_tag_id: TaskTagId) -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Trash)
	)
	.on_press(ManageTaskTagsModalMessage::DeleteTaskTag(task_tag_id).into())
	.style(move |t, s| delete_button_style(t, s, true, true))
}

pub fn clear_task_needed_time_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::XLg)
	)
	.on_press(ProjectPageMessage::ClearTaskNeededTime.into())
	.style(move |t, s| secondary_button_style(t, s, false, false, false, true))
}

pub fn clear_task_due_date_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::XLg)
	)
	.padding(SMALL_HORIZONTAL_PADDING)
	.on_press(ProjectPageMessage::ClearTaskDueDate.into())
	.style(move |t, s| secondary_button_style(t, s, false, false, false, true))
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
	.style(secondary_button_style_only_round_bottom)
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
	.style(move |t, s| secondary_button_style(t, s, false, true, false, false))
}

pub fn edit_color_palette_button(color: Color, on_press: UiMessage) -> Element<'static, UiMessage> {
	tooltip(
		color_palette_item_button(color, false, on_press),
		text("Edit color").size(SMALL_TEXT_SIZE),
		Position::Bottom
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn color_palette_item_button(color: Color, selected: bool, on_press: UiMessage) -> Button<'static, UiMessage> {
	button(
		Quad {
			width: Length::Fixed(25.0),
			height: Length::Fixed(25.0),
			inner_bounds: InnerBounds::Ratio(0.8, 0.8),
			quad_color: color.into(),
			quad_border: rounded(f32::MAX),
			bg_color: None,
			..Default::default()
		}
	)
	.on_press(on_press)
	.style(move |t, s| color_palette_button_style(t, s, selected))
}

pub fn confirm_ok_button(on_confirmed: &UiMessage) -> Button<'static, UiMessage> {
	button(
		text("Ok")
			.align_x(Horizontal::Center)
	)
	.width(Fill)
	.style(dangerous_button_style)
	.on_press(UiMessage::ConfirmModalConfirmed(Box::new(on_confirmed.clone())))
}

pub fn confirm_cancel_button() -> Button<'static, UiMessage> {
	button(
		text("Cancel")
			.align_x(Horizontal::Center)
	)
	.width(Fill)
	.style(secondary_button_style_default)
	.on_press(ConfirmModalMessage::Close.into())
}

pub fn search_tasks_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::Search)
	)
	.style(secondary_button_style_default)
	.on_press(ProjectPageMessage::OpenSearchTasks.into())
}

pub fn cancel_search_tasks_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::XLg)
	)
	.style(secondary_button_style_only_round_right)
	.on_press(ProjectPageMessage::CloseSearchTasks.into())
}

pub fn settings_tab_button(tab: SettingTab, selected_tab: SettingTab) -> Button<'static, UiMessage> {
	button(
		text(format!("{tab:?}"))
	)
	.width(Fill)
	.style(move |t, s| settings_tab_button_style(t, s, tab == selected_tab))
	.on_press(SettingsModalMessage::SwitchSettingsTab(tab).into())
}

pub fn import_source_code_todos_button() -> Element<'static, UiMessage> {
	tooltip(
		button(
			icon_to_text(Bootstrap::FileEarmarkCode)
		)
		.on_press(ProjectPageMessage::ImportSourceCodeTodosDialog.into())
		.style(secondary_button_style_default),

		text("Import TODO's").size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn reimport_source_code_todos_button() -> Button<'static, UiMessage> {
	icon_button("Reimport TODO's", Bootstrap::FileEarmarkCode)
		.on_press(ProjectPageMessage::ImportSourceCodeTodosDialog.into())
		.style(secondary_button_style_default)
}

pub fn show_source_code_todos_button(show: bool, source_code_todos_len: usize) -> Button<'static, UiMessage> {
	icon_button(
		format!("{} source code todos ({source_code_todos_len})", if show { "Hide" } else { "Show" }),
		if show { Bootstrap::CaretDownFill } else { Bootstrap::CaretRightFill }
	)
	.on_press(ProjectPageMessage::ShowSourceCodeTodos(!show).into())
	.style(secondary_button_style_default)
}

pub fn edit_project_name_button() -> Element<'static, UiMessage> {
	tooltip(
		button(
			icon_to_text(Bootstrap::PencilSquare)
		)
		.on_press(ProjectPageMessage::EditProjectName.into())
		.style(hidden_secondary_button_style),

		text("Edit name").size(SMALL_TEXT_SIZE),

		Position::Bottom
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn start_timer_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::PlayFill)
			.size(45)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center)
	)
	.width(Length::Fixed(1.75 * 45.0))
	.height(Length::Fixed(1.75 * 45.0))
	.on_press(StopwatchPageMessage::Start{ task: None }.into())
	.style(move |t, s| timer_button_style(t, s, false))
}

pub fn stop_timer_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::XLg)
			.size(45)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center)
	)
	.width(Length::Fixed(1.75 * 45.0))
	.height(Length::Fixed(1.75 * 45.0))
	.on_press(StopwatchPageMessage::Stop.into())
	.style(move |t, s| timer_button_style(t, s, true))
}

pub fn resume_timer_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::PlayFill)
			.size(45)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center)
	)
	.width(Length::Fixed(1.75 * 45.0))
	.height(Length::Fixed(1.75 * 45.0))
	.on_press(StopwatchPageMessage::Resume.into())
	.style(move |t, s| timer_button_style(t, s, true))
}

pub fn pause_timer_button() -> Button<'static, UiMessage> {
	button(
		icon_to_text(Bootstrap::PauseFill)
			.size(45)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center)
	)
	.width(Length::Fixed(1.75 * 45.0))
	.height(Length::Fixed(1.75 * 45.0))
	.on_press(StopwatchPageMessage::Pause.into())
	.style(move |t, s| timer_button_style(t, s, true))
}

pub fn start_task_timer_button<'a>(project_id: ProjectId, task_id: TaskId, round_top_left: bool) -> Element<'a, UiMessage> {
	tooltip(
		button(
			icon_to_text(Bootstrap::Stopwatch)
		)
		.on_press(
			StopwatchPageMessage::Start{
				task: Some((project_id, task_id))
			}
			.into()
		)
		.style(move |t, s| secondary_button_style(t, s, round_top_left, true, false, false)),

		text("Start a timer for this task"),

		Position::Bottom
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn import_google_tasks_button() -> Button<'static, UiMessage> {
	button("Import")
		.on_press(SettingsModalMessage::ImportGoogleTasksFileDialog.into())
		.style(dangerous_button_style)
}