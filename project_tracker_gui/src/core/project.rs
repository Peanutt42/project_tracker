use serde::{Serialize, Deserialize};
use crate::core::{OrderedHashMap, Task, TaskId, TaskState};


pub type ProjectId = usize;

pub fn generate_project_id() -> ProjectId {
	rand::random()
}

#[derive(Debug, Clone)]
pub enum ProjectMessage {
	CreateTask(String),
	ChangeTaskName(String),
	ChangeTaskState(TaskState),
	MoveTaskUp,
	MoveTaskDown,
	DeleteTask,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

	pub fn update(&mut self, task_id: TaskId, message: ProjectMessage) {
		match message {
			ProjectMessage::CreateTask(name) => self.add_task(task_id, name),
			ProjectMessage::ChangeTaskName(new_name) => {
				if let Some(task) = self.tasks.get_mut(&task_id) {
					task.name = new_name;
				}
			},
			ProjectMessage::ChangeTaskState(new_state) => {
				// reorder
				match new_state {
					TaskState::Todo => {
						// put new todo task at the top of the done tasks / at the end of all todo tasks
						for (i, (_task_id, task)) in self.tasks.iter().enumerate() {
							if task.is_done() {
								self.tasks.move_to(task_id, i);
								break;
							}
						}
					},
					TaskState::Done => {
						self.tasks.move_to_bottom(&task_id);
					},
				}

				if let Some(task) = self.tasks.get_mut(&task_id) {
					task.state = new_state;
				}
			},
			ProjectMessage::MoveTaskUp => self.tasks.move_up(&task_id),
			ProjectMessage::MoveTaskDown => self.tasks.move_down(&task_id),
			ProjectMessage::DeleteTask => self.tasks.remove(&task_id),
		}
	}
}
