use iced::{widget::{column, scrollable}, Element, Length};
use crate::project_tracker::UiMessage;
use crate::core::{OrderedHashMap, Task, TaskId, ProjectId, TaskFilter};
use crate::styles::LARGE_SPACING_AMOUNT;

pub fn task_list(tasks: &OrderedHashMap<TaskId, Task>, filter: TaskFilter, project_id: ProjectId) -> Element<UiMessage>{
	scrollable(
		column(
			tasks
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
