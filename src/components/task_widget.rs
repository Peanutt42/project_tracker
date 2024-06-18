use iced::{Alignment, Element, Length, Padding, widget::{row, Row, text, text_input, mouse_area, button, checkbox}, theme};
use once_cell::sync::Lazy;
use crate::core::{ProjectId, Task, TaskId, TaskMessage, TaskState};
use crate::project_tracker::UiMessage;
use crate::pages::ProjectPageMessage;
use crate::components::{edit_task_button, delete_task_button, move_task_up_button, move_task_down_button};
use crate::styles::{MIDDLE_TEXT_SIZE, PADDING_AMOUNT, GREY, GreenCheckboxStyle, TextInputStyle, TaskButtonStyle, strikethrough_text};

pub static EDIT_TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub fn task_widget(task: &Task, task_id: TaskId, project_id: ProjectId, editing: bool, hovered: bool, can_move_up: bool, can_move_down: bool) -> Element<UiMessage> {
	let inner_text_element = if editing {
		text_input("Task name", &task.name)
			.id(EDIT_TASK_NAME_INPUT_ID.clone())
			.size(MIDDLE_TEXT_SIZE)
			.width(Length::Fill)
			.on_input(move |new_task_name| TaskMessage::ChangeName(new_task_name).to_ui_message(project_id, task_id))
			.on_submit(ProjectPageMessage::StopEditing.into())
			.style(theme::TextInput::Custom(Box::new(TextInputStyle)))
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
			.width(Length::Shrink).into()
	};

	custom_task_widget(inner_text_element,  task.state, Some(task_id), project_id, editing, hovered, can_move_up, can_move_down)
}

#[allow(clippy::too_many_arguments)]
pub fn custom_task_widget(inner_text_element: Element<UiMessage>, task_state: TaskState, task_id: Option<TaskId>, project_id: ProjectId, editing: bool, hovered: bool, can_move_up: bool, can_move_down: bool) -> Element<UiMessage> {
	if let Some(task_id) = task_id {
		if editing {
			let move_project_element: Option<Element<UiMessage>> = {
				if task_state.is_todo() {
					match (can_move_up, can_move_down) {
						(true, true) => Some(row![
							move_task_up_button(project_id, task_id),
							move_task_down_button(project_id, task_id),
						].into()),
						(true, false) => Some(move_task_up_button(project_id, task_id).into()),
						(false, true) => Some(move_task_down_button(project_id, task_id).into()),
						(false, false) => None,
					}
				}
				else {
					None
				}
			};

			Row::new()
				.push(inner_text_element)
				.push_maybe(move_project_element)
				.push(delete_task_button(project_id, task_id))
				.align_items(Alignment::Center)
				.into()
		}
		else {
			mouse_area(
				button(
					row![
						row![
							checkbox("", task_state.is_done())
								.on_toggle(move |checked| {
									TaskMessage::ChangeState(
										if checked {
											TaskState::Done
										}
										else {
											TaskState::Todo
										}
									)
									.to_ui_message(project_id, task_id)
								})
								.style(theme::Checkbox::Custom(Box::new(GreenCheckboxStyle))),

							inner_text_element,
						]
						.width(Length::Fill)
						.align_items(Alignment::Start),

						edit_task_button(task_id, hovered),
					]
					.align_items(Alignment::Center)
					.width(Length::Fill)
				)
				.style(theme::Button::custom(TaskButtonStyle))
				.padding(Padding{ left: PADDING_AMOUNT, ..Padding::ZERO })
				.on_press(UiMessage::Nothing)
			)
			.on_move(move |_pos| ProjectPageMessage::HoveringTask(task_id).into())
			.on_exit(ProjectPageMessage::StoppedHoveringTask.into())
			.into()
		}
	}
	else {
		inner_text_element
	}
}
