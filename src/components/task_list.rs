use iced::{Element, widget::{column, scrollable}};
use crate::project_tracker::UiMessage;
use crate::project::Task;
use crate::styles::SPACING_AMOUNT;

pub fn task_list(tasks: &[Task]) -> Element<UiMessage>{
	scrollable(
		column(
			tasks
				.iter()
				.map(|task| task.view())
		)
		.spacing(SPACING_AMOUNT)
	)
	.into()
}
