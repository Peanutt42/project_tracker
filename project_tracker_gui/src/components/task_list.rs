use std::collections::HashSet;
use iced::{alignment::{Alignment, Horizontal}, theme, widget::{column, container, row, scrollable, text::LineHeight, text_input, Column}, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{core::{DateFormatting, Project, TaskTagId}, pages::{CachedTaskList, EditTaskState, TaskDropzone, BOTTOM_TODO_TASK_DROPZONE_ID}, project_tracker::UiMessage, styles::{LARGE_PADDING_AMOUNT, PADDING_AMOUNT}};
use crate::core::{Task, TaskId, ProjectId};
use crate::components::{show_done_tasks_button, unfocusable, task_widget, cancel_create_task_button, delete_all_done_tasks_button, task_tags_buttons, in_between_dropzone};
use crate::styles::{SPACING_AMOUNT, HORIZONTAL_PADDING, ScrollableStyle, TextInputStyle, scrollable_vertical_direction};
use crate::pages::ProjectPageMessage;

pub static TASK_LIST_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
pub static CREATE_NEW_TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[allow(clippy::too_many_arguments)]
pub fn task_list<'a>(project_id: ProjectId, project: &'a Project, cached_task_list: &'a CachedTaskList, edited_task: &'a Option<EditTaskState>, dragged_task: Option<TaskId>, just_minimal_dragging: bool, hovered_task_dropzone: Option<TaskDropzone>, show_done_tasks: bool, create_new_task: &'a Option<(String, HashSet<TaskTagId>)>, date_formatting: DateFormatting) -> Element<'a, UiMessage> {
	let mut todo_task_elements = Vec::new();
	let mut done_task_elements = Vec::new(); // only gets populated when 'show_done_tasks'

	let task_view = |task_id: TaskId, task: &'a Task, is_todo: bool| {
		let edited_name = match edited_task {
			Some(edit_task_state) if task_id == edit_task_state.task_id => Some(edit_task_state),
			_ => None,
		};
		let dragging = match dragged_task {
			Some(dragged_task_id) => dragged_task_id == task_id,
			_ => false,
		};
		let highlight = match hovered_task_dropzone {
			Some(TaskDropzone::Task(hovered_task_id)) => hovered_task_id == task_id,
			_ => false,
		};
		task_widget(task, task_id, is_todo, project_id, &project.task_tags, edited_name, dragging, just_minimal_dragging, highlight, date_formatting)
	};

	for task_id in cached_task_list.todo.iter() {
		if let Some(task) = project.todo_tasks.get(task_id) {
			todo_task_elements.push(task_view(*task_id, task, true));
		}
	}
	if show_done_tasks {
		for task_id in cached_task_list.done.iter() {
			if let Some(task) = project.done_tasks.get(task_id) {
				done_task_elements.push(task_view(*task_id, task, false));
			}
		}
	}

	if let Some((create_new_task_name, create_new_task_tags)) = &create_new_task {
		let create_new_task_element =
			column![
				task_tags_buttons(
					&project.task_tags,
					create_new_task_tags,
					|tag_id| ProjectPageMessage::ToggleCreateNewTaskTag(tag_id).into()
				),

				row![
					unfocusable(
						text_input("New task name", create_new_task_name)
							.id(CREATE_NEW_TASK_NAME_INPUT_ID.clone())
							.line_height(LineHeight::Relative(1.2))
							.on_input(|input| ProjectPageMessage::ChangeCreateNewTaskName(input).into())
							.on_submit(ProjectPageMessage::CreateNewTask.into())
							.style(theme::TextInput::Custom(Box::new(TextInputStyle {
								// is the first tag enabled?
								round_left_top: project.task_tags
									.iter()
									.next()
									.map(|(tag_id, _tag)|
										!create_new_task_tags.contains(&tag_id)
									)
									.unwrap_or(true),
								round_left_bottom: true,
								..TextInputStyle::NO_ROUNDING
							}))),

						ProjectPageMessage::CloseCreateNewTask.into()
					),

					cancel_create_task_button(),
				]
				.align_items(Alignment::Center)
			]
			.padding(Padding{ top: SPACING_AMOUNT as f32, ..Padding::ZERO })
			.into();

		todo_task_elements.push(create_new_task_element)
	}

	let highlight_bottom_todo_task_dropzone = matches!(hovered_task_dropzone, Some(TaskDropzone::EndOfTodoTaskList));
	todo_task_elements.push(in_between_dropzone(BOTTOM_TODO_TASK_DROPZONE_ID.clone(), highlight_bottom_todo_task_dropzone));

	let show_tasks_button: Element<UiMessage> =
		if cached_task_list.done.is_empty() {
			column![].into()
		}
		else {
			container(
				row![
					show_done_tasks_button(show_done_tasks, cached_task_list.done.len())
				]
				.push_maybe(
					if done_task_elements.is_empty() {
						None
					}
					else {
						Some(
							container(delete_all_done_tasks_button(project_id, &project.name))
								.width(Length::Fill)
								.align_x(Horizontal::Right)
						)
					}
				)
			)
			.padding(Padding{
				left: LARGE_PADDING_AMOUNT,
				right: LARGE_PADDING_AMOUNT,
				top: PADDING_AMOUNT,
				bottom: if show_done_tasks { 0.0 } else { PADDING_AMOUNT },
			})
			.into()
		};

	scrollable(
		column![
			Column::with_children(todo_task_elements)
				.padding(HORIZONTAL_PADDING),

			show_tasks_button,

			Column::with_children(done_task_elements)
				.padding(HORIZONTAL_PADDING),
		]
	)
	.id(TASK_LIST_ID.clone())
	.width(Length::Fill)
	.height(Length::Fill)
	.style(theme::Scrollable::custom(ScrollableStyle))
	.direction(scrollable_vertical_direction())
	.into()
}
