use crate::{
	components::{self, date_text, duration_text},
	core::{IcedColorConversion, SerializableDateConversion, SortModeUI},
	icons::{icon_to_text, Bootstrap},
	integrations::CodeEditor,
	modals::{
		confirm_modal, create_task_modal, error_msg_modal, manage_task_tags_modal, settings_modal,
		task_modal, wait_closing_modal,
	},
	pages::{
		self, format_stopwatch_duration,
		overview_page::{self, CalendarView},
		project_page, sidebar_page, stopwatch_page, STOPWATCH_TASK_DROPZONE_ID,
	},
	preferences::{FirstWeekday, SerializedOverviewPage},
	project_tracker::Message,
	styles::{
		circle_button_style, danger_text_style, dangerous_button_style, delete_button_style,
		delete_done_tasks_button_style, dropdown_container_style, enum_dropdown_button_style,
		hidden_secondary_button_style, overview_button_style, primary_button_style,
		secondary_button_style, secondary_button_style_default, secondary_button_style_no_rounding,
		secondary_button_style_only_round_left, secondary_button_style_only_round_right,
		secondary_button_style_only_round_top, selection_list_button_style,
		settings_tab_button_style, stopwatch_page_button_style, task_tag_button_style,
		text_input_style, timer_button_style, tooltip_container_style, BOLD_FONT, GAP,
		HEADING_TEXT_SIZE, JET_BRAINS_MONO_FONT, LARGE_TEXT_SIZE, SMALL_HORIZONTAL_PADDING,
		SMALL_PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SMALL_TEXT_SIZE, SPACING_AMOUNT,
	},
	theme_mode::ThemeMode,
	DateFormatting, PreferenceMessage,
};
use iced::{
	alignment::{Horizontal, Vertical},
	border::rounded,
	widget::{
		button, column, container, rich_text, row, text, text::Span, text_input, tooltip, Button,
		Column, Space,
	},
	Alignment, Color, Element,
	Length::{self, Fill, Fixed},
};
use iced_aw::{drop_down, drop_down::Offset, quad::Quad, widgets::InnerBounds, DropDown, Spinner};
use iced_date_picker::{date_picker, Date};
use project_tracker_core::{
	Database, DatabaseMessage, ProjectId, SerializableDate, SortMode, TaskId, TaskTag, TaskTagId,
};
use std::{path::PathBuf, time::Duration};

pub const ICON_FONT_SIZE: f32 = 16.0;
pub const ICON_BUTTON_WIDTH: f32 = ICON_FONT_SIZE * 1.8;
pub const LARGE_ICON_BUTTON_WIDTH: f32 = LARGE_TEXT_SIZE * 1.8;
const SETTINGS_SELECTION_LIST_WIDTH: f32 = 280.0;

fn icon_button<Message>(icon: Bootstrap) -> Button<'static, Message> {
	button(
		icon_to_text(icon)
			.size(ICON_FONT_SIZE)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center),
	)
	.width(ICON_BUTTON_WIDTH)
}

fn large_icon_button<Message: 'static>(icon: Bootstrap) -> Button<'static, Message> {
	button(
		icon_to_text(icon)
			.size(LARGE_TEXT_SIZE)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center),
	)
	.width(LARGE_ICON_BUTTON_WIDTH)
}

fn icon_label_button<Message: 'static>(
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
			Some(sidebar_page::Message::OpenCreateNewProject.into())
		} else {
			None
		})
		.style(primary_button_style)
}

pub fn open_create_task_modal_button() -> Button<'static, Message> {
	large_icon_button(Bootstrap::PlusLg)
		.on_press(Message::OpenCreateTaskModalCurrent)
		.style(circle_button_style)
}

pub fn create_new_task_modal_button() -> Button<'static, Message> {
	button(text("Create").align_x(Horizontal::Center))
		.on_press(create_task_modal::Message::CreateTask.into())
		.style(primary_button_style)
}

pub fn close_create_new_task_modal_button() -> Button<'static, Message> {
	button(text("Cancel").align_x(Horizontal::Center))
		.on_press(Message::CloseCreateTaskModal)
		.style(secondary_button_style_default)
}

pub fn cancel_create_project_button() -> Button<'static, Message> {
	icon_button(Bootstrap::XLg)
		.on_press(sidebar_page::Message::CloseCreateNewProject.into())
		.style(secondary_button_style_default)
}

fn delete_project_button() -> Button<'static, Message> {
	icon_label_button("Delete", Bootstrap::Trash)
		.width(Fill)
		.on_press(project_page::Message::ConfirmDeleteProject.into())
		.style(move |t, s| delete_button_style(t, s, false, false, true, true))
}

pub fn project_context_menu_button(opened: bool) -> Element<'static, Message> {
	DropDown::new(
		icon_button(Bootstrap::ThreeDotsVertical)
			.on_press(if opened {
				project_page::Message::HideContextMenu.into()
			} else {
				project_page::Message::ShowContextMenu.into()
			})
			.style(secondary_button_style_default),
		container(
			column![
				manage_task_tags_button(),
				import_source_code_todos_button(),
				delete_project_button(),
			]
			.width(Length::Fixed(150.0)),
		)
		.style(dropdown_container_style),
		opened,
	)
	.width(Fill)
	.alignment(drop_down::Alignment::BottomStart)
	.offset(Offset::new(-ICON_BUTTON_WIDTH, ICON_BUTTON_WIDTH))
	.on_dismiss(project_page::Message::HideContextMenu.into())
	.into()
}

/// !viewing => editing
pub fn toggle_view_edit_task_description_button(viewing: bool) -> Element<'static, Message> {
	tooltip(
		if viewing {
			icon_button(Bootstrap::Pencil)
				.on_press(task_modal::Message::EditDescription.into())
				.style(primary_button_style)
		} else {
			icon_button(Bootstrap::Book)
				.on_press(task_modal::Message::ViewDescription.into())
				.style(primary_button_style)
		},
		if viewing {
			text("Edit").size(SMALL_TEXT_SIZE)
		} else {
			text("View").size(SMALL_TEXT_SIZE)
		},
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn delete_task_button() -> Button<'static, Message> {
	icon_label_button("Delete", Bootstrap::Trash)
		.on_press(task_modal::Message::DeleteTask.into())
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
	.on_press(confirm_modal::Message::open(
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
	.on_press(project_page::Message::ShowDoneTasks(!show).into())
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
		.on_press(match confirm_label {
			Some(label) => confirm_modal::Message::open(label, on_press),
			None => on_press.into(),
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
				round_left,
				round_right,
			)
		})
		.width(SETTINGS_SELECTION_LIST_WIDTH / 3.0)
		.on_press(PreferenceMessage::SetThemeMode(theme_mode).into())
}

pub fn overview_button(selected: bool) -> Button<'static, Message> {
	button(
		row![
			icon_to_text(Bootstrap::List).size(LARGE_TEXT_SIZE),
			text("Overview").size(LARGE_TEXT_SIZE),
		]
		.width(Fill)
		.spacing(SPACING_AMOUNT)
		.align_y(Alignment::Center),
	)
	.width(Fill)
	.on_press(pages::Message::OpenOverview.into())
	.style(move |t, s| overview_button_style(t, s, selected))
}

pub fn stopwatch_button(
	stopwatch_page: &stopwatch_page::Page,
	selected: bool,
	dropzone_highlight: bool,
	database: Option<&Database>,
) -> Element<'static, Message> {
	let stopwatch_label = match stopwatch_page {
		stopwatch_page::Page::TakingBreak { clock, .. } => Some(clock.label().to_string()),
		stopwatch_page::Page::StopTaskTime {
			clock,
			project_id,
			task_id,
			..
		} => Some(match clock {
			Some(clock) => clock.label().to_string(),
			None => format_stopwatch_duration(
				stopwatch_page::Page::get_spend_seconds(*project_id, *task_id, database)
					.unwrap_or(0.0)
					.round_ties_even() as i64,
			),
		}),
		stopwatch_page::Page::TrackTime { elapsed_time, .. } => Some(format_stopwatch_duration(
			elapsed_time.as_secs_f64().round_ties_even() as i64,
		)),
		stopwatch_page::Page::Idle => None,
	};

	// TODO: different colors for break, task, tracking time
	let stopwatch_ticking = !matches!(stopwatch_page, stopwatch_page::Page::Idle);

	let stopwatch_icon = match stopwatch_page {
		stopwatch_page::Page::TakingBreak { .. } => Bootstrap::CupHot,
		stopwatch_page::Page::StopTaskTime { .. } => Bootstrap::HourglassSplit,
		_ => Bootstrap::Stopwatch,
	};

	container(
		button(
			row![
				icon_to_text(stopwatch_icon).size(LARGE_TEXT_SIZE),
				text("Stopwatch").size(LARGE_TEXT_SIZE)
			]
			.push_maybe(stopwatch_label.map(|stopwatch_label| {
				container(
					text(stopwatch_label)
						.size(SMALL_TEXT_SIZE)
						.font(JET_BRAINS_MONO_FONT),
				)
				.width(Fill)
				.align_x(Horizontal::Right)
			}))
			.width(Fill)
			.spacing(SPACING_AMOUNT)
			.align_y(Alignment::Center),
		)
		.width(Fill)
		.on_press(pages::Message::OpenStopwatch.into())
		.style(move |t, s| {
			stopwatch_page_button_style(t, s, selected, stopwatch_ticking, dropzone_highlight)
		}),
	)
	.id(STOPWATCH_TASK_DROPZONE_ID.clone())
	.into()
}

pub fn settings_button() -> Button<'static, Message> {
	large_icon_button(Bootstrap::Gear)
		.on_press(settings_modal::Message::Open.into())
		.style(secondary_button_style_default)
}

pub fn date_formatting_button<'a>(
	format: &'a DateFormatting,
	selected_format: &'a DateFormatting,
	is_left: bool,
) -> Button<'a, Message> {
	button(text(format.as_str()).align_x(Horizontal::Center))
		.width(SETTINGS_SELECTION_LIST_WIDTH / 2.0)
		.on_press(settings_modal::Message::SetDateFormatting(*format).into())
		.style(move |t, s| {
			selection_list_button_style(
				t,
				s,
				*selected_format == *format,
				is_left,
				!is_left,
				is_left,
				!is_left,
			)
		})
}

pub fn first_weekday_button<'a>(
	first_weekday: &'a FirstWeekday,
	selected_first_weekday: &'a FirstWeekday,
	is_left: bool,
) -> Button<'a, Message> {
	button(text!("{first_weekday:?}").align_x(Horizontal::Center))
		.width(SETTINGS_SELECTION_LIST_WIDTH / 2.0)
		.on_press(PreferenceMessage::SetFirstWeekday(*first_weekday).into())
		.style(move |t, s| {
			selection_list_button_style(
				t,
				s,
				*selected_first_weekday == *first_weekday,
				is_left,
				!is_left,
				is_left,
				!is_left,
			)
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

pub fn create_empty_database_button() -> Element<'static, Message> {
	button(text("Create new database"))
		.style(dangerous_button_style)
		.on_press(Message::DatabaseImported(Ok(Database::default())))
		.into()
}

pub fn import_database_button(importing: bool) -> Element<'static, Message> {
	button(
		row![
			if importing {
				Element::new(
					Spinner::new()
						.width(Length::Fixed(ICON_FONT_SIZE))
						.height(Length::Fixed(ICON_FONT_SIZE))
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

pub fn export_database_button(exporting: bool) -> Element<'static, Message> {
	button(
		row![
			if exporting {
				Element::new(
					Spinner::new()
						.width(Length::Fixed(ICON_FONT_SIZE))
						.height(Length::Fixed(ICON_FONT_SIZE))
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
	.on_press_maybe(if exporting {
		None
	} else {
		Some(Message::ExportDatabaseDialog)
	})
	.style(dangerous_button_style)
	.into()
}

pub fn import_json_database_button(importing_json: bool) -> Element<'static, Message> {
	button(
		row![
			if importing_json {
				Element::new(
					Spinner::new()
						.width(Length::Fixed(ICON_FONT_SIZE))
						.height(Length::Fixed(ICON_FONT_SIZE))
						.circle_radius(2.0),
				)
			} else {
				icon_to_text(Bootstrap::FiletypeJson)
					.align_y(Vertical::Center)
					.into()
			},
			text("Import Json")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_y(Alignment::Center),
	)
	.on_press_maybe(if importing_json {
		None
	} else {
		Some(Message::ImportJsonDatabaseDialog)
	})
	.style(dangerous_button_style)
	.into()
}

pub fn export_as_json_database_button(exporting_json: bool) -> Element<'static, Message> {
	button(
		row![
			if exporting_json {
				Element::new(
					Spinner::new()
						.width(Length::Fixed(ICON_FONT_SIZE))
						.height(Length::Fixed(ICON_FONT_SIZE))
						.circle_radius(2.0),
				)
			} else {
				icon_to_text(Bootstrap::FiletypeJson)
					.align_y(Vertical::Center)
					.into()
			},
			text("Export as Json")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_y(Alignment::Center),
	)
	.on_press_maybe(if exporting_json {
		None
	} else {
		Some(Message::ExportDatabaseAsJsonDialog)
	})
	.style(dangerous_button_style)
	.into()
}

pub fn export_database_as_markdown_button(exporting_json: bool) -> Element<'static, Message> {
	button(
		row![
			if exporting_json {
				Element::new(
					Spinner::new()
						.width(Length::Fixed(ICON_FONT_SIZE))
						.height(Length::Fixed(ICON_FONT_SIZE))
						.circle_radius(2.0),
				)
			} else {
				icon_to_text(Bootstrap::FiletypeMd)
					.align_y(Vertical::Center)
					.into()
			},
			text("Export as Markdown")
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_y(Alignment::Center),
	)
	.on_press_maybe(if exporting_json {
		None
	} else {
		Some(Message::ExportDatabaseAsMarkdownDialog)
	})
	.style(dangerous_button_style)
	.into()
}

pub fn task_tag_button<Message>(task_tag: &TaskTag, toggled: bool) -> Button<Message> {
	let button = button(text(&task_tag.name))
		.style(move |t, s| task_tag_button_style(t, s, task_tag.color.to_iced_color(), toggled));

	button
}

fn manage_task_tags_button() -> Element<'static, Message> {
	tooltip(
		icon_label_button("Manage Tags", Bootstrap::Bookmark)
			.width(Fill)
			.on_press(project_page::Message::OpenManageTaskTagsModal.into())
			.style(secondary_button_style_only_round_top),
		text("Manage tags").size(SMALL_TEXT_SIZE),
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn create_new_task_tags_button() -> Button<'static, manage_task_tags_modal::Message> {
	icon_label_button("Create new", Bootstrap::BookmarkPlusFill)
		.on_press(manage_task_tags_modal::Message::OpenCreateNewTaskTag)
		.style(primary_button_style)
}

pub fn cancel_create_new_task_tag_button() -> Button<'static, manage_task_tags_modal::Message> {
	icon_button(Bootstrap::XLg)
		.on_press(manage_task_tags_modal::Message::CloseCreateNewTaskTag)
		.style(secondary_button_style_only_round_right)
}

pub fn delete_task_tag_button(
	task_tag_id: TaskTagId,
) -> Button<'static, manage_task_tags_modal::Message> {
	icon_button(Bootstrap::Trash)
		.on_press(manage_task_tags_modal::Message::DeleteTaskTag(task_tag_id))
		.style(move |t, s| delete_button_style(t, s, true, true, true, true))
}

pub fn clear_task_needed_time_button<Message>(on_press: Message) -> Button<'static, Message> {
	icon_button(Bootstrap::XLg)
		.on_press(on_press)
		.style(secondary_button_style_only_round_right)
}

pub fn clear_task_due_date_button<Message>(on_press: Message) -> Button<'static, Message> {
	icon_button(Bootstrap::XLg)
		.on_press(on_press)
		.style(secondary_button_style_only_round_right)
}

pub fn add_due_date_button<Message: 'static>(on_press: Message) -> Button<'static, Message> {
	button(
		row![icon_to_text(Bootstrap::CalendarCheck), text("Add due date")]
			.spacing(SMALL_SPACING_AMOUNT),
	)
	.on_press(on_press)
	.style(secondary_button_style_default)
}

pub fn edit_due_date_button<Message: 'static>(
	due_date: &SerializableDate,
	date_formatting: DateFormatting,
	on_press: Message,
) -> Button<'static, Message> {
	button(
		row![
			icon_to_text(Bootstrap::CalendarCheck),
			date_text(due_date, date_formatting)
		]
		.spacing(SMALL_SPACING_AMOUNT),
	)
	.on_press(on_press)
	.style(secondary_button_style_only_round_left)
}

pub fn edit_color_palette_button(
	color: Color,
	editing: bool,
	on_press: Message,
) -> Element<'static, Message> {
	tooltip(
		color_palette_item_button(color, editing, true, true, true, true, on_press),
		text("Edit color").size(SMALL_TEXT_SIZE),
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn color_palette_item_button<Message: 'static>(
	color: Color,
	selected: bool,
	round_left_top: bool,
	round_right_top: bool,
	round_left_bottom: bool,
	round_right_bottom: bool,
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
	.style(move |t, s| {
		selection_list_button_style(
			t,
			s,
			selected,
			round_left_top,
			round_right_top,
			round_left_bottom,
			round_right_bottom,
		)
	})
	.padding(SMALL_PADDING_AMOUNT)
}

pub fn confirm_ok_button(
	on_confirmed: &Message,
	custom_label: Option<&'static str>,
) -> Button<'static, Message> {
	button(text(custom_label.unwrap_or("Ok")).align_x(Horizontal::Center))
		.width(Fill)
		.style(dangerous_button_style)
		.on_press(Message::ConfirmModalConfirmed(Box::new(
			on_confirmed.clone(),
		)))
}

pub fn confirm_cancel_button(custom_label: Option<&'static str>) -> Button<'static, Message> {
	button(text(custom_label.unwrap_or("Cancel")).align_x(Horizontal::Center))
		.width(Fill)
		.style(secondary_button_style_default)
		.on_press(confirm_modal::Message::Close.into())
}

pub fn search_tasks_button() -> Button<'static, Message> {
	icon_button(Bootstrap::Search)
		.style(secondary_button_style_default)
		.on_press(project_page::Message::OpenSearchTasks.into())
}

pub fn cancel_search_tasks_button() -> Button<'static, Message> {
	icon_button(Bootstrap::XLg)
		.style(secondary_button_style_only_round_right)
		.on_press(project_page::Message::CloseSearchTasks.into())
}

pub fn settings_tab_button(
	tab: settings_modal::SettingTab,
	selected_tab: settings_modal::SettingTab,
) -> Button<'static, Message> {
	icon_label_button(tab.label(), tab.icon())
		.width(Fill)
		.style(move |t, s| settings_tab_button_style(t, s, tab == selected_tab))
		.on_press(settings_modal::Message::OpenTab(tab).into())
}

fn import_source_code_todos_button() -> Button<'static, Message> {
	icon_label_button("Import Todos", Bootstrap::FileEarmarkCode)
		.width(Fill)
		.on_press(project_page::Message::ImportSourceCodeTodosDialog.into())
		.style(secondary_button_style_no_rounding)
}

pub fn reimport_source_code_todos_button(
	importing: bool,
	reimport_possible: bool,
) -> Button<'static, Message> {
	button(
		row![
			if importing {
				Element::new(
					Spinner::new()
						.width(Length::Fixed(ICON_FONT_SIZE))
						.height(Length::Fixed(ICON_FONT_SIZE))
						.circle_radius(2.0),
				)
			} else {
				icon_to_text(Bootstrap::FileEarmarkCode)
					.align_y(Vertical::Center)
					.into()
			},
			text(if reimport_possible {
				"Reimport TODO's"
			} else {
				"Import TODO's"
			})
		]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_y(Alignment::Center),
	)
	.on_press(if reimport_possible {
		project_page::Message::ReimportSourceCodeTodos.into()
	} else {
		project_page::Message::ImportSourceCodeTodosDialog.into()
	})
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
	.on_press(project_page::Message::ShowSourceCodeTodos(!show).into())
	.style(secondary_button_style_default)
}

pub fn track_time_button() -> Button<'static, Message> {
	button(
		icon_to_text(Bootstrap::PlayFill)
			.size(45)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center),
	)
	.width(Length::Fixed(1.75 * 45.0))
	.height(Length::Fixed(1.75 * 45.0))
	.on_press(stopwatch_page::Message::StartTrackingTime.into())
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
	.on_press(stopwatch_page::Message::Stop.into())
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
	.on_press(stopwatch_page::Message::Resume.into())
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
	.on_press(stopwatch_page::Message::Pause.into())
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
	.on_press(stopwatch_page::Message::CompleteTask.into())
	.style(move |t, s| timer_button_style(t, s, true))
}

pub fn start_task_timer_button<'a>(
	project_id: ProjectId,
	task_id: TaskId,
	stopping_task: bool,
) -> Element<'a, Message> {
	tooltip(
		icon_button(if stopping_task {
			Bootstrap::Pause
		} else {
			Bootstrap::Stopwatch
		})
		.on_press(
			if stopping_task {
				stopwatch_page::Message::Stop
			} else {
				stopwatch_page::Message::StopTask {
					project_id,
					task_id,
				}
			}
			.into(),
		)
		.style(secondary_button_style_default),
		if stopping_task {
			text("Stop working on this task")
		} else {
			text("Start a timer for this task")
		},
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn import_google_tasks_button() -> Button<'static, Message> {
	button("Import")
		.on_press(settings_modal::Message::ImportGoogleTasksFileDialog.into())
		.style(dangerous_button_style)
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
			.align_y(Vertical::Center),
		)
		.on_press(if opened {
			project_page::Message::CloseSortModeDropdown.into()
		} else {
			project_page::Message::OpenSortModeDropdown.into()
		})
		.style(secondary_button_style_default),
		container(Column::with_children(SortMode::ALL.iter().enumerate().map(
			|(i, mode)| {
				icon_label_button(mode.as_str(), mode.icon())
					.width(Fill)
					.style(move |t, s| {
						enum_dropdown_button_style(
							t,
							s,
							sort_mode == *mode,
							i == 0,
							i == SortMode::ALL.len() - 1,
						)
					})
					.on_press(project_page::Message::SetSortMode(*mode).into())
					.into()
			},
		)))
		.style(dropdown_container_style),
		opened,
	)
	.width(Fixed(160.0))
	.alignment(drop_down::Alignment::Bottom)
	.offset(0.0)
	.on_dismiss(project_page::Message::CloseSortModeDropdown.into())
	.into()
}

pub fn show_password_button() -> Element<'static, Message> {
	tooltip(
		icon_button(Bootstrap::EyeFill)
			.on_press(settings_modal::Message::ShowPassword.into())
			.style(secondary_button_style_default),
		text("Show password").size(SMALL_TEXT_SIZE),
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn hide_password_button() -> Element<'static, Message> {
	tooltip(
		icon_button(Bootstrap::EyeSlashFill)
			.on_press(settings_modal::Message::HidePassword.into())
			.style(secondary_button_style_default),
		text("Hide password").size(SMALL_TEXT_SIZE),
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn take_break_button(minutes: usize) -> Button<'static, Message> {
	button(text(format!("{minutes} min")).size(HEADING_TEXT_SIZE))
		.on_press(stopwatch_page::Message::TakeBreak(minutes).into())
		.style(move |t, s| timer_button_style(t, s, false))
}

pub fn open_folder_location_button(
	filepath: PathBuf,
	parent_filepath: Option<PathBuf>,
) -> Element<'static, Message> {
	tooltip(
		button(text(filepath.to_string_lossy().to_string()))
			.on_press_maybe(parent_filepath.map(Message::OpenFolderLocation))
			.padding(SMALL_HORIZONTAL_PADDING)
			.style(secondary_button_style_default),
		text("Open folder location").size(SMALL_TEXT_SIZE),
		tooltip::Position::Bottom,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn error_msg_ok_button() -> Button<'static, Message> {
	button(text("Ok").align_x(Horizontal::Center).width(Fill))
		.width(Fill)
		.style(dangerous_button_style)
		.on_press(error_msg_modal::Message::Close.into())
}

pub fn task_tag_name_button(
	task_tag_id: TaskTagId,
	task_tag_name: &str,
) -> Button<manage_task_tags_modal::Message> {
	button(text(task_tag_name).width(Fill))
		.on_press(manage_task_tags_modal::Message::EditTaskTagName(
			task_tag_id,
		))
		.style(hidden_secondary_button_style)
}

pub fn force_close_anyways_button() -> Button<'static, wait_closing_modal::Message> {
	button(text("Close anyways"))
		.on_press(wait_closing_modal::Message::ForceCloseAnyways)
		.style(dangerous_button_style)
}

pub fn open_project_button(
	project_id: ProjectId,
	project_name: &str,
	project_color: Color,
) -> Button<Message> {
	let project_link_text = Span::new(format!("{project_name}:"))
		.link(Message::ContentPageMessage(
			pages::Message::OpenProjectPage(project_id),
		))
		.underline(true)
		.color(project_color);

	button(rich_text![project_link_text])
		.style(hidden_secondary_button_style)
		.on_press(pages::Message::OpenProjectPage(project_id).into())
}

pub fn open_task_by_name_link_button(
	project_id: ProjectId,
	task_id: TaskId,
	task_name: &str,
) -> Button<Message> {
	let task_link_text = Span::new(task_name)
		.link(Message::OpenTaskModal {
			project_id,
			task_id,
		})
		.underline(true)
		.size(HEADING_TEXT_SIZE)
		.font(BOLD_FONT);

	button(rich_text![task_link_text])
		.style(hidden_secondary_button_style)
		.on_press(Message::OpenTaskModal {
			project_id,
			task_id,
		})
}

#[allow(clippy::too_many_arguments)]
pub fn edit_needed_time_button<'a, Message: 'static + Clone>(
	task_needed_time_minutes: Option<usize>,
	new_needed_time_minutes: &'a Option<String>,
	on_edit: Message,
	on_input: impl Fn(String) -> Message + 'a,
	on_submit: Option<Message>,
	stop_editing: Message,
	clear_needed_time: Message,
	text_input_id: text_input::Id,
) -> Element<'a, Message> {
	match new_needed_time_minutes {
		Some(new_needed_time_minutes) => {
			let edit_needed_time_element = components::on_input(
				text_input("ex: 30min", new_needed_time_minutes)
					.id(text_input_id)
					.width(Fixed(80.0))
					.on_input(on_input)
					.on_submit_maybe(on_submit)
					.style(move |t, s| text_input_style(t, s, true, false, false, true)),
			)
			.on_esc(stop_editing);

			row![
				edit_needed_time_element,
				clear_task_needed_time_button(clear_needed_time)
			]
			.into()
		}
		None => button(match task_needed_time_minutes {
			Some(needed_time_minutes) => {
				duration_text(Duration::from_secs(needed_time_minutes as u64 * 60))
			}
			None => text("Add needed time"),
		})
		.on_press(on_edit)
		.style(secondary_button_style_default)
		.into(),
	}
}

pub fn due_date_button<Message: 'static + Clone>(
	edit_due_date: bool,
	due_date: &Option<SerializableDate>,
	date_formatting: DateFormatting,
	on_edit: Message,
	stop_editing: Message,
	on_submit: impl Fn(Date) -> Message + 'static,
	on_clear: Message,
) -> Element<Message> {
	let add_due_date_button = add_due_date_button(on_edit.clone());

	if edit_due_date {
		date_picker(
			true,
			due_date
				.map(|due_date| due_date.to_iced_date())
				.unwrap_or(Date::today()),
			add_due_date_button,
			on_submit,
			stop_editing,
		)
		.into()
	} else {
		match due_date {
			Some(due_date) => row![
				edit_due_date_button(due_date, date_formatting, on_edit),
				clear_task_due_date_button(on_clear),
			]
			.into(),
			None => add_due_date_button.into(),
		}
	}
}

pub fn open_in_code_editor_button(
	file_location: String,
	code_editor: &CodeEditor,
) -> Button<'static, Message> {
	button(
		row![
			code_editor.icon(),
			text(format!("Open in {}", code_editor.name()))
		]
		.align_y(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT),
	)
	.on_press(Message::OpenInCodeEditor(file_location))
	.style(secondary_button_style_default)
}

pub fn code_editor_dropdown_button(
	selected_code_editor: Option<&CodeEditor>,
	dropdown_expanded: bool,
) -> Element<Message> {
	let default_custom_code_editor = CodeEditor::Custom {
		name: "Custom editor name".to_string(),
		command: "custom_editor --open".to_string(),
	};

	DropDown::new(
		button(
			row![
				icon_to_text(if dropdown_expanded {
					Bootstrap::CaretDownFill
				} else {
					Bootstrap::CaretRightFill
				})
				.size(ICON_FONT_SIZE),
				row![
					code_editor_icon(selected_code_editor.cloned()),
					text(code_editor_label(selected_code_editor.cloned()))
				]
				.spacing(SMALL_SPACING_AMOUNT)
				.align_y(Vertical::Center)
			]
			.spacing(SPACING_AMOUNT)
			.align_y(Vertical::Center),
		)
		.width(Length::Fixed(150.0))
		.on_press(settings_modal::Message::ToggleCodeEditorDropdownExpanded.into())
		.style(secondary_button_style_default),
		container(
			column![
				code_editor_button(None, selected_code_editor, true, false),
				code_editor_button(Some(CodeEditor::VSCode), selected_code_editor, false, false),
				code_editor_button(Some(CodeEditor::Zed), selected_code_editor, false, false),
				code_editor_button(
					Some(default_custom_code_editor),
					selected_code_editor,
					false,
					true
				),
			]
			.width(Fill),
		)
		.style(dropdown_container_style),
		dropdown_expanded,
	)
	.width(Fixed(150.0))
	.alignment(drop_down::Alignment::Bottom)
	.offset(0.0)
	.on_dismiss(settings_modal::Message::CollapseCodeEditorDropdown.into())
	.into()
}

fn code_editor_icon(code_editor: Option<CodeEditor>) -> Element<'static, Message> {
	match code_editor {
		Some(code_editor) => code_editor.icon(),
		None => Space::new(ICON_FONT_SIZE, ICON_FONT_SIZE).into(),
	}
}

fn code_editor_label(code_editor: Option<CodeEditor>) -> String {
	match code_editor {
		Some(code_editor) => code_editor.label().to_string(),
		None => "None".to_string(),
	}
}

fn code_editor_button(
	code_editor: Option<CodeEditor>,
	selected_code_editor: Option<&CodeEditor>,
	round_top: bool,
	round_bottom: bool,
) -> Button<Message> {
	let selected = match selected_code_editor {
		Some(CodeEditor::Custom { .. }) => {
			matches!(code_editor, Some(CodeEditor::Custom { .. }))
		}
		_ => selected_code_editor == code_editor.as_ref(),
	};

	button(
		row![
			code_editor_icon(code_editor.clone()),
			text(code_editor_label(code_editor.clone())),
		]
		.spacing(SPACING_AMOUNT)
		.align_y(Vertical::Center),
	)
	.on_press(settings_modal::Message::SetCodeEditor(code_editor).into())
	.width(Fill)
	.style(move |t, s| {
		selection_list_button_style(
			t,
			s,
			selected,
			round_top,
			round_top,
			round_bottom,
			round_bottom,
		)
	})
}

pub fn retry_loading_database_button() -> Button<'static, Message> {
	icon_label_button("Retry", Bootstrap::ArrowClockwise)
		.style(dangerous_button_style)
		.on_press(Message::LoadDatabase)
}

pub fn synchronization_settings_button(
	label: &'static str,
	selected: bool,
	on_press: Message,
	round_left: bool,
	round_right: bool,
) -> Button<'static, Message> {
	button(text(label).align_x(Horizontal::Center))
		.style(move |t, s| {
			selection_list_button_style(
				t,
				s,
				selected,
				round_left,
				round_right,
				round_left,
				round_right,
			)
		})
		.width(110.0)
		.on_press(on_press)
}

pub fn show_error_popup_button(error_msg: String) -> Element<'static, Message> {
	tooltip(
		button(
			icon_to_text(Bootstrap::Bug)
				.style(danger_text_style)
				.size(ICON_FONT_SIZE)
				.align_x(Horizontal::Center)
				.align_y(Vertical::Center),
		)
		.width(ICON_BUTTON_WIDTH)
		.on_press(error_msg_modal::Message::open(error_msg))
		.style(secondary_button_style_default),
		text("Show full error popup").size(SMALL_TEXT_SIZE),
		tooltip::Position::Top,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn retry_synchronization_button() -> Element<'static, Message> {
	tooltip(
		icon_button(Bootstrap::ArrowClockwise)
			.on_press(Message::SyncDatabase)
			.style(secondary_button_style_default),
		text("Retry").size(SMALL_TEXT_SIZE),
		tooltip::Position::Top,
	)
	.gap(GAP)
	.style(tooltip_container_style)
	.into()
}

pub fn select_synchronization_filepath_button() -> Button<'static, Message> {
	icon_label_button("Select", Bootstrap::Folder)
		.on_press(settings_modal::Message::BrowseSynchronizationFilepath.into())
		.style(dangerous_button_style)
}

pub fn calendar_view_button(
	calendar_view: CalendarView,
	selected: bool,
	round_left: bool,
	round_right: bool,
) -> Button<'static, Message> {
	button(text(calendar_view.label()).align_x(Horizontal::Center))
		.width(SETTINGS_SELECTION_LIST_WIDTH / 3.0)
		.on_press(
			PreferenceMessage::SetOverviewPage(SerializedOverviewPage::Calendar {
				view: calendar_view,
			})
			.into(),
		)
		.style(move |t, s| {
			selection_list_button_style(
				t,
				s,
				selected,
				round_left,
				round_right,
				round_left,
				round_right,
			)
		})
}

pub fn calendar_navigation_button(forward: bool) -> Button<'static, Message> {
	large_icon_button(if forward {
		Bootstrap::ArrowRight
	} else {
		Bootstrap::ArrowLeft
	})
	.on_press(
		if forward {
			overview_page::Message::GoForward
		} else {
			overview_page::Message::GoBackward
		}
		.into(),
	)
	.style(hidden_secondary_button_style)
}

pub fn calendar_today_button() -> Button<'static, Message> {
	button(text("Today").align_y(Vertical::Center))
		.on_press(overview_page::Message::GoToToday.into())
		.style(primary_button_style)
		.height(LARGE_ICON_BUTTON_WIDTH)
}
