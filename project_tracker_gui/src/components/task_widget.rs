use std::{borrow::Cow, time::Duration, str::FromStr};
use iced::{theme, widget::{button, checkbox, column, container, container::Id, row, text, text_editor, text_input, Column, Row}, Alignment, Element, Length, Padding};
use iced_drop::droppable;
use iced_aw::{date_picker, date_picker::Date};
use once_cell::sync::Lazy;
use crate::{core::{DatabaseMessage, DateFormatting, OrderedHashMap, ProjectId, Task, TaskId, TaskTag, TaskTagId, TASK_TAG_QUAD_HEIGHT}, pages::{EditTaskState, SidebarPageMessage}, styles::{SecondaryButtonStyle, TaskBackgroundContainerStyle, TextInputStyle, SMALL_HORIZONTAL_PADDING, SPACING_AMOUNT, TINY_SPACING_AMOUNT}};
use crate::pages::ProjectPageMessage;
use crate::project_tracker::UiMessage;
use crate::styles::{TextEditorStyle, SMALL_PADDING_AMOUNT, GREY, GreenCheckboxStyle, HiddenSecondaryButtonStyle, strikethrough_text};
use crate::components::{delete_task_button, clear_task_needed_time_button, clear_task_due_date_button, unfocusable, duration_widget, duration_text, task_tags_buttons, date_text, in_between_dropzone, date_widget};

pub static EDIT_NEEDED_TIME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
pub static EDIT_DUE_DATE_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[allow(clippy::too_many_arguments)]
pub fn task_widget<'a>(task: &'a Task, task_id: TaskId, is_task_todo: bool, project_id: ProjectId, task_tags: &'a OrderedHashMap<TaskTagId, TaskTag>, edit_task_state: Option<&'a EditTaskState>, dragging: bool, just_minimal_dragging: bool, highlight: bool, date_formatting: DateFormatting) -> Element<'a, UiMessage> {
	let inner_text_element: Element<UiMessage> = if let Some(edit_task_state) = edit_task_state {
		unfocusable(
			text_editor(&edit_task_state.new_name)
				.on_action(|action| ProjectPageMessage::TaskNameAction(action).into())
				.style(theme::TextEditor::Custom(Box::new(TextEditorStyle {
					// is the first tag enabled?
					round_top_left: task_tags
						.iter()
						.next()
						.map(|(tag_id, _tag)|
							!task.tags.contains(&tag_id)
						)
						.unwrap_or(true),
					round_top_right: false,
					round_bottom_left: false,
					round_bottom_right: edit_task_state.new_name.line_count() > 1 // multiline?
				}))),

			ProjectPageMessage::StopEditingTask.into()
		)
		.into()
	}
	else {
		(
			if is_task_todo {
				text(&task.name)
			}
			else {
				text(strikethrough_text(&task.name))
			}
		)
		.style(
			if is_task_todo {
				theme::Text::Default
			}
			else {
				theme::Text::Color(GREY)
			}
		)
		.width(Length::Shrink)
		.into()
	};

	if let Some(edit_task_state) = edit_task_state {
		column![
			task_tags_buttons(
				task_tags,
				&task.tags,
				|tag_id| ProjectPageMessage::ToggleTaskTag(tag_id).into()
			),

			row![
				inner_text_element,
				delete_task_button(project_id, task_id),
			]
			.align_items(Alignment::Start),

			row![
				if edit_task_state.edit_needed_time {
					let stop_editing_task_message: UiMessage = ProjectPageMessage::StopEditingTask.into();

					let edit_needed_time_element = unfocusable(
						text_input(
							"mins",
							&match task.needed_time_minutes {
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
									DatabaseMessage::ChangeTaskNeededTime {
										project_id,
										task_id,
										new_needed_time_minutes,
									}.into()
								},
								None => ProjectPageMessage::InvalidNeededTimeInput.into(),
							}
						})
						.on_submit(ProjectPageMessage::StopEditingTaskNeededTime.into())
						.style(theme::TextInput::Custom(Box::new(TextInputStyle{
							round_left_top: false,
							round_left_bottom: true,
							round_right_top: false,
							round_right_bottom: false
						}))),

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
							if let Some(needed_duration_minutes) = &task.needed_time_minutes {
								duration_text(Cow::Owned(Duration::from_secs(*needed_duration_minutes as u64 * 60)))
							}
							else {
								text("Add needed time")
							}
						)
						.padding(SMALL_HORIZONTAL_PADDING)
						.on_press(ProjectPageMessage::EditTaskNeededTime.into())
						.style(theme::Button::custom(SecondaryButtonStyle::ONLY_ROUND_BOTTOM))
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
						button(
							date_text(due_date, date_formatting)
						)
						.padding(SMALL_HORIZONTAL_PADDING)
						.on_press(ProjectPageMessage::EditTaskDueDate.into())
						.style(theme::Button::custom(SecondaryButtonStyle {
							round_left_bottom: true,
							..SecondaryButtonStyle::NO_ROUNDING
						})),
						clear_task_due_date_button(),
					]
					.into()
				}
				else {
					Element::new(
						button(
							text("Add due date")
						)
						.padding(SMALL_HORIZONTAL_PADDING)
						.on_press(ProjectPageMessage::EditTaskDueDate.into())
						.style(theme::Button::custom(SecondaryButtonStyle::ONLY_ROUND_BOTTOM))
					)
				},
			]
			.spacing(SPACING_AMOUNT),
		]
		.into()
	}
	else {
		let tags_element = Row::with_children(
			task.tags
				.iter()
				.map(|tag_id| {
					task_tags
						.get(tag_id)
						.map(TaskTag::view)
						.unwrap_or("<invalid tag id>".into())
				})
		)
		.spacing(TINY_SPACING_AMOUNT);

		let inner: Element<UiMessage> = row![
			container(
				checkbox("", !is_task_todo)
					.on_toggle(move |checked| {
						if checked {
							DatabaseMessage::SetTaskDone { project_id, task_id }.into()
						}
						else {
							DatabaseMessage::SetTaskTodo { project_id, task_id }.into()
						}
					})
					.style(theme::Checkbox::Custom(Box::new(GreenCheckboxStyle)))
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
				.push_maybe(
					if task.needed_time_minutes.is_some() || task.due_date.is_some() {
						Some(
							Row::new()
								.push_maybe(
									task.needed_time_minutes.map(|duration_minutes| duration_widget(Cow::Owned(Duration::from_secs(duration_minutes as u64 * 60))))
								)
								.push_maybe(
									task.due_date.as_ref().map(|due_date| date_widget(due_date, date_formatting))
								)
								.spacing(SPACING_AMOUNT)
						)
					}
					else {
						None
					}
				)
				.spacing(TINY_SPACING_AMOUNT)
		]
		.width(Length::Fill)
		.align_items(Alignment::Start)
		.into();

		column![
			in_between_dropzone(
				if is_task_todo {
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
				.style(theme::Container::Custom(Box::new(TaskBackgroundContainerStyle{ dragging })))
			)
			.on_drop(move |point, rect| SidebarPageMessage::DropTask{ project_id, task_id, point, rect }.into())
			.on_drag(move |point, rect| SidebarPageMessage::DragTask{ project_id, task_id, is_task_todo, point, rect }.into())
			.on_click(ProjectPageMessage::PressTask(task_id).into())
			.on_cancel(SidebarPageMessage::CancelDragTask.into())
			.drag_overlay(!just_minimal_dragging)
			.drag_hide(!just_minimal_dragging)
			.style(theme::Button::custom(HiddenSecondaryButtonStyle))
		]
		.into()
	}
}