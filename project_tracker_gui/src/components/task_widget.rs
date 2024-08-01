use iced::{theme, widget::{button, checkbox, column, container, container::Id, row, text, text_input, Row}, Alignment, Element, Length, Padding};
use iced_drop::droppable;
use once_cell::sync::Lazy;
use crate::{core::{DatabaseMessage, OrderedHashMap, ProjectId, Task, TaskId, TaskState, TaskTag, TaskTagId}, pages::SidebarPageMessage, styles::{DropZoneContainerStyle, TaskBackgroundContainerStyle, TaskTagButtonStyle, TINY_SPACING_AMOUNT}};
use crate::pages::ProjectPageMessage;
use crate::project_tracker::UiMessage;
use crate::styles::{TextInputStyle, SMALL_PADDING_AMOUNT, GREY, GreenCheckboxStyle, HiddenSecondaryButtonStyle, strikethrough_text};
use crate::components::{delete_task_button, unfocusable};

pub static EDIT_TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[allow(clippy::too_many_arguments)]
pub fn task_widget<'a>(task: &'a Task, task_id: TaskId, project_id: ProjectId, task_tags: &'a OrderedHashMap<TaskTagId, TaskTag>, edited_name: Option<&'a String>, dragging: bool, highlight: bool) -> Element<'a, UiMessage> {
	let inner_text_element: Element<UiMessage> = if let Some(edited_name) = edited_name {
		unfocusable(
			text_input("Task name", edited_name)
				.id(EDIT_TASK_NAME_INPUT_ID.clone())
				.width(Length::Fill)
				.on_input(move |new_task_name| ProjectPageMessage::ChangeEditedTaskName(new_task_name).into())
				.on_submit(DatabaseMessage::ChangeTaskName{ project_id, task_id, new_task_name: edited_name.clone() }.into())
				.style(theme::TextInput::Custom(Box::new(TextInputStyle { round_left: true, round_right: false }))),

			ProjectPageMessage::StopEditingTask.into()
		)
		.into()
	}
	else {
		text(if task.is_done() { strikethrough_text(&task.name) } else { task.name.clone() })
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
			Row::with_children(
				task_tags
					.iter()
					.map(|(tag_id, tag)| {
						button(
							text(&tag.name)
						)
						.on_press(ProjectPageMessage::ToggleTaskTag(tag_id).into())
						.style(theme::Button::custom(
							TaskTagButtonStyle {
								color: tag.color.into(),
								toggled: task.tags.contains(&tag_id)
							}
						))
						.into()
					})
			)
			.spacing(TINY_SPACING_AMOUNT),

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
				.style(theme::Checkbox::Custom(Box::new(GreenCheckboxStyle))),

			inner_text_element
		]
		.width(Length::Fill)
		.align_items(Alignment::Start)
		.into();

		droppable(
			container(
				if task.tags.is_empty() {
					inner
				}
				else {
					column![
						tags_element,
						inner
					]
					.spacing(TINY_SPACING_AMOUNT)
					.into()
				}
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