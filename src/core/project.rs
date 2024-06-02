use serde::{Serialize, Deserialize};
use crate::core::{OrderedHashMap, TaskId, generate_task_id, Task, TaskState};

pub type ProjectId = usize;

pub fn generate_project_id() -> ProjectId {
	rand::random()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
	pub name: String,
	pub tasks: OrderedHashMap<TaskId, Task>,
}

impl Project {
	pub fn new(name: String) -> Self {
		Self {
			name,
			tasks: OrderedHashMap::new(),
		}
	}

	pub fn add_task(&mut self, name: String) {
		let task_id = generate_task_id();
		self.tasks.insert(task_id, Task::new(name, TaskState::Todo));
	}

	pub fn get_tasks_done(&self) -> usize {
		self.tasks
			.values()
			.filter(|t| t.is_done())
			.count()
	}

	pub fn calculate_completion_percentage(tasks_done: usize, task_count: usize) -> f32 {
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
}