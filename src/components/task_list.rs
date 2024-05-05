use iced::{Element, widget::{column, scrollable}};
use crate::project_tracker::UiMessage;
use crate::task::Task;

pub fn task_list(tasks: &[Task]) -> Element<UiMessage>{
	scrollable(
		column(
			tasks
				.iter()
				.map(|task| task.view())
		)
	)
	.into()
}
