use std::collections::HashMap;
use iced::{widget::{column, scrollable}, Element, Length};
use crate::{project::{ProjectId, TaskFilter}, project_tracker::UiMessage};
use crate::project::{Task, TaskId};
use crate::styles::LARGE_SPACING_AMOUNT;

pub fn task_list<'a>(tasks: &'a HashMap<TaskId, Task>, task_ordering: &'a [TaskId], filter: TaskFilter, project_id: ProjectId) -> Element<'a, UiMessage>{
	scrollable(
		column(
			task_ordering
				.iter()
				.filter(|task_id| filter.matches(tasks.get(task_id).unwrap()))
				.map(|task_id| tasks.get(task_id).unwrap().view(project_id, *task_id))
		)
		.spacing(LARGE_SPACING_AMOUNT)
	)
	.width(Length::Fill)
	.height(Length::Fill)
	.into()
}
