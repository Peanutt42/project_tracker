use std::collections::HashMap;
use iced::{widget::{column, scrollable}, Element, Length};
use crate::{project::{ProjectId, TaskFilter}, project_tracker::UiMessage};
use crate::project::{Task, TaskId};
use crate::styles::LARGE_SPACING_AMOUNT;

pub fn task_list(tasks: &HashMap<TaskId, Task>, filter: TaskFilter, project_id: ProjectId) -> Element<UiMessage>{
	scrollable(
		column(
			tasks
				.values()
				.filter(|t| filter.matches(t))
				.map(|task| task.view(project_id))
		)
		.spacing(LARGE_SPACING_AMOUNT)
	)
	.width(Length::Fill)
	.height(Length::Fill)
	.into()
}
