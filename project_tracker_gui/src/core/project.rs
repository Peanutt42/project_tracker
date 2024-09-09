use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use indexmap::IndexMap;
use iced::{widget::container::Id, Color};
use crate::core::{OrderedHashMap, Task, TaskId, TaskType, TaskTag, TaskTagId, SerializableDate};

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
	pub todo_tasks: OrderedHashMap<TaskId, Task>,
	#[serde(with = "indexmap::map::serde_seq")]
	pub done_tasks: IndexMap<TaskId, Task>,
	pub source_code_todos: IndexMap<TaskId, Task>,

	#[serde(skip, default = "Id::unique")]
	pub project_dropzone_id: Id,

	#[serde(skip, default = "Id::unique")]
	pub task_dropzone_id: Id,
}

impl Project {
	pub fn new(name: String, color: SerializableColor) -> Self {
		Self {
			name,
			color,
			task_tags: OrderedHashMap::new(),
			todo_tasks: OrderedHashMap::new(),
			done_tasks: IndexMap::new(),
			source_code_todos: IndexMap::new(),
			project_dropzone_id: Id::unique(),
			task_dropzone_id: Id::unique(),
		}
	}

	/// task can be todo or done
	pub fn get_task(&self, task_id: &TaskId) -> Option<&Task> {
		self.todo_tasks
			.get(task_id)
			.or(self.done_tasks.get(task_id))
			.or(self.source_code_todos.get(task_id))
	}

	/// task can be todo or done or source code todos
	pub fn get_task_mut(&mut self, task_id: &TaskId) -> Option<&mut Task> {
		self.todo_tasks
			.get_mut(task_id)
			.or(self.done_tasks.get_mut(task_id))
			.or(self.source_code_todos.get_mut(task_id))
	}

	pub fn add_task(&mut self, task_id: TaskId, name: String, tags: HashSet<TaskTagId>, create_at_top: bool) {
		let task = Task::new(name, tags);

		if create_at_top {
			self.todo_tasks.insert_at_top(task_id, task);
		}
		else {
			self.todo_tasks.insert(task_id, task);
		}
	}

	/// task can be todo or done or source code todos
	pub fn remove_task(&mut self, task_id: &TaskId) -> Option<(TaskType, Task)> {
		self.todo_tasks
			.remove(task_id)
			.map(|task| (TaskType::Todo, task))
			.or(
				self.done_tasks
					.shift_remove(task_id)
					.map(|task| (TaskType::Done, task))
			)
			.or(
				self.source_code_todos
					.shift_remove(task_id)
					.map(|task| (TaskType::SourceCodeTodo, task))
			)
	}

	pub fn set_task_name(&mut self, task_id: TaskId, new_name: String) {
		if let Some(task) = self.get_task_mut(&task_id) {
			task.name = new_name;
		}
	}

	pub fn set_task_todo(&mut self, task_id: TaskId) {
		if let Some(task) = self.done_tasks.shift_remove(&task_id) {
			self.todo_tasks.insert(task_id, task);
		}
	}

	pub fn set_task_done(&mut self, task_id: TaskId) {
		if let Some(task) = self.todo_tasks.remove(&task_id).or(self.source_code_todos.shift_remove(&task_id)) {
			self.done_tasks.insert(task_id, task);
		}
	}

	pub fn set_task_needed_time(&mut self, task_id: TaskId, new_needed_time_minutes: Option<usize>) {
		if let Some(task) = self.get_task_mut(&task_id) {
			task.needed_time_minutes = new_needed_time_minutes;
		}
	}

	pub fn set_task_due_date(&mut self, task_id: TaskId, new_due_date: Option<SerializableDate>) {
		if let Some(task) = self.get_task_mut(&task_id) {
			task.due_date = new_due_date;
		}
	}

	pub fn toggle_task_tag(&mut self, task_id: TaskId, task_tag_id: TaskTagId) {
		if let Some(task) = self.get_task_mut(&task_id) {
			if task.tags.contains(&task_tag_id) {
				task.tags.remove(&task_tag_id);
			}
			else {
				task.tags.insert(task_tag_id);
			}
		}
	}

	pub fn total_tasks(&self) -> usize {
		self.todo_tasks.len() + self.done_tasks.len() + self.source_code_todos.len()
	}

	pub fn get_completion_percentage(&self) -> f32 {
		let tasks_done = self.done_tasks.len();
		match tasks_done {
			0 => 0.0,
			_ => tasks_done as f32 / self.total_tasks() as f32,
		}
	}


	// ignores iced unique id's, probably only for tests
	pub fn has_same_content_as(&self, other: &Project) -> bool {
		if self.name != other.name ||
			self.color != other.color ||
			self.task_tags != other.task_tags
		{
			return false;
		}

		for (task_id, task) in self.todo_tasks.iter() {
			if let Some(other_task) = other.todo_tasks.get(&task_id) {
				if !task.has_same_content_as(other_task) {
					return false;
				}
			}
			else {
				return false;
			}
		}

		for (task_id, task) in self.done_tasks.iter() {
			if let Some(other_task) = other.done_tasks.get(task_id) {
				if !task.has_same_content_as(other_task) {
					return false;
				}
			}
			else {
				return false;
			}
		}

		for (task_id, task) in self.source_code_todos.iter() {
			if let Some(other_task) = other.source_code_todos.get(task_id) {
				if !task.has_same_content_as(other_task) {
					return false;
				}
			}
			else {
				return false;
			}
		}

		true
	}
}



#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SerializableColor(pub [u8; 3]);

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