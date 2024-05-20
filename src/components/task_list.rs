use iced::{Element, widget::{column, scrollable}};
use crate::project_tracker::UiMessage;
use crate::project::Task;
use crate::styles::SPACING_AMOUNT;

pub fn task_list<'a>(tasks: &'a [Task], project_name: &'a str) -> Element<'a, UiMessage>{
	scrollable(
		column(
			tasks
				.iter()
				.map(|task| task.view(project_name))
		)
		.spacing(SPACING_AMOUNT)
	)
	.into()
}
