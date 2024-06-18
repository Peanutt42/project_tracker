use serde::{Serialize, Deserialize};
use crate::core::{OrderedHashMap, Task, TaskId, TaskState};

use super::generate_task_id;

pub type ProjectId = usize;

pub fn generate_project_id() -> ProjectId {
	rand::random()
}

#[derive(Debug, Clone)]
pub enum ProjectMessage {
	CreateTask(String),
	ChangeTaskName {
		task_id: TaskId,
		new_name: String,
	},
	ChangeTaskState {
		task_id: TaskId,
		new_state: TaskState,
	},
	MoveTaskUp(TaskId),
	MoveTaskDown(TaskId),
	DeleteTask(TaskId),
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

	pub fn add_task(&mut self, task_id: TaskId, name: String) {
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

	pub fn update(&mut self, message: ProjectMessage) {
		match message {
			ProjectMessage::CreateTask(name) => self.add_task(generate_task_id(), name),
			ProjectMessage::ChangeTaskName { task_id, new_name } => {
				if let Some(task) = self.tasks.get_mut(&task_id) {
					task.name = new_name;
				}
			},
			ProjectMessage::ChangeTaskState { task_id, new_state } => {
				if let Some(task) = self.tasks.get_mut(&task_id) {
					task.state = new_state;
				}
				// reorder
				match new_state {
					TaskState::Todo => {
						if let Some(task_order_index) = self.tasks.get_order(&task_id) {
							// put new todo task at the top of the done tasks / at the end of all todo tasks
							for (i, task_id) in self.tasks.iter().enumerate() {
								if self.tasks.get(task_id).unwrap().is_done() {
									if i == 0 {
										self.tasks.order.insert(0, *task_id);
									}
									else {
										self.tasks.order.swap(task_order_index, i - 1);
									}
									break;
								}
							}
						}
					},
					TaskState::Done => {
						self.tasks.move_to_bottom(&task_id);
					},
				}
			},
			ProjectMessage::MoveTaskUp(task_id) => self.tasks.move_up(&task_id),
			ProjectMessage::MoveTaskDown(task_id) => self.tasks.move_down(&task_id),
			ProjectMessage::DeleteTask(task_id) => self.tasks.remove(&task_id),
		}
	}
}
