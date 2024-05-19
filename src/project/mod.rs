use iced::{widget::{column, text}, Element};
use serde::{Serialize, Deserialize};
use crate::{components::{task_list, completion_bar}, project_tracker::UiMessage, styles::{HORIZONTAL_PADDING, SPACING_AMOUNT, TITLE_TEXT_SIZE}};

mod task;
pub use task::Task;

mod task_state;
pub use task_state::TaskState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
	pub name: String,
	pub tasks: Vec<Task>,
}

impl Project {
	pub fn new(name: String, tasks: Vec<Task>) -> Self {
		Self {
			name,
			tasks,
		}
	}

	pub fn get_tasks_done(&self) -> usize {
		self.tasks
			.iter()
			.filter(|t| t.is_done())
			.count()
	}

	fn calculate_completion_percentage(tasks_done: usize, task_count: usize) -> f32 {
		if task_count == 0 {
			0.0
		}
		else {
			tasks_done as f32 / task_count as f32
		}
	}

	pub fn get_completion_percentage(&self) -> f32 {
		Self::calculate_completion_percentage(self.get_tasks_done(), self.tasks.len())
	}

	pub fn view(&self) -> Element<UiMessage> {
		let tasks_done = self.get_tasks_done();
		let tasks_len = self.tasks.len();
		let completion_percentage = Self::calculate_completion_percentage(tasks_done, tasks_len);

		column![
			text(&self.name).size(TITLE_TEXT_SIZE),
			completion_bar(completion_percentage),
			text(format!("{tasks_done}/{tasks_len} finished ({}%)", (completion_percentage * 100.0).round())),
			task_list(&self.tasks)
		]
		.spacing(SPACING_AMOUNT)
		.padding(HORIZONTAL_PADDING)
		.into()
	}
}
