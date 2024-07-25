use iced::{Alignment, Element, Length, Padding, widget::{row, Row, text, text_input, button, checkbox}, theme};
use once_cell::sync::Lazy;
use crate::core::{DatabaseMessage, ProjectId, Task, TaskId, TaskState};
use crate::pages::ProjectPageMessage;
use crate::project_tracker::UiMessage;
use crate::styles::{TextInputStyle, SMALL_PADDING_AMOUNT, GREY, GreenCheckboxStyle, HiddenSecondaryButtonStyle, strikethrough_text};
use crate::components::{move_task_up_button, move_task_down_button, delete_task_button, unfocusable};

pub static EDIT_TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub fn task_widget<'a>(task: &'a Task, task_id: TaskId, project_id: ProjectId, edited_name: Option<&'a String>, can_move_up: bool, can_move_down: bool) -> Element<'a, UiMessage> {
	let inner_text_element = if let Some(edited_name) = edited_name {
		unfocusable(
			text_input("Task name", edited_name)
				.id(EDIT_TASK_NAME_INPUT_ID.clone())
				.width(Length::Fill)
				.on_input(move |new_task_name| ProjectPageMessage::ChangeEditedTaskName(new_task_name).into())
				.on_submit(DatabaseMessage::ChangeTaskName{ project_id, task_id, new_task_name: edited_name.clone() }.into())
				.style(theme::TextInput::Custom(Box::new(TextInputStyle))),

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

	custom_task_widget(inner_text_element, task.state, Some(task_id), project_id, edited_name.is_some(), can_move_up, can_move_down)
}

#[allow(clippy::too_many_arguments)]
pub fn custom_task_widget(inner_text_element: Element<UiMessage>, task_state: TaskState, task_id: Option<TaskId>, project_id: ProjectId, editing: bool, can_move_up: bool, can_move_down: bool) -> Element<UiMessage> {
	if let Some(task_id) = task_id {
		if editing {
			let move_project_element: Option<Element<UiMessage>> = {
				if task_state.is_todo() {
					Some(
						row![
							move_task_up_button(project_id, task_id, can_move_up),
							move_task_down_button(project_id, task_id, can_move_down)
						]
						.into()
					)
				}
				else {
					None
				}
			};

			Row::new()
				.push_maybe(move_project_element)
				.push(inner_text_element)
				.push(delete_task_button(project_id, task_id))
				.align_items(Alignment::Center)
				.into()
		}
		else {
			button(
				row![
					checkbox("", task_state.is_done())
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

					inner_text_element,
				]
				.width(Length::Fill)
				.align_items(Alignment::Start)
			)
			.style(theme::Button::custom(HiddenSecondaryButtonStyle))
			.padding(Padding::new(SMALL_PADDING_AMOUNT))
			.on_press(ProjectPageMessage::EditTask(task_id).into())
			.into()
		}
	}
	else {
		inner_text_element
	}
}
