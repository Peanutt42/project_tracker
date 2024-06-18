use iced::{theme, widget::{column, container, scrollable, text_input, text::LineHeight, Column, row}, alignment::Alignment, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{core::{generate_task_id, TaskMessage, TaskState}, project_tracker::UiMessage, styles::LARGE_PADDING_AMOUNT};
use crate::core::{OrderedHashMap, Task, TaskId, ProjectId};
use crate::components::{show_done_tasks_button, task_widget, custom_task_widget, cancel_create_task_button};
use crate::styles::{SMALL_SPACING_AMOUNT, SPACING_AMOUNT, HORIZONTAL_PADDING, MIDDLE_TEXT_SIZE, ScrollableStyle, TextInputStyle, scrollable_vertical_direction};
use crate::pages::ProjectPageMessage;

pub static CREATE_NEW_TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub fn task_list<'a>(tasks: &'a OrderedHashMap<TaskId, Task>, project_id: ProjectId, hovered_task_id: Option<TaskId>, project_being_edited_id: Option<TaskId>, show_done_tasks: bool, create_new_task_name: &'a Option<String>) -> Element<'a, UiMessage> {
	let mut todo_tasks = Vec::new();
	let mut done_tasks = Vec::new(); // only gets populated when 'show_done_tasks'
	let mut done_task_count = 0; // always counts how many, independant of 'show_done_tasks'

	let task_view = |i: usize, task_id: TaskId, task: &'a Task| {
		let editing = match project_being_edited_id {
			Some(project_being_edited_id) => task_id == project_being_edited_id,
			None => false,
		};
		let hovered = match hovered_task_id {
			Some(hovered_task_id) => task_id == hovered_task_id,
			None => false,
		};
		let can_move_up = i != 0;
		// once there is a done task, all other tasks after that are also done
		let can_move_down = i < tasks.len() - 1 && if let Some(task) = tasks.get(&tasks.order[i + 1]) { task.is_todo() } else { false };
		task_widget(task, task_id, project_id, editing, hovered, can_move_up, can_move_down)
	};

	for (i, task_id) in tasks.iter().enumerate() {
		if let Some(task) = tasks.get(task_id) {
			if task.is_todo() {
				todo_tasks.push(task_view(i, *task_id, task));
			}
			else {
				done_task_count += 1;
				if show_done_tasks {
					done_tasks.push(task_view(i, *task_id, task));
				}
			}
		}
	}

	if let Some(create_new_task_name) = &create_new_task_name {
		let inner_text_element =
			row![
				text_input("New task name", create_new_task_name)
					.id(CREATE_NEW_TASK_NAME_INPUT_ID.clone())
					.size(MIDDLE_TEXT_SIZE)
					.line_height(LineHeight::Relative(1.2))
					.on_input(|input| ProjectPageMessage::ChangeCreateNewTaskName(input).into())
					.on_submit(TaskMessage::Create(create_new_task_name.clone()).to_ui_message(project_id, generate_task_id()))
					.style(theme::TextInput::Custom(Box::new(TextInputStyle))),

				cancel_create_task_button(),
			]
			.align_items(Alignment::Center)
			.into();

		todo_tasks.push(custom_task_widget(inner_text_element, TaskState::Todo, None, project_id, false, false, false, false))
	}

	let show_tasks_button: Element<UiMessage> =
		if done_task_count == 0 {
			column![].into()
		}
		else {
			container(show_done_tasks_button(show_done_tasks, done_task_count))
				.padding(Padding{ left: LARGE_PADDING_AMOUNT, top: LARGE_PADDING_AMOUNT, ..Padding::ZERO })
				.into()
		};

	scrollable(
		column![
			Column::with_children(todo_tasks)
				.spacing(SMALL_SPACING_AMOUNT)
				.padding(HORIZONTAL_PADDING),

			show_tasks_button,

			Column::with_children(done_tasks)
				.spacing(SMALL_SPACING_AMOUNT)
				.padding(HORIZONTAL_PADDING),
		]
		.spacing(SPACING_AMOUNT)
	)
	.width(Length::Fill)
	.height(Length::Fill)
	.style(theme::Scrollable::custom(ScrollableStyle))
	.direction(scrollable_vertical_direction())
	.into()
}
