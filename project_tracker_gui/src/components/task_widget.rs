use std::{borrow::Cow, time::Duration, str::FromStr};
use iced::{theme, widget::{button, checkbox, column, container, container::Id, row, text, text_editor, text_input, Column, Row}, Alignment, Element, Length, Padding};
use iced_drop::droppable;
use once_cell::sync::Lazy;
use crate::{core::{DatabaseMessage, OrderedHashMap, ProjectId, Task, TaskId, TaskState, TaskTag, TaskTagId, TASK_TAG_QUAD_HEIGHT}, pages::{EditTaskState, SidebarPageMessage}, styles::{DropZoneContainerStyle, RoundedSecondaryButtonStyle, TaskBackgroundContainerStyle, TextInputStyle, BORDER_RADIUS, SMALL_HORIZONTAL_PADDING, TINY_SPACING_AMOUNT}};
use crate::pages::ProjectPageMessage;
use crate::project_tracker::UiMessage;
use crate::styles::{TextEditorStyle, SMALL_PADDING_AMOUNT, GREY, GreenCheckboxStyle, HiddenSecondaryButtonStyle, strikethrough_text};
use crate::components::{delete_task_button, clear_task_needed_time_button, unfocusable, duration_widget, duration_text, task_tags_buttons};

pub static EDIT_NEEDED_TIME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[allow(clippy::too_many_arguments)]
pub fn task_widget<'a>(task: &'a Task, task_id: TaskId, project_id: ProjectId, task_tags: &'a OrderedHashMap<TaskTagId, TaskTag>, edit_task_state: Option<&'a EditTaskState>, dragging: bool, just_minimal_dragging: bool, highlight: bool) -> Element<'a, UiMessage> {
	let inner_text_element: Element<UiMessage> = if let Some(edit_task_state) = edit_task_state {
		unfocusable(
			text_editor(&edit_task_state.new_name)
				.on_action(|action| ProjectPageMessage::TaskNameAction(action).into())
				.style(theme::TextEditor::Custom(Box::new(TextEditorStyle{ round_top_right: false, round_bottom_right: edit_task_state.new_name.line_count() != 1 }))),

			ProjectPageMessage::StopEditingTask.into()
		)
		.into()
	}
	else {
		(
			if task.is_todo() {
				text(&task.name)
			}
			else {
				text(strikethrough_text(&task.name))
			}
		)
		.style(
			if task.is_done() {
				theme::Text::Color(GREY)
			}
			else {
				theme::Text::Default
			}
		)
		.width(Length::Shrink)
		.into()
	};

	if let Some(edit_task_state) = edit_task_state {
		column![
			container(
				task_tags_buttons(
					task_tags,
					&task.tags,
					|tag_id| ProjectPageMessage::ToggleTaskTag(tag_id).into()
				)
			)
			.padding(Padding{ left: BORDER_RADIUS, ..Padding::ZERO }),

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
						.style(theme::TextInput::Custom(Box::new(TextInputStyle{ round_left: true, round_right: false }))),

						stop_editing_task_message
					);

					row![
						edit_needed_time_element,
						clear_task_needed_time_button(task_id),
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
						.style(theme::Button::custom(RoundedSecondaryButtonStyle))
					)
				},
			],
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
				checkbox("", task.state.is_done())
					.on_toggle(move |checked| {
						DatabaseMessage::ChangeTaskState {
							project_id,
							task_id,
							new_task_state:
								if checked {
									TaskState::Done
								}
								else {
									TaskState::Todo
								},
						}.into()
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
					task.needed_time_minutes.map(|duration_minutes| duration_widget(Cow::Owned(Duration::from_secs(duration_minutes as u64 * 60))))
				)
				.spacing(TINY_SPACING_AMOUNT)
		]
		.width(Length::Fill)
		.align_items(Alignment::Start)
		.into();

		droppable(
			container(
				inner
			)
			.id(if task.state.is_todo() {
				task.dropzone_id.clone()
			}
			else {
				Id::unique()
			})
			.padding(Padding::new(SMALL_PADDING_AMOUNT))
			.style(if highlight {
					theme::Container::Custom(Box::new(DropZoneContainerStyle{ highlight }))
				}
				else {
					theme::Container::Custom(Box::new(TaskBackgroundContainerStyle{ dragging }))
				})
		)
		.on_drop(move |point, rect| SidebarPageMessage::DropTask{ project_id, task_id, point, rect }.into())
		.on_drag(move |point, rect| SidebarPageMessage::DragTask{ project_id, task_id, task_state: task.state, point, rect }.into())
		.on_click(ProjectPageMessage::PressTask(task_id).into())
		.on_cancel(SidebarPageMessage::CancelDragTask.into())
		.drag_overlay(!just_minimal_dragging)
		.drag_hide(!just_minimal_dragging)
		.style(theme::Button::custom(HiddenSecondaryButtonStyle))
		.into()
	}
}