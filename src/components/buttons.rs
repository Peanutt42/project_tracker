use crate::{
	components::{date_text, duration_text}, core::{
		DatabaseMessage, DateFormatting, PreferenceMessage, ProjectId, SerializableDate, SortMode, SynchronizationSetting, TaskId, TaskTag, TaskTagId
	}, icons::{icon_to_text, Bootstrap}, modals::{
		ConfirmModalMessage, CreateTaskModalMessage, ManageTaskTagsModalMessage, SettingTab, SettingsModalMessage, TaskModalMessage
	}, pages::{
		format_stopwatch_duration, ProjectPageMessage, SidebarPageMessage, StopwatchPage,
		StopwatchPageMessage, STOPWATCH_TASK_DROPZONE_ID,
	}, project_tracker::Message, styles::{
		circle_button_style, create_task_modal_ok_button_style, dangerous_button_style, delete_button_style, delete_done_tasks_button_style, dropdown_container_style, enum_dropdown_button_style, primary_button_style, secondary_button_style, secondary_button_style_default, secondary_button_style_no_rounding, secondary_button_style_only_round_left, secondary_button_style_only_round_right, secondary_button_style_only_round_top, selection_list_button_style, settings_tab_button_style, stopwatch_page_button_style, task_tag_button_style, timer_button_style, tooltip_container_style, GAP, LARGE_TEXT_SIZE, SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SMALL_TEXT_SIZE, SPACING_AMOUNT
	}, theme_mode::ThemeMode
};
use iced::{
	alignment::{Horizontal, Vertical}, border::rounded, widget::{button, column, container, row, text, tooltip, Button, Column}, Alignment, Color, Element, Length::{self, Fill, Fixed}};
use iced_aw::{drop_down::{self, Offset}, quad::Quad, widgets::InnerBounds, DropDown, Spinner};
use std::{borrow::Cow, path::PathBuf, time::Duration};

pub const ICON_FONT_SIZE: f32 = 16.0;
pub const ICON_BUTTON_WIDTH: f32 = ICON_FONT_SIZE * 1.8;

fn icon_button(icon: Bootstrap) -> Button<'static, Message> {
	button(
		icon_to_text(icon)
			.size(ICON_FONT_SIZE)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center),
	)
	.width(ICON_BUTTON_WIDTH)
}

fn large_icon_button(icon: Bootstrap) -> Button<'static, Message> {
	button(
		icon_to_text(icon)
			.size(LARGE_TEXT_SIZE)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center),
	)
	.width(LARGE_TEXT_SIZE * 1.8)
}

fn icon_label_button(
	label: impl text::IntoFragment<'static>,
	icon: Bootstrap,
) -> Button<'static, Message> {
	button(
		row![icon_to_text(icon).size(ICON_FONT_SIZE), text(label)]
			.align_y(Alignment::Center)
			.spacing(SMALL_SPACING_AMOUNT),
	)
}

pub fn create_new_project_button(enabled: bool) -> Button<'static, Message> {
	large_icon_button(Bootstrap::PlusLg)
		.on_press_maybe(if enabled {
			Some(SidebarPageMessage::OpenCreateNewProject.into())
		} else {
			None
		})
		.style(primary_button_style)
}

pub fn open_create_task_modal_button() -> Button<'static, Message> {
	large_icon_button(Bootstrap::PlusLg)
		.on_press(Message::OpenCreateTaskModal)
		.style(circle_button_style)
}

pub fn create_new_task_modal_button() -> Button<'static, CreateTaskModalMessage> {
	button(text("Create").align_x(Horizontal::Center))
		.on_press(CreateTaskModalMessage::CreateTask)
		.style(create_task_modal_ok_button_style)
}

pub fn close_create_new_task_modal_button() -> Button<'static, CreateTaskModalMessage> {
	button(text("Cancel").align_x(Horizontal::Center))
		.on_press(CreateTaskModalMessage::Close)
		.style(secondary_button_style_default)
}

pub fn cancel_create_project_button() -> Button<'static, Message> {
	icon_button(Bootstrap::XLg)
		.on_press(SidebarPageMessage::CloseCreateNewProject.into())
		.style(secondary_button_style_default)
}

fn delete_project_button() -> Button<'static, Message> {
	icon_label_button("Delete", Bootstrap::Trash)
		.width(Fill)
		.on_press(ProjectPageMessage::ConfirmDeleteProject.into())
		.style(move |t, s| delete_button_style(t, s, false, false, true, true))
}

pub fn project_context_menu_button(opened: bool) -> Element<'static, Message> {
	DropDown::new(
		icon_button(Bootstrap::ThreeDotsVertical)
			.on_press(if opened {
				ProjectPageMessage::HideContextMenu.into()
			}
			else {
				ProjectPageMessage::ShowContextMenu.into()
			})
			.style(secondary_button_style_default),

		container(
			column![
				manage_task_tags_button(),
				import_source_code_todos_button(),
				delete_project_button(),
			]
			.width(Length::Fixed(150.0))
		)
		.style(dropdown_container_style),

		opened
	)
	.width(Fill)
	.alignment(drop_down::Alignment::BottomStart)
	.offset(Offset::new(-ICON_BUTTON_WIDTH, ICON_BUTTON_WIDTH))
	.on_dismiss(ProjectPageMessage::HideContextMenu.into())
	.into()
}

pub fn edit_task_description_button(editing: bool) -> Element<'static, Message> {
	tooltip(
		icon_button(Bootstrap::PencilSquare)
			.on_press(TaskModalMessage::EditDescription.into())
			.style(move |t, s| selection_list_button_style(t, s, editing, false, true)),

		text("Edit description").size(SMALL_TEXT_SIZE),

		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn view_task_description_button(viewing: bool) -> Element<'static, Message> {
	tooltip(
		icon_button(Bootstrap::BodyText)
			.on_press(TaskModalMessage::ViewDescription.into())
			.style(move |t, s| selection_list_button_style(t, s, viewing, true, false)),

		text("View description").size(SMALL_TEXT_SIZE),

		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn delete_task_button() -> Button<'static, Message> {
	icon_label_button("Delete", Bootstrap::Trash)
		.on_press(TaskModalMessage::DeleteTask.into())
		.style(move |t, s| delete_button_style(t, s, true, true, true, true))
}

pub fn delete_all_done_tasks_button(
	project_id: ProjectId,
	project_name: &str,
) -> Button<'static, Message> {
	button(
		row![icon_to_text(Bootstrap::Trash), text("Delete done tasks")]
			.spacing(SMALL_SPACING_AMOUNT),
	)
	.on_press(ConfirmModalMessage::open(
		format!("Delete all done tasks of project '{project_name}'?"),
		DatabaseMessage::DeleteDoneTasks(project_id),
	))
	.style(delete_done_tasks_button_style)
}

pub fn show_done_tasks_button(show: bool, done_task_len: usize) -> Button<'static, Message> {
	icon_label_button(
		format!(
			"{} done ({done_task_len})",
			if show { "Hide" } else { "Show" }
		),
		if show {
			Bootstrap::CaretDownFill
		} else {
			Bootstrap::CaretRightFill
		},
	)
	.on_press(ProjectPageMessage::ShowDoneTasks(!show).into())
	.style(secondary_button_style_default)
}

pub fn dangerous_button(
	icon: Bootstrap,
	text: &'static str,
	confirm_label: Option<String>,
	on_press: impl Into<Message>,
) -> Button<'static, Message> {
	icon_label_button(text, icon)
		.style(dangerous_button_style)
		.on_press(if let Some(label) = confirm_label {
			ConfirmModalMessage::open(label, on_press)
		} else {
			on_press.into()
		})
}

pub fn theme_mode_button(
	theme_mode: ThemeMode,
	current_theme_mode: ThemeMode,
	round_left: bool,
	round_right: bool,
) -> Button<'static, Message> {
	button(text(format!("{:?}", theme_mode)).align_x(Horizontal::Center))
		.style(move |t, s| {
			selection_list_button_style(
				t,
				s,
				theme_mode == current_theme_mode,
				round_left,
				round_right,
			)
		})
		.width(80.0)
		.on_press(PreferenceMessage::SetThemeMode(theme_mode).into())
}

pub fn stopwatch_button(
	stopwatch_page: &StopwatchPage,
	selected: bool,
	dropzone_highlight: bool
) -> Element<'static, Message> {
	let stopwatch_label = match stopwatch_page {
		StopwatchPage::Ticking {
			clock,
			elapsed_time,
			..
		} => Some(if clock.label().is_empty() {
			format_stopwatch_duration(elapsed_time.as_secs_f64().round_ties_even() as i64)
		} else {
			clock.label().to_string()
		}),
		_ => None,
	};

	let stopwatch_ticking = matches!(stopwatch_page, StopwatchPage::Ticking { .. });

	container(
		button(
			row![
				icon_to_text(Bootstrap::Stopwatch).size(LARGE_TEXT_SIZE),
				text("Stopwatch").size(LARGE_TEXT_SIZE)
			]
			.push_maybe(stopwatch_label.map(|stopwatch_label| {
				container(text(stopwatch_label).size(SMALL_TEXT_SIZE))
					.width(Fill)
					.align_x(Horizontal::Right)
			}))
			.width(Fill)
			.spacing(SPACING_AMOUNT)
			.align_y(Alignment::Center)
			.padding(SMALL_HORIZONTAL_PADDING),
		)
		.width(Fill)
		.on_press(Message::OpenStopwatch)
		.style(move |t, s| {
			stopwatch_page_button_style(
				t,
				s,
				selected,
				stopwatch_ticking,
				dropzone_highlight
			)
		})
	)
	.id(STOPWATCH_TASK_DROPZONE_ID.clone())
	.into()
}

pub fn settings_button() -> Button<'static, Message> {
	large_icon_button(Bootstrap::Gear)
		.on_press(SettingsModalMessage::Open.into())
		.style(secondary_button_style_default)
}

pub fn select_synchronization_filepath_button() -> Button<'static, Message> {
	icon_label_button("Select file", Bootstrap::Folder)
		.on_press(SettingsModalMessage::BrowseSynchronizationFilepath.into())
		.style(secondary_button_style_default)
}

pub fn date_formatting_button<'a>(
	format: &'a DateFormatting,
	selected_format: &'a DateFormatting,
	is_left: bool,
) -> Button<'a, Message> {
	button(text(format.as_str()).align_x(Horizontal::Center))
		.width(120.0)
		.on_press(SettingsModalMessage::SetDateFormatting(*format).into())
		.style(move |t, s| {
			selection_list_button_style(t, s, *selected_format == *format, is_left, !is_left)
		})
}

pub fn copy_to_clipboard_button(copied_text: String) -> Element<'static, Message> {
	tooltip(
		icon_button(Bootstrap::Clipboard)
			.on_press(Message::CopyToClipboard(copied_text))
			.style(secondary_button_style_default),
		text("Copy to clipboard").size(SMALL_TEXT_SIZE),
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn toggle_sidebar_button(round_all_sides: bool) -> Element<'static, Message> {
	tooltip(
		icon_button(Bootstrap::LayoutSidebar)
			.on_press(Message::ToggleSidebar)
			.style(move |t, s| {
				secondary_button_style(
					t,
					s,
					round_all_sides,
					round_all_sides,
					round_all_sides,
					true,
				)
			}),
		text("Toggle sidebar (Ctrl + B)").size(SMALL_TEXT_SIZE),
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn import_database_button(importing: bool) -> Element<'static, Message> {
	button(
		row![
			if importing {
				Element::new(
					Spinner::new()
						.width(Length::Fixed(16.0))
						.height(Length::Fixed(16.0))
						.circle_radius(2.0),
				)
			} else {
				icon_to_text(Bootstrap::Download)
					.align_y(Vertical::Center)
					.into()
			},
			text("Import")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_y(Alignment::Center),
	)
	.on_press_maybe(if importing {
		None
	} else {
		Some(Message::ImportDatabaseDialog)
	})
	.style(dangerous_button_style)
	.into()
}

pub fn export_database_button(importing: bool) -> Element<'static, Message> {
	button(
		row![
			if importing {
				Element::new(
					Spinner::new()
						.width(Length::Fixed(16.0))
						.height(Length::Fixed(16.0))
						.circle_radius(2.0),
				)
			} else {
				icon_to_text(Bootstrap::Upload)
					.align_y(Vertical::Center)
					.into()
			},
			text("Export")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_y(Alignment::Center),
	)
	.on_press_maybe(if importing {
		None
	} else {
		Some(Message::ExportDatabaseDialog)
	})
	.style(dangerous_button_style)
	.into()
}

pub fn sync_database_button(
	synchronizing: bool,
	synchronization_filepath: Option<PathBuf>,
) -> Element<'static, Message> {
	button(
		row![
			if synchronizing {
				Element::new(
					Spinner::new()
						.width(Length::Fixed(16.0))
						.height(Length::Fixed(16.0))
						.circle_radius(2.0),
				)
			} else {
				icon_to_text(Bootstrap::ArrowClockwise)
					.align_y(Vertical::Center)
					.into()
			},
			text("Synchronize")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_y(Alignment::Center),
	)
	.on_press_maybe(synchronization_filepath.map(Message::SyncDatabase))
	.style(dangerous_button_style)
	.into()
}

pub fn sync_database_from_server_button(downloading: bool) -> Button<'static, Message> {
	button(
		row![
			if downloading {
				Element::new(
					Spinner::new()
						.width(Length::Fixed(16.0))
						.height(Length::Fixed(16.0))
						.circle_radius(2.0),
				)
			} else {
				icon_to_text(Bootstrap::ArrowClockwise)
					.align_y(Vertical::Center)
					.into()
			},
			text("Synchronize")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_y(Alignment::Center)
	)
	.on_press(Message::SyncDatabaseFromServer)
	.style(dangerous_button_style)
}

pub fn task_tag_button<Message>(
	task_tag: &TaskTag,
	toggled: bool,
) -> Button<Message> {
	let button = button(text(&task_tag.name)).style(move |t, s| {
		task_tag_button_style(t, s, task_tag.color.into(), toggled)
	});

	button
}

fn manage_task_tags_button() -> Element<'static, Message> {
	tooltip(
		icon_label_button("Manage Tags", Bootstrap::Bookmark)
			.width(Fill)
			.on_press(ProjectPageMessage::OpenManageTaskTagsModal.into())
			.style(secondary_button_style_only_round_top),
		text("Manage tags").size(SMALL_TEXT_SIZE),
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn create_new_task_tags_button() -> Button<'static, Message> {
	icon_label_button("Create new", Bootstrap::BookmarkPlusFill)
		.on_press(ManageTaskTagsModalMessage::OpenCreateNewTaskTag.into())
		.style(primary_button_style)
}

pub fn cancel_create_new_task_tag_button() -> Button<'static, Message> {
	icon_button(Bootstrap::XLg)
		.on_press(ManageTaskTagsModalMessage::CloseCreateNewTaskTag.into())
		.style(secondary_button_style_only_round_right)
}

pub fn delete_task_tag_button(task_tag_id: TaskTagId) -> Button<'static, Message> {
	icon_button(Bootstrap::Trash)
		.on_press(ManageTaskTagsModalMessage::DeleteTaskTag(task_tag_id).into())
		.style(move |t, s| delete_button_style(t, s, true, true, true, true))
}

pub fn clear_task_needed_time_button() -> Button<'static, Message> {
	icon_button(Bootstrap::XLg)
		.on_press(TaskModalMessage::ClearTaskNeededTime.into())
		.style(secondary_button_style_only_round_right)
}

pub fn edit_task_needed_time_button(needed_time_minutes: Option<usize>) -> Button<'static, Message> {
	button(
		if let Some(needed_time_minutes) = needed_time_minutes
		{
			duration_text(Cow::Owned(Duration::from_secs(
				needed_time_minutes as u64 * 60
			)))
		} else {
			text("Add needed time")
		}
	)
	.on_press(TaskModalMessage::EditNeededTime.into())
	.style(secondary_button_style_default)
}

pub fn clear_task_due_date_button(project_id: ProjectId, task_id: TaskId) -> Button<'static, Message> {
	icon_button(Bootstrap::XLg)
		.on_press(DatabaseMessage::ChangeTaskDueDate {
			project_id,
			task_id,
			new_due_date: None
		}.into())
		.style(secondary_button_style_only_round_right)
}

pub fn add_due_date_button() -> Button<'static, Message> {
	button(
		row![icon_to_text(Bootstrap::CalendarCheck), text("Add due date")]
			.spacing(SMALL_SPACING_AMOUNT),
	)
	.on_press(TaskModalMessage::EditDueDate.into())
	.style(secondary_button_style_default)
}

pub fn edit_due_date_button(
	due_date: &SerializableDate,
	date_formatting: DateFormatting,
) -> Button<'static, Message> {
	button(
		row![
			icon_to_text(Bootstrap::CalendarCheck),
			date_text(due_date, date_formatting)
		]
		.spacing(SMALL_SPACING_AMOUNT),
	)
	.on_press(TaskModalMessage::EditDueDate.into())
	.style(secondary_button_style_only_round_left)
}

pub fn edit_color_palette_button(
	color: Color,
	editing: bool,
	on_press: Message,
) -> Element<'static, Message> {
	tooltip(
		color_palette_item_button(color, editing, true, true, on_press),
		text("Edit color").size(SMALL_TEXT_SIZE),
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn color_palette_item_button(
	color: Color,
	selected: bool,
	round_left: bool,
	round_right: bool,
	on_press: Message,
) -> Button<'static, Message> {
	button(Quad {
		width: Length::Fixed(25.0),
		height: Length::Fixed(25.0),
		inner_bounds: InnerBounds::Ratio(0.8, 0.8),
		quad_color: color.into(),
		quad_border: rounded(f32::MAX),
		bg_color: None,
		..Default::default()
	})
	.on_press(on_press)
	.style(move |t, s| selection_list_button_style(t, s, selected, round_left, round_right))
	.padding(SMALL_PADDING_AMOUNT)
}

pub fn confirm_ok_button(on_confirmed: &Message) -> Button<'static, Message> {
	button(text("Ok").align_x(Horizontal::Center))
		.width(Fill)
		.style(dangerous_button_style)
		.on_press(Message::ConfirmModalConfirmed(Box::new(
			on_confirmed.clone(),
		)))
}

pub fn confirm_cancel_button() -> Button<'static, Message> {
	button(text("Cancel").align_x(Horizontal::Center))
		.width(Fill)
		.style(secondary_button_style_default)
		.on_press(ConfirmModalMessage::Close.into())
}

pub fn search_tasks_button() -> Button<'static, Message> {
	icon_button(Bootstrap::Search)
		.style(secondary_button_style_default)
		.on_press(ProjectPageMessage::OpenSearchTasks.into())
}

pub fn cancel_search_tasks_button() -> Button<'static, Message> {
	icon_button(Bootstrap::XLg)
		.style(secondary_button_style_only_round_right)
		.on_press(ProjectPageMessage::CloseSearchTasks.into())
}

pub fn settings_tab_button(
	tab: SettingTab,
	selected_tab: SettingTab,
) -> Button<'static, Message> {
	button(text(format!("{tab:?}")))
		.width(Fill)
		.style(move |t, s| settings_tab_button_style(t, s, tab == selected_tab))
		.on_press(SettingsModalMessage::SwitchSettingsTab(tab).into())
}

fn import_source_code_todos_button() -> Element<'static, Message> {
	tooltip(
		icon_label_button("Import Todos", Bootstrap::FileEarmarkCode)
			.width(Fill)
			.on_press(ProjectPageMessage::ImportSourceCodeTodosDialog.into())
			.style(secondary_button_style_no_rounding),
		text("Import TODO's").size(SMALL_TEXT_SIZE),
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn reimport_source_code_todos_button() -> Button<'static, Message> {
	icon_label_button("Reimport TODO's", Bootstrap::FileEarmarkCode)
		.on_press(ProjectPageMessage::ImportSourceCodeTodosDialog.into())
		.style(secondary_button_style_default)
}

pub fn show_source_code_todos_button(
	show: bool,
	source_code_todos_len: usize,
) -> Button<'static, Message> {
	icon_label_button(
		format!(
			"{} source code todos ({source_code_todos_len})",
			if show { "Hide" } else { "Show" }
		),
		if show {
			Bootstrap::CaretDownFill
		} else {
			Bootstrap::CaretRightFill
		},
	)
	.on_press(ProjectPageMessage::ShowSourceCodeTodos(!show).into())
	.style(secondary_button_style_default)
}

pub fn start_timer_button() -> Button<'static, Message> {
	button(
		icon_to_text(Bootstrap::PlayFill)
			.size(45)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center),
	)
	.width(Length::Fixed(1.75 * 45.0))
	.height(Length::Fixed(1.75 * 45.0))
	.on_press(StopwatchPageMessage::Start { task: None }.into())
	.style(move |t, s| timer_button_style(t, s, false))
}

pub fn stop_timer_button() -> Button<'static, Message> {
	button(
		icon_to_text(Bootstrap::X)
			.size(45)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center),
	)
	.width(Length::Fixed(1.75 * 45.0))
	.height(Length::Fixed(1.75 * 45.0))
	.on_press(StopwatchPageMessage::Stop.into())
	.style(move |t, s| timer_button_style(t, s, true))
}

pub fn resume_timer_button() -> Button<'static, Message> {
	button(
		icon_to_text(Bootstrap::PlayFill)
			.size(45)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center),
	)
	.width(Length::Fixed(1.75 * 45.0))
	.height(Length::Fixed(1.75 * 45.0))
	.on_press(StopwatchPageMessage::Resume.into())
	.style(move |t, s| timer_button_style(t, s, true))
}

pub fn pause_timer_button() -> Button<'static, Message> {
	button(
		icon_to_text(Bootstrap::PauseFill)
			.size(45)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center),
	)
	.width(Length::Fixed(1.75 * 45.0))
	.height(Length::Fixed(1.75 * 45.0))
	.on_press(StopwatchPageMessage::Pause.into())
	.style(move |t, s| timer_button_style(t, s, true))
}

pub fn complete_task_timer_button() -> Button<'static, Message> {
	button(
		icon_to_text(Bootstrap::CheckLg)
			.size(45)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center),
	)
	.width(Length::Fixed(1.75 * 45.0))
	.height(Length::Fixed(1.75 * 45.0))
	.on_press(StopwatchPageMessage::CompleteTask.into())
	.style(move |t, s| timer_button_style(t, s, true))
}

pub fn start_task_timer_button<'a>(
	project_id: ProjectId,
	task_id: TaskId
) -> Element<'a, Message> {
	tooltip(
		icon_button(Bootstrap::Stopwatch)
			.on_press(
				StopwatchPageMessage::Start {
					task: Some((project_id, task_id)),
				}
				.into(),
			)
			.style(secondary_button_style_default),
		text("Start a timer for this task"),
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn import_google_tasks_button() -> Button<'static, Message> {
	button("Import")
		.on_press(SettingsModalMessage::ImportGoogleTasksFileDialog.into())
		.style(dangerous_button_style)
}

pub fn open_link_button(url: String) -> Element<'static, Message> {
	tooltip(
		icon_button(Bootstrap::BoxArrowUpRight)
			.on_press(Message::OpenUrl(url))
			.style(secondary_button_style_default),

		text("Open link").size(SMALL_TEXT_SIZE),

		tooltip::Position::Bottom
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn sort_dropdown_button(opened: bool, sort_mode: SortMode) -> Element<'static, Message> {
	DropDown::new(
		button(
			row![
				icon_to_text(if opened {
					Bootstrap::CaretDownFill
				} else {
					Bootstrap::CaretRightFill
				})
				.size(ICON_FONT_SIZE),
				text("Sort:"),
				icon_to_text(sort_mode.icon()).size(ICON_FONT_SIZE),
				text(sort_mode.as_str()),
			]
			.spacing(SMALL_SPACING_AMOUNT)
			.align_y(Vertical::Center)
		)
		.on_press(if opened {
			ProjectPageMessage::CloseSortModeDropdown.into()
		}
		else {
			ProjectPageMessage::OpenSortModeDropdown.into()
		})
		.style(secondary_button_style_default),

		container(
			Column::with_children(SortMode::ALL.iter().enumerate().map(|(i, mode)| {
				icon_label_button(
					mode.as_str(),
					mode.icon()
				)
				.width(Fill)
				.style(move |t, s| enum_dropdown_button_style(
					t,
					s,
					sort_mode == *mode,
					i == 0,
					i == SortMode::ALL.len() - 1
				))
				.on_press(ProjectPageMessage::SetSortMode(*mode).into())
				.into()
			}))
		)
		.style(dropdown_container_style),

		opened
	)
	.width(Fixed(140.0))
	.alignment(drop_down::Alignment::Bottom)
	.offset(0.0)
	.on_dismiss(ProjectPageMessage::CloseSortModeDropdown.into())
	.into()
}

pub fn synchronization_type_button(synchronization_setting: SynchronizationSetting, selected_setting: &SynchronizationSetting, round_left: bool, round_right: bool) -> Button<'static, Message> {
	let selected = synchronization_setting.is_same_type(selected_setting);
	let name: &'static str = synchronization_setting.as_str();

	button(text(name).align_x(Horizontal::Center))
		.style(move |t, s| {
			selection_list_button_style(
				t,
				s,
				selected,
				round_left,
				round_right,
			)
		})
		.width(80.0)
		.on_press(PreferenceMessage::SetSynchronization(Some(synchronization_setting)).into())
}

pub fn show_password_button() -> Element<'static, Message> {
	tooltip(
		icon_button(Bootstrap::EyeFill)
			.on_press(SettingsModalMessage::ShowPassword.into())
			.style(secondary_button_style_default),

		text("Show password").size(SMALL_TEXT_SIZE),

		tooltip::Position::Bottom
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn hide_password_button() -> Element<'static, Message> {
	tooltip(
		icon_button(Bootstrap::EyeSlashFill)
			.on_press(SettingsModalMessage::HidePassword.into())
			.style(secondary_button_style_default),

		text("Hide password").size(SMALL_TEXT_SIZE),

		tooltip::Position::Bottom
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}