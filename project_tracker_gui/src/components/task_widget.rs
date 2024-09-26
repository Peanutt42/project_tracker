use std::{borrow::Cow, str::FromStr, time::Duration};
use iced::{widget::{button, checkbox, column, container, container::Id, row, text, text_editor, text_input, Column, Row}, Alignment, Element, Length::{self, Fill}, Padding, Theme};
use iced_aw::{Bootstrap, core::icons::bootstrap::icon_to_text};
use iced_drop::droppable;
use iced_aw::{date_picker, date_picker::Date};
use once_cell::sync::Lazy;
use crate::{core::{DatabaseMessage, DateFormatting, OrderedHashMap, ProjectId, Task, TaskId, TaskTag, TaskTagId, TaskType, TASK_TAG_QUAD_HEIGHT}, pages::{EditTaskState, SidebarPageMessage}, styles::{checkbox_style, secondary_button_style_default, secondary_button_style_only_round_bottom, shadow_container_style, strikethrough_text, task_background_container_style, task_button_style, text_editor_style, text_input_style, GREY, SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SMALL_TEXT_SIZE, SPACING_AMOUNT, TINY_SPACING_AMOUNT}};
use crate::pages::ProjectPageMessage;
use crate::project_tracker::UiMessage;
use crate::components::{delete_task_button, clear_task_needed_time_button, clear_task_due_date_button, unfocusable, duration_widget, duration_text, task_tags_buttons, in_between_dropzone, add_due_date_button, edit_due_date_button, days_left_widget, start_task_timer_button};

pub static EDIT_NEEDED_TIME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
pub static EDIT_DUE_DATE_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[allow(clippy::too_many_arguments)]
pub fn task_widget<'a>(task: &'a Task, task_id: TaskId, task_type: TaskType, project_id: ProjectId, task_tags: &'a OrderedHashMap<TaskTagId, TaskTag>, edit_task_state: Option<&'a EditTaskState>, dragging: bool, just_minimal_dragging: bool, highlight: bool, stopwatch_label: Option<&'a String>, date_formatting: DateFormatting) -> Element<'a, UiMessage> {
	if let Some(edit_task_state) = edit_task_state {
		edit_task_widget_view(task, project_id, task_id, task_tags, edit_task_state, date_formatting)
	}
	else {
		task_widget_view(task, task_type, project_id, task_id, task_tags, dragging, just_minimal_dragging, highlight, stopwatch_label)
	}
}

fn edit_task_widget_view<'a>(task: &'a Task, project_id: ProjectId, task_id: TaskId, task_tags: &'a OrderedHashMap<TaskTagId, TaskTag>, edit_task_state: &'a EditTaskState, date_formatting: DateFormatting) -> Element<'a, UiMessage> {
	let edit_task_name_text_editor: Element<'a, UiMessage> = {
		let round_top_left = task.needed_time_minutes.is_none() &&
						task_tags.get_key_at_order(0).map(|tag_id| !task.tags.contains(tag_id)).unwrap_or(true);
		let round_bottom_right = edit_task_state.new_name.line_count() > 1; // multiline?

		unfocusable(
			container(
				text_editor(&edit_task_state.new_name)
					.on_action(move |action| ProjectPageMessage::TaskNameAction(action).into())
					.style(move |t, s| text_editor_style(
						t,
						s,
						round_top_left,
						false,
						false,
						round_bottom_right
					))
			)
			.style(shadow_container_style),

			ProjectPageMessage::StopEditingTask.into()
		)
		.into()
	};

	column![
		task_tags_buttons(
			task_tags,
			&task.tags,
			|tag_id| ProjectPageMessage::ToggleTaskTag(tag_id).into()
		),

		row![
			Row::new()
				.push_maybe(task.needed_time_minutes.as_ref().map(|_|
					start_task_timer_button(
						project_id,
						task_id,
						task_tags
							.iter()
							.next()
							.map(|(tag_id, _tag)|
								!task.tags.contains(&tag_id)
							)
							.unwrap_or(true)
					)
				))
				.push(
					column![
						edit_task_name_text_editor,
						row![
							if let Some(new_task_needed_minutes) = &edit_task_state.new_needed_time_minutes {
								let stop_editing_task_message: UiMessage = ProjectPageMessage::StopEditingTask.into();

								let edit_needed_time_element = unfocusable(
									text_input(
										"mins",
										&match new_task_needed_minutes {
											Some(needed_time_minutes) => format!("{needed_time_minutes}"),
											None => String::new(),
										}
									)
									.id(EDIT_NEEDED_TIME_TEXT_INPUT_ID.clone())
									.width(Length::Fixed(50.0))
									.on_input(move |input| {
										let new_needed_time_minutes = match usize::from_str(&input) {
											Ok(new_needed_time_minutes) => Some(Some(new_needed_time_minutes)),
											Err(_) => {
												if input.is_empty() {
													Some(None)
												}
												else {
													None
												}
											},
										};
										match new_needed_time_minutes {
											Some(new_needed_time_minutes) => {
												ProjectPageMessage::ChangeNewTaskNeededTimeInput(new_needed_time_minutes).into()
											},
											None => ProjectPageMessage::InvalidNeededTimeInput.into(),
										}
									})
									.on_submit(ProjectPageMessage::ChangeTaskNeededTime.into())
									.style(move |t, s| text_input_style(t, s, false, false, false, true)),

									stop_editing_task_message
								);

								row![
									edit_needed_time_element,
									clear_task_needed_time_button(),
								]
								.into()
							}
							else {
								Element::new(
									button(
										row![
											icon_to_text(Bootstrap::Stopwatch),
											if let Some(needed_duration_minutes) = &task.needed_time_minutes {
												duration_text(Cow::Owned(Duration::from_secs(*needed_duration_minutes as u64 * 60)))
											}
											else {
												text("Add needed time")
											}
										]
										.spacing(SMALL_SPACING_AMOUNT)
									)
									.padding(SMALL_HORIZONTAL_PADDING)
									.on_press(ProjectPageMessage::EditTaskNeededTime.into())
									.style(secondary_button_style_only_round_bottom)
								)
							},

							if edit_task_state.edit_due_date {
								Element::new(date_picker(
									edit_task_state.edit_due_date,
									task.due_date.unwrap_or(Date::today().into()),
									text("Edit due date"),
									ProjectPageMessage::StopEditingTaskDueDate.into(),
									move |date| ProjectPageMessage::ChangeTaskDueDate(date.into()).into()
								))
							}
							else if let Some(due_date) = &task.due_date {
								row![
									edit_due_date_button(due_date, date_formatting),
									clear_task_due_date_button(),
								]
								.into()
							}
							else {
								Element::new(add_due_date_button())
							},
						]
						.spacing(SPACING_AMOUNT),
					]
				)
				.align_y(Alignment::Start),

			delete_task_button(project_id, task_id)
		]
		.align_y(Alignment::Start)
	]
	.padding(Padding{ top: SPACING_AMOUNT as f32, ..Padding::ZERO })
	.into()
}

#[allow(clippy::too_many_arguments)]
fn task_widget_view<'a>(task: &'a Task, task_type: TaskType, project_id: ProjectId, task_id: TaskId, task_tags: &'a OrderedHashMap<TaskTagId, TaskTag>, dragging: bool, just_minimal_dragging: bool, highlight: bool, stopwatch_label: Option<&'a String>) -> Element<'a, UiMessage> {
	let tags_element = Row::with_children(
		task_tags.iter()
			.filter(|(tag_id, _tag)| task.tags.contains(tag_id))
			.map(|(_tag_id, tag)| tag.view())
	)
	.spacing(TINY_SPACING_AMOUNT);

	let inner_text_element: Element<'a, UiMessage> = {
		let text_style = if matches!(task_type, TaskType::Done) {
			text::Style { color: Some(GREY) }
		}
		else {
			text::Style::default()
		};

		if matches!(task_type, TaskType::Done) {
			text(strikethrough_text(&task.name))
				.width(Fill)
				.style(move |_theme| text_style)
				.into()
		}
		else {
			text(task.name.clone())
				.width(Fill)
				.style(move |_theme| text_style)
				.into()
		}
	};

	let inner: Element<UiMessage> = row![
		container(
			checkbox("", matches!(task_type, TaskType::Done))
				.on_toggle(move |checked| {
					if checked {
						DatabaseMessage::SetTaskDone { project_id, task_id }.into()
					}
					else {
						DatabaseMessage::SetTaskTodo { project_id, task_id }.into()
					}
				})
				.style(checkbox_style)
		)
		.padding(Padding{
			top: if task.tags.is_empty() {
				0.0
			}
			else {
				TASK_TAG_QUAD_HEIGHT + TINY_SPACING_AMOUNT
			},
			..Padding::ZERO
		}),


		Column::new()
			.push_maybe(if task.tags.is_empty() {
				None
			}
			else {
				Some(tags_element)
			})
			.push(inner_text_element)
			.spacing(TINY_SPACING_AMOUNT),

		Column::new()
			.push_maybe(
				if task.needed_time_minutes.is_some() || task.due_date.is_some() {
					Some(
						Column::new()
							.push_maybe(task.due_date.as_ref().map(|due_date| days_left_widget(*due_date)))
							.push_maybe(task.needed_time_minutes.map(|duration_minutes| duration_widget(Cow::Owned(Duration::from_secs(duration_minutes as u64 * 60)))))
							.push_maybe(
								stopwatch_label.map(|label| -> Element<UiMessage> {
									button(
										row![
											icon_to_text(Bootstrap::Stopwatch).size(SMALL_TEXT_SIZE),
											text(label).style(|theme: &Theme| text::Style {
												color: Some(theme.extended_palette().danger.base.color)
											})
										]
										.spacing(TINY_SPACING_AMOUNT)
									)
									.padding(SMALL_HORIZONTAL_PADDING)
									.style(secondary_button_style_default)
									.on_press(UiMessage::OpenStopwatch)
									.into()
								})
							)
							.spacing(TINY_SPACING_AMOUNT)
							.align_x(Alignment::End)
							.into()
					) as Option<Element<UiMessage>>
				}
				else {
					None
				}
			)
	]
	.width(Fill)
	.align_y(Alignment::Start)
	.into();

	column![
		in_between_dropzone(
			if  matches!(task_type, TaskType::Todo) {
				task.dropzone_id.clone()
			}
			else {
				Id::unique()
			},
			highlight
		),

		droppable(
			container(
				inner
			)
			.padding(Padding::new(SMALL_PADDING_AMOUNT))
			.style(move |t| task_background_container_style(t, dragging && !just_minimal_dragging))
		)
		.on_drop(move |point, rect| SidebarPageMessage::DropTask{ project_id, task_id, point, rect }.into())
		.on_drag(move |point, rect| SidebarPageMessage::DragTask{ project_id, task_id, task_is_todo: matches!(task_type, TaskType::Todo), point, rect }.into())
		.on_click(ProjectPageMessage::PressTask(task_id).into())
		.on_cancel(SidebarPageMessage::CancelDragTask.into())
		.drag_overlay(!just_minimal_dragging)
		.drag_hide(!just_minimal_dragging)
		.style(task_button_style)
	]
	.into()
}