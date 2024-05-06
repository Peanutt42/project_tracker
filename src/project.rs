use iced::{Element, widget::{column, text}};
use iced_aw::modal;
use serde::{Serialize, Deserialize};
use crate::{components::{create_new_task_button, CreateNewTaskModal}, project_tracker::UiMessage};
use crate::components::{task_list, completion_bar};
use crate::task::Task;

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
			self.get_tasks_done() as f32 / self.tasks.len() as f32
		}
	}

	pub fn view<'a>(&'a self, create_new_task_modal: &'a CreateNewTaskModal) -> Element<UiMessage> {
		let tasks_left = self.get_tasks_left();
		let tasks_len = self.tasks.len();

		let project_view = column![
			text(&self.name).size(21),
			completion_bar(self.get_completion_percentage()),
			text(format!("{tasks_left}/{tasks_len} finished ({}%)", (self.get_completion_percentage() * 100.0).round())),
			create_new_task_button(),
			task_list(&self.tasks)
		];

		modal(project_view, create_new_task_modal.view(self.name.clone()))
			.into()
	}
}
