use iced::{theme, widget::{checkbox, button, mouse_area, row, text, text_input, Row}, Alignment, Padding, Element, Length};
use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;
use crate::{pages::ProjectPageMessage, project_tracker::UiMessage, styles::{strikethrough_text, PADDING_AMOUNT}};
use crate::styles::{MIDDLE_TEXT_SIZE, GREY, GreenCheckboxStyle, TextInputStyle, TaskButtonStyle};
use crate::components::{edit_task_button, delete_task_button, move_task_up_button, move_task_down_button};
use crate::core::{ProjectId, TaskState};

pub static EDIT_TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub type TaskId = usize;

pub fn generate_task_id() -> TaskId {
	rand::random()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
	pub name: String,
	pub state: TaskState,
}

impl Task {
	pub fn new(name: String, state: TaskState) -> Self {
		Self {
			name,
			state,
		}
	}

	pub fn is_done(&self) -> bool {
		self.state.is_done()
	}

	pub fn view(&self, project_id: ProjectId, self_task_id: TaskId, editing: bool, hovered: bool, can_move_up: bool, can_move_down: bool) -> Element<UiMessage> {
		if editing {
			let move_project_element: Option<Element<UiMessage>> = {
				match (can_move_up, can_move_down) {
					(true, true) => Some(row![
						move_task_up_button(project_id, self_task_id),
						move_task_down_button(project_id, self_task_id),
					].into()),
					(true, false) => Some(move_task_up_button(project_id, self_task_id).into()),
					(false, true) => Some(move_task_down_button(project_id, self_task_id).into()),
					(false, false) => None,
				}
			};

			Row::new()
				.push(
					text_input("Task name", &self.name)
						.id(EDIT_TASK_NAME_INPUT_ID.clone())
						.size(MIDDLE_TEXT_SIZE)
						.width(Length::Fill)
						.on_input(move |new_task_name| UiMessage::ChangeTaskName { project_id, task_id: self_task_id, new_task_name })
						.on_submit(ProjectPageMessage::StopEditing.into())
						.style(theme::TextInput::Custom(Box::new(TextInputStyle)))
				)
				.push_maybe(move_project_element)
				.push(delete_task_button(project_id, self_task_id))
				.align_items(Alignment::Center)
				.into()
		}
		else {
			mouse_area(
				button(
					row![
						row![
							checkbox("", self.state.is_done())
								.on_toggle(move |checked| {
									UiMessage::SetTaskState {
										project_id,
										task_id: self_task_id,
										task_state:
											if checked {
												TaskState::Done
											}
											else {
												TaskState::Todo
											},
									}
								})
								.style(theme::Checkbox::Custom(Box::new(GreenCheckboxStyle))),

							text(if self.state.is_done() { strikethrough_text(&self.name) } else { self.name.clone() })
								.style(
									if self.is_done() {
										theme::Text::Color(GREY)
									}
									else {
										theme::Text::Default
									}
								)
								.width(Length::Shrink),
						]
						.width(Length::Fill)
						.align_items(Alignment::Start),

						edit_task_button(self_task_id, hovered),
					]
					.align_items(Alignment::Center)
					.width(Length::Fill)
				)
				.style(theme::Button::custom(TaskButtonStyle))
				.padding(Padding{ left: PADDING_AMOUNT, ..Padding::ZERO })
				.on_press(UiMessage::Nothing)
			)
			.on_move(move |_pos| ProjectPageMessage::HoveringTask(self_task_id).into())
			.on_exit(ProjectPageMessage::StoppedHoveringTask.into())
			.into()
		}
	}
}
