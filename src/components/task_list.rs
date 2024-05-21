use iced::{widget::{column, scrollable}, Element, Length};
use crate::{project::TaskFilter, project_tracker::UiMessage};
use crate::project::Task;
use crate::styles::LARGE_SPACING_AMOUNT;

pub fn task_list<'a>(tasks: &'a [Task], filter: TaskFilter, project_name: &'a str) -> Element<'a, UiMessage>{
	scrollable(
		column(
			tasks
				.iter()
				.filter(|t| filter.matches(t))
				.map(|task| task.view(project_name))
		)
		.spacing(LARGE_SPACING_AMOUNT)
	)
	.width(Length::Fill)
	.into()
}
