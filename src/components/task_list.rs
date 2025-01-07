use crate::{
	components::{
		delete_all_done_tasks_button, in_between_dropzone, markdown,
		reimport_source_code_todos_button, show_done_tasks_button, show_source_code_todos_button,
		task_widget, vertical_scrollable,
	},
	core::TaskUiIdMap,
	integrations::CodeEditor,
	pages::{CachedTaskList, TaskDropzone, BOTTOM_TODO_TASK_DROPZONE_ID},
	project_tracker::Message,
	styles::PADDING_AMOUNT,
};
use iced::widget::{container::Id, Space};
use iced::{
	alignment::Horizontal,
	widget::{column, container, row, scrollable, Column},
	Element,
	Length::Fill,
	Padding,
};
use project_tracker_core::{Project, ProjectId, Task, TaskId, TaskType};
use std::collections::HashMap;
use std::sync::LazyLock;

pub static TASK_LIST_ID: LazyLock<scrollable::Id> = LazyLock::new(scrollable::Id::unique);

#[allow(clippy::too_many_arguments)]
pub fn task_list<'a>(
	project_id: ProjectId,
	project: &'a Project,
	cached_task_list: &'a CachedTaskList,
	task_ui_id_map: &'a TaskUiIdMap,
	task_description_markdown_items: &'a HashMap<TaskId, Vec<markdown::Item>>,
	code_editor: Option<&'a CodeEditor>,
	dragged_task: Option<TaskId>,
	just_minimal_dragging: bool,
	hovered_task_dropzone: Option<TaskDropzone>,
	show_done_tasks: bool,
	show_source_code_todos: bool,
	importing_source_code_todos: bool,
) -> Element<'a, Message> {
	let mut todo_task_elements = Vec::new();
	let mut done_task_elements = Vec::new(); // only gets populated when 'show_done_tasks'
	let mut source_code_todo_elements = Vec::new(); // only gets populated when 'show_source_code_todos'

	let task_view = |task_id: TaskId, task: &'a Task, task_type: TaskType| {
		let dragging = match dragged_task {
			Some(dragged_task_id) => dragged_task_id == task_id,
			_ => false,
		};
		let highlight = match hovered_task_dropzone {
			Some(TaskDropzone::Task(hovered_task_id)) => hovered_task_id == task_id,
			_ => false,
		};
		let task_dropzone_id = task_ui_id_map
			.get_dropzone_id(task_id)
			.unwrap_or(Id::unique());
		let task_description_markdown_items = task_description_markdown_items.get(&task_id);
		task_widget(
			task,
			task_id,
			task_dropzone_id,
			task_type,
			task_description_markdown_items,
			project_id,
			project,
			code_editor,
			dragging,
			just_minimal_dragging,
			true,
			highlight,
			true,
		)
	};

	for task_id in cached_task_list.todo.iter() {
		if let Some(task) = project.todo_tasks.get(task_id) {
			todo_task_elements.push(task_view(*task_id, task, TaskType::Todo));
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

	let highlight_bottom_todo_task_dropzone =
		matches!(hovered_task_dropzone, Some(TaskDropzone::EndOfTodoTaskList));
	todo_task_elements.push(in_between_dropzone(
		BOTTOM_TODO_TASK_DROPZONE_ID.clone(),
		highlight_bottom_todo_task_dropzone,
	));

	let show_source_code_todos_button: Element<Message> =
		if cached_task_list.source_code_todo.is_empty() {
			Space::new(0.0, 0.0).into()
		} else {
			container(row![
				show_source_code_todos_button(
					show_source_code_todos,
					cached_task_list.source_code_todo.len()
				),
				container(reimport_source_code_todos_button(
					importing_source_code_todos,
					project.source_code_directory.is_some()
				))
				.width(Fill)
				.align_x(Horizontal::Right)
			])
			.padding(Padding {
				top: PADDING_AMOUNT,
				bottom: if show_source_code_todos {
					0.0
				} else {
					PADDING_AMOUNT
				},
				..Padding::ZERO
			})
			.into()
		};

	let show_tasks_button: Element<Message> = if cached_task_list.done.is_empty() {
		column![].into()
	} else {
		container(
			row![show_done_tasks_button(
				show_done_tasks,
				cached_task_list.done.len()
			)]
			.push_maybe(if done_task_elements.is_empty() {
				None
			} else {
				Some(
					container(delete_all_done_tasks_button(project_id, &project.name))
						.width(Fill)
						.align_x(Horizontal::Right),
				)
			}),
		)
		.padding(Padding {
			top: PADDING_AMOUNT,
			bottom: if show_done_tasks { 0.0 } else { PADDING_AMOUNT },
			..Padding::ZERO
		})
		.into()
	};

	let task_indentation_padding = Padding {
		right: PADDING_AMOUNT,
		..Padding::ZERO
	};

	vertical_scrollable(column![
		Column::with_children(todo_task_elements).padding(task_indentation_padding),
		show_source_code_todos_button,
		Column::with_children(source_code_todo_elements).padding(task_indentation_padding),
		show_tasks_button,
		Column::with_children(done_task_elements).padding(task_indentation_padding),
	])
	.id(TASK_LIST_ID.clone())
	.height(Fill)
	.into()
}
