use std::collections::HashSet;
use iced::{alignment::{Alignment, Horizontal}, widget::{column, container, row, scrollable, text::LineHeight, text_input, Column}, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{core::{DateFormatting, Project, TaskTagId, TaskType}, pages::{CachedTaskList, EditTaskState, StopwatchPage, TaskDropzone, BOTTOM_TODO_TASK_DROPZONE_ID}, project_tracker::UiMessage, styles::PADDING_AMOUNT};
use crate::core::{Task, TaskId, ProjectId};
use crate::components::{vertical_scrollable, show_done_tasks_button, show_source_code_todos_button, unfocusable, task_widget, cancel_create_task_button, delete_all_done_tasks_button, reimport_source_code_todos_button, task_tags_buttons, in_between_dropzone};
use crate::styles::{SPACING_AMOUNT, text_input_style};
use crate::pages::ProjectPageMessage;

pub static TASK_LIST_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
pub static CREATE_NEW_TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[allow(clippy::too_many_arguments)]
pub fn task_list<'a>(project_id: ProjectId, project: &'a Project, cached_task_list: &'a CachedTaskList, edit_task_state: &'a Option<EditTaskState>, dragged_task: Option<TaskId>, just_minimal_dragging: bool, hovered_task_dropzone: Option<TaskDropzone>, show_done_tasks: bool, show_source_code_todos: bool, create_new_task: &'a Option<(String, HashSet<TaskTagId>)>, stopwatch_page: &'a StopwatchPage, date_formatting: DateFormatting, create_new_tasks_at_top: bool) -> Element<'a, UiMessage> {
	let mut todo_task_elements = Vec::new();
	let mut done_task_elements = Vec::new(); // only gets populated when 'show_done_tasks'
	let mut source_code_todo_elements = Vec::new(); // only gets populated when 'show_source_code_todos'

	let task_view = |task_id: TaskId, task: &'a Task, task_type: TaskType| {
		let edit_task_state = match edit_task_state {
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
		let stopwatch_label = match stopwatch_page {
			StopwatchPage::Idle => None,
			StopwatchPage::Ticking { task, clock, .. } => {
				task.as_ref().and_then(|(timed_project_id, timed_task_id)| {
					if *timed_project_id == project_id && *timed_task_id == task_id {
						Some(clock.label())
					}
					else {
						None
					}
				})
			},
		};
		task_widget(task, task_id, task_type, project_id, &project.task_tags, edit_task_state, dragging, just_minimal_dragging, highlight, stopwatch_label, date_formatting)
	};

	if create_new_tasks_at_top {
		if let Some((create_new_task_name, create_new_task_tags)) = create_new_task {
			todo_task_elements.push(create_new_task_element(project, create_new_task_name, create_new_task_tags));
		}
	}
	for task_id in cached_task_list.todo.iter() {
		if let Some(task) = project.todo_tasks.get(task_id) {
			todo_task_elements.push(task_view(*task_id, task, TaskType::Todo));
		}
	}

	if !create_new_tasks_at_top {
		if let Some((create_new_task_name, create_new_task_tags)) = create_new_task {
			todo_task_elements.push(create_new_task_element(project, create_new_task_name, create_new_task_tags));
		}
	}
	if show_done_tasks {
		for task_id in cached_task_list.done.iter() {
			if let Some(task) = project.done_tasks.get(task_id) {
				done_task_elements.push(task_view(*task_id, task, TaskType::Done));
			}
		}
	}
	if show_source_code_todos {
		for task_id in cached_task_list.source_code_todo.iter() {
			if let Some(task) = project.source_code_todos.get(task_id) {
				source_code_todo_elements.push(task_view(*task_id, task, TaskType::SourceCodeTodo));
			}
		}
	}

	let highlight_bottom_todo_task_dropzone = matches!(hovered_task_dropzone, Some(TaskDropzone::EndOfTodoTaskList));
	todo_task_elements.push(in_between_dropzone(BOTTOM_TODO_TASK_DROPZONE_ID.clone(), highlight_bottom_todo_task_dropzone));

	let show_source_code_todos_button: Element<UiMessage> =
			if cached_task_list.source_code_todo.is_empty() {
				column![].into()
			}
			else {
				container(
					row![
						show_source_code_todos_button(show_source_code_todos, cached_task_list.source_code_todo.len())
					]
					.push_maybe(
						if source_code_todo_elements.is_empty() {
							None
						}
						else {
							Some(
								container(reimport_source_code_todos_button())
									.width(Length::Fill)
									.align_x(Horizontal::Right)
							)
						}
					)
				)
				.padding(Padding{
					top: PADDING_AMOUNT,
					bottom: if show_source_code_todos { 0.0 } else { PADDING_AMOUNT },
					..Padding::ZERO
				})
				.into()
			};


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
				top: PADDING_AMOUNT,
				bottom: if show_done_tasks { 0.0 } else { PADDING_AMOUNT },
				..Padding::ZERO
			})
			.into()
		};

	let task_indentation_padding = Padding {
		left: PADDING_AMOUNT,
		right: PADDING_AMOUNT,
		..Padding::ZERO
	};

	vertical_scrollable(
		column![
			Column::with_children(todo_task_elements).padding(task_indentation_padding),

			show_source_code_todos_button,

			Column::with_children(source_code_todo_elements).padding(task_indentation_padding),

			show_tasks_button,

			Column::with_children(done_task_elements).padding(task_indentation_padding),
		]
	)
	.id(TASK_LIST_ID.clone())
	.height(Length::Fill)
	.into()
}

fn create_new_task_element<'a>(project: &'a Project, create_new_task_name: &'a str, create_new_task_tags: &'a HashSet<TaskTagId>) -> Element<'a, UiMessage> {
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
					.style(move |t, s| text_input_style(
						t,
						s,
						// is the first tag enabled?
						project.task_tags
							.iter()
							.next()
							.map(|(tag_id, _tag)|
								!create_new_task_tags.contains(&tag_id)
							)
							.unwrap_or(true),
						false,
						false,
						true
					)),

				ProjectPageMessage::CloseCreateNewTask.into()
			),

			cancel_create_task_button(),
		]
		.align_y(Alignment::Center)
	]
	.padding(Padding{ top: SPACING_AMOUNT as f32, ..Padding::ZERO })
	.into()
}