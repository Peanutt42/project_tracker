use iced::{theme, widget::{checkbox, column, container, container::Id, row, text, text_input, Row}, Alignment, Element, Length, Padding};
use iced_drop::droppable;
use once_cell::sync::Lazy;
use crate::{core::{DatabaseMessage, OrderedHashMap, ProjectId, Task, TaskId, TaskState, TaskTag, TaskTagId, TASK_TAG_QUAD_HEIGHT}, pages::SidebarPageMessage, styles::{DropZoneContainerStyle, TaskBackgroundContainerStyle, BORDER_RADIUS, TINY_SPACING_AMOUNT}};
use crate::pages::ProjectPageMessage;
use crate::project_tracker::UiMessage;
use crate::styles::{TextInputStyle, SMALL_PADDING_AMOUNT, GREY, GreenCheckboxStyle, HiddenSecondaryButtonStyle, strikethrough_text};
use crate::components::{delete_task_button, unfocusable};

use super::task_tags_buttons;

pub static EDIT_TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[allow(clippy::too_many_arguments)]
pub fn task_widget<'a>(task: &'a Task, task_id: TaskId, project_id: ProjectId, task_tags: &'a OrderedHashMap<TaskTagId, TaskTag>, edited_name: Option<&'a String>, dragging: bool, highlight: bool) -> Element<'a, UiMessage> {
	let inner_text_element: Element<UiMessage> = if let Some(edited_name) = edited_name {
		unfocusable(
			text_input("Task name", edited_name)
				.id(EDIT_TASK_NAME_INPUT_ID.clone())
				.width(Length::Fill)
				.on_input(move |new_task_name| ProjectPageMessage::ChangeEditedTaskName(new_task_name).into())
				.on_submit(ProjectPageMessage::ChangeTaskName.into())
				.style(theme::TextInput::Custom(Box::new(TextInputStyle { round_left: true, round_right: false }))),

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

	if edited_name.is_some() {
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
			.align_items(Alignment::Center)
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
			.padding(Padding{ top: TASK_TAG_QUAD_HEIGHT + TINY_SPACING_AMOUNT, ..Padding::ZERO }),


			if task.tags.is_empty() {
				inner_text_element
			}
			else {
				column![
					tags_element,
					inner_text_element
				]
				.spacing(TINY_SPACING_AMOUNT)
				.into()
			}
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
		.drag_hide(true)
		.style(theme::Button::custom(HiddenSecondaryButtonStyle))
		.into()
	}
}