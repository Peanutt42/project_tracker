use std::collections::BTreeSet;

use serde::{Serialize, Deserialize};
use iced::{widget::container::Id, Color};
use crate::core::{OrderedHashMap, Task, TaskId, TaskState, TaskTag, TaskTagId, SerializableDate};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub usize);

impl ProjectId {
	pub fn generate() -> Self {
		Self(rand::random())
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
	pub name: String,
	pub color: SerializableColor,
	pub task_tags: OrderedHashMap<TaskTagId, TaskTag>,
	pub tasks: OrderedHashMap<TaskId, Task>,

	#[serde(skip, default = "Id::unique")]
	pub preview_dropzone_id: Id,
}

impl Project {
	pub fn new(name: String) -> Self {
		Self {
			name,
			color: SerializableColor::default(),
			task_tags: OrderedHashMap::new(),
			tasks: OrderedHashMap::new(),
			preview_dropzone_id: Id::unique(),
		}
	}

	pub fn add_task(&mut self, task_id: TaskId, name: String, tags: BTreeSet<TaskTagId>) {
		self.tasks.insert(task_id, Task::new(name, TaskState::Todo, tags));
	}

	pub fn set_task_name(&mut self, task_id: TaskId, new_name: String) {
		if let Some(task) = self.tasks.get_mut(&task_id) {
			task.name = new_name;
		}
	}

	pub fn set_task_state(&mut self, task_id: TaskId, new_state: TaskState) {
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
	}

	pub fn set_task_needed_time(&mut self, task_id: TaskId, new_needed_time_minutes: Option<usize>) {
		if let Some(task) = self.tasks.get_mut(&task_id) {
			task.needed_time_minutes = new_needed_time_minutes;
		}
	}

	pub fn set_task_due_date(&mut self, task_id: TaskId, new_due_date: Option<SerializableDate>) {
		if let Some(task) = self.tasks.get_mut(&task_id) {
			task.due_date = new_due_date;
		}
	}

	pub fn toggle_task_tag(&mut self, task_id: TaskId, task_tag_id: TaskTagId) {
		if let Some(task) = self.tasks.get_mut(&task_id) {
			if task.tags.contains(&task_tag_id) {
				task.tags.remove(&task_tag_id);
			}
			else {
				task.tags.insert(task_tag_id);
			}
		}
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



#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SerializableColor([u8; 3]);

impl From<SerializableColor> for Color {
	fn from(value: SerializableColor) -> Self {
		Color::from_rgb8(value.0[0], value.0[1], value.0[2])
	}
}

impl From<Color> for SerializableColor {
	fn from(value: Color) -> Self {
		Self ([
			(value.r * 255.0) as u8,
			(value.g * 255.0) as u8,
			(value.b * 255.0) as u8
		])
	}
}