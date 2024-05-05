use iced::{Element, widget::{column, text}};
use crate::project_tracker::UiMessage;
use crate::components::{task_list, completion_bar};
use crate::task::Task;

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

	pub fn get_tasks_left(&self) -> usize {
		self.tasks
			.iter()
			.filter(|t| !t.is_done())
			.count()
	}

	pub fn get_completion_percentage(&self) -> f32 {
		if self.tasks.is_empty() {
			0.0
		}
		else {
			self.get_tasks_left() as f32 / self.tasks.len() as f32
		}
	}

	pub fn view(&self) -> Element<UiMessage> {
		let tasks_left = self.get_tasks_left();
		let tasks_len = self.tasks.len();

		column![
			text(&self.name).size(21),
			completion_bar(self.get_completion_percentage()),
			text(format!("{tasks_left}/{tasks_len} finished ({}%)", (self.get_completion_percentage() * 100.0).round())),
			task_list(&self.tasks)
		]
		.into()
	}
}
