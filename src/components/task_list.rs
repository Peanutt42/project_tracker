use iced::{widget::{column, scrollable}, theme, Element, Length};
use crate::project_tracker::UiMessage;
use crate::core::{OrderedHashMap, Task, TaskId, ProjectId, TaskFilter};
use crate::styles::{SMALL_SPACING_AMOUNT, ScrollableStyle, scrollable_vertical_direction};

pub fn task_list(tasks: &OrderedHashMap<TaskId, Task>, filter: TaskFilter, project_id: ProjectId, hovered_task_id: Option<TaskId>, project_being_edited_id: Option<TaskId>) -> Element<UiMessage>{
	scrollable(
		column(
			tasks
				.iter()
				.enumerate()
				.filter(|(_i, task_id)| filter.matches(tasks.get(task_id).unwrap()))
				.map(|(i, task_id)| {
					let editing = match project_being_edited_id {
						Some(project_being_edited_id) => *task_id == project_being_edited_id,
						None => false,
					};
					let hovered = match hovered_task_id {
						Some(hovered_task_id) => *task_id == hovered_task_id,
						None => false,
					};
					let can_move_up = i != 0;
					let can_move_down = i != tasks.len() - 1;
					tasks.get(task_id).unwrap().view(project_id, *task_id, editing, hovered, can_move_up, can_move_down)
				})
		)
		.spacing(SMALL_SPACING_AMOUNT)
	)
	.width(Length::Fill)
	.height(Length::Fill)
	.style(theme::Scrollable::custom(ScrollableStyle))
	.direction(scrollable_vertical_direction())
	.into()
}
