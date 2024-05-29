use std::collections::HashMap;
use iced::{widget::{column, scrollable}, Element, Length};
use crate::{project::{ProjectId, TaskFilter}, project_tracker::UiMessage};
use crate::project::{Task, TaskId};
use crate::styles::LARGE_SPACING_AMOUNT;

pub fn task_list(tasks: &HashMap<TaskId, Task>, filter: TaskFilter, project_id: ProjectId) -> Element<UiMessage>{
	scrollable(
		column(
			tasks
				.iter()
				.filter(|(_task_id, task)| filter.matches(task))
				.map(|(task_id, task)| task.view(project_id, *task_id))
		)
		.spacing(LARGE_SPACING_AMOUNT)
	)
	.width(Length::Fill)
	.height(Length::Fill)
	.into()
}
