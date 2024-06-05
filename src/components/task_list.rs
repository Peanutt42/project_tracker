use iced::{widget::{column, scrollable}, theme, Element, Length};
use crate::project_tracker::UiMessage;
use crate::core::{OrderedHashMap, Task, TaskId, ProjectId, TaskFilter};
use crate::styles::{LARGE_SPACING_AMOUNT, ScrollableStyle, scrollable_vertical_direction};

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
	.style(theme::Scrollable::custom(ScrollableStyle))
	.direction(scrollable_vertical_direction())
	.into()
}
