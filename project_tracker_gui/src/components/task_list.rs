use iced::{alignment::{Alignment, Horizontal}, theme, widget::{column, container, row, scrollable, text::LineHeight, text_input, Column}, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{project_tracker::UiMessage, styles::{LARGE_PADDING_AMOUNT, PADDING_AMOUNT}};
use crate::core::{OrderedHashMap, Task, TaskState, TaskId, generate_task_id, ProjectId, DatabaseMessage};
use crate::components::{show_done_tasks_button, unfocusable, task_widget, custom_task_widget, cancel_create_task_button, delete_all_done_tasks_button};
use crate::styles::{SPACING_AMOUNT, HORIZONTAL_PADDING, ScrollableStyle, TextInputStyle, scrollable_vertical_direction};
use crate::pages::ProjectPageMessage;

pub static TASK_LIST_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
pub static CREATE_NEW_TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub fn task_list<'a>(tasks: &'a OrderedHashMap<TaskId, Task>, project_id: ProjectId, project_name: &str, edited_task: &'a Option<(TaskId, String)>, dragged_task: Option<TaskId>, show_done_tasks: bool, create_new_task_name: &'a Option<String>) -> Element<'a, UiMessage> {
	let mut todo_task_elements = Vec::new();
	let mut done_task_elements = Vec::new(); // only gets populated when 'show_done_tasks'
	let mut done_task_count = 0; // always counts how many, independant of 'show_done_tasks'

	let task_view = |i: usize, task_id: TaskId, task: &'a Task| {
		let can_move_up = i != 0;
		// once there is a done task, all other tasks after that are also done
		let can_move_down = tasks.get_at_order(i + 1).map(|task| task.is_todo()).unwrap_or(false);
		let edited_name = match edited_task {
			Some((edited_task_id, edited_task_name)) if task_id == *edited_task_id => Some(edited_task_name),
			_ => None,
		};
		let dragging = match dragged_task {
			Some(dragged_task_id) => dragged_task_id == task_id,
			_ => false,
		};
		task_widget(task, task_id, project_id, edited_name, dragging, can_move_up, can_move_down)
	};

	for (i, (task_id, task)) in tasks.iter().enumerate() {
		if task.is_todo() {
			todo_task_elements.push(task_view(i, task_id, task));
		}
		else {
			done_task_count += 1;
			if show_done_tasks {
				done_task_elements.push(task_view(i, task_id, task));
			}
		}
	}

	if let Some(create_new_task_name) = &create_new_task_name {
		let inner_text_element =
			row![
				unfocusable(
					text_input("New task name", create_new_task_name)
						.id(CREATE_NEW_TASK_NAME_INPUT_ID.clone())
						.line_height(LineHeight::Relative(1.2))
						.on_input(|input| ProjectPageMessage::ChangeCreateNewTaskName(input).into())
						.on_submit(DatabaseMessage::CreateTask {
							project_id,
							task_id: generate_task_id(),
							task_name: create_new_task_name.clone(),
						}.into())
						.style(theme::TextInput::Custom(Box::new(TextInputStyle))),

					ProjectPageMessage::CloseCreateNewTask.into()
				),

				cancel_create_task_button(),
			]
			.align_items(Alignment::Center)
			.into();

		todo_task_elements.push(custom_task_widget(inner_text_element, TaskState::Todo, None, project_id, false, false, false, false))
	}

	let show_tasks_button: Element<UiMessage> =
		if done_task_count == 0 {
			column![].into()
		}
		else {
			container(
				row![
					show_done_tasks_button(show_done_tasks, done_task_count)
				]
				.push_maybe(
					if done_task_elements.is_empty() {
						None
					}
					else {
						Some(
							container(delete_all_done_tasks_button(project_id, project_name))
								.width(Length::Fill)
								.align_x(Horizontal::Right)
						)
					}
				)
			)
			.padding(Padding{ left: LARGE_PADDING_AMOUNT, right: PADDING_AMOUNT, top: LARGE_PADDING_AMOUNT, bottom: 0.0 })
			.into()
		};

	scrollable(
		column![
			Column::with_children(todo_task_elements)
				.spacing(SPACING_AMOUNT)
				.padding(HORIZONTAL_PADDING),

			show_tasks_button,

			Column::with_children(done_task_elements)
				.spacing(SPACING_AMOUNT)
				.padding(HORIZONTAL_PADDING),
		]
		.spacing(SPACING_AMOUNT)
	)
	.id(TASK_LIST_ID.clone())
	.width(Length::Fill)
	.height(Length::Fill)
	.style(theme::Scrollable::custom(ScrollableStyle))
	.direction(scrollable_vertical_direction())
	.into()
}
