use crate::{
	ordered_hash_map::OrderedHashMapIter, OrderedHashMap, SerializableDate, Task, TaskId, TaskTag,
	TaskTagId, TaskType, TimeSpend,
};
use indexmap::{
	map::{Iter, ValuesMut},
	IndexMap,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub Uuid);

impl ProjectId {
	pub fn generate() -> Self {
		Self(Uuid::new_v4())
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortMode {
	#[default]
	Manual,
	DueDate,
	NeededTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
	pub name: String,
	pub color: SerializableColor,
	pub sort_mode: SortMode,
	pub task_tags: OrderedHashMap<TaskTagId, TaskTag>,
	pub todo_tasks: OrderedHashMap<TaskId, Task>,
	#[serde(with = "indexmap::map::serde_seq")]
	pub done_tasks: IndexMap<TaskId, Task>,
	#[serde(with = "indexmap::map::serde_seq")]
	pub source_code_todos: IndexMap<TaskId, Task>,
	#[serde(default)]
	pub source_code_directory: Option<PathBuf>,
}

impl Project {
	pub fn new(
		name: String,
		color: SerializableColor,
		task_tags: OrderedHashMap<TaskTagId, TaskTag>,
		sort_mode: SortMode,
	) -> Self {
		Self {
			name,
			color,
			task_tags,
			sort_mode,
			todo_tasks: OrderedHashMap::new(),
			done_tasks: IndexMap::new(),
			source_code_todos: IndexMap::new(),
			source_code_directory: None,
		}
	}

	/// task can be todo or done
	pub fn get_task(&self, task_id: &TaskId) -> Option<&Task> {
		self.todo_tasks
			.get(task_id)
			.or(self.done_tasks.get(task_id))
			.or(self.source_code_todos.get(task_id))
	}

	pub fn get_task_and_type(&self, task_id: &TaskId) -> Option<(&Task, TaskType)> {
		self.todo_tasks
			.get(task_id)
			.map(|t| (t, TaskType::Todo))
			.or(self.done_tasks.get(task_id).map(|t| (t, TaskType::Done)))
			.or(self
				.source_code_todos
				.get(task_id)
				.map(|t| (t, TaskType::SourceCodeTodo)))
	}

	/// task can be todo or done or source code todos
	pub fn get_task_mut(&mut self, task_id: &TaskId) -> Option<&mut Task> {
		self.todo_tasks
			.get_mut(task_id)
			.or(self.done_tasks.get_mut(task_id))
			.or(self.source_code_todos.get_mut(task_id))
	}

	#[allow(clippy::too_many_arguments)]
	pub fn add_task(
		&mut self,
		task_id: TaskId,
		name: String,
		description: String,
		tags: HashSet<TaskTagId>,
		due_date: Option<SerializableDate>,
		needed_time_minutes: Option<usize>,
		time_spend: Option<TimeSpend>,
		create_at_top: bool,
	) {
		let task = Task::new(
			name,
			description,
			needed_time_minutes,
			time_spend,
			due_date,
			tags,
		);

		if create_at_top {
			self.todo_tasks.insert_at_top(task_id, task);
		} else {
			self.todo_tasks.insert(task_id, task);
		}
	}

	/// task can be todo or done or source code todos
	pub fn remove_task(&mut self, task_id: &TaskId) -> Option<(TaskType, Task)> {
		self.todo_tasks
			.remove(task_id)
			.map(|task| (TaskType::Todo, task))
			.or(self
				.done_tasks
				.shift_remove(task_id)
				.map(|task| (TaskType::Done, task)))
			.or(self
				.source_code_todos
				.shift_remove(task_id)
				.map(|task| (TaskType::SourceCodeTodo, task)))
	}

	pub fn set_task_name(&mut self, task_id: TaskId, new_name: String) {
		if let Some(task) = self.get_task_mut(&task_id) {
			task.name = new_name;
		}
	}

	pub fn set_task_description(&mut self, task_id: TaskId, new_description: String) {
		if let Some(task) = self.get_task_mut(&task_id) {
			task.description = new_description;
		}
	}

	pub fn set_task_todo(&mut self, task_id: TaskId) {
		if let Some(task) = self.done_tasks.shift_remove(&task_id) {
			self.todo_tasks.insert(task_id, task);
		}
	}

	pub fn set_task_done(&mut self, task_id: TaskId) {
		if let Some(task) = self
			.todo_tasks
			.remove(&task_id)
			.or(self.source_code_todos.shift_remove(&task_id))
		{
			self.done_tasks.insert(task_id, task);
		}
	}

	pub fn set_task_needed_time(
		&mut self,
		task_id: TaskId,
		new_needed_time_minutes: Option<usize>,
	) {
		if let Some(task) = self.get_task_mut(&task_id) {
			task.needed_time_minutes = new_needed_time_minutes;
		}
	}

	pub fn set_task_time_spend(&mut self, task_id: TaskId, new_time_spend: Option<TimeSpend>) {
		if let Some(task) = self.get_task_mut(&task_id) {
			task.time_spend = new_time_spend;
		}
	}

	pub fn start_task_time_spend(&mut self, task_id: TaskId) {
		if let Some(task) = self.get_task_mut(&task_id) {
			match &mut task.time_spend {
				Some(time_spend) => time_spend.start(),
				None => {
					let mut time_spend = TimeSpend::new(0.0);
					time_spend.start();
					task.time_spend = Some(time_spend);
				}
			}
		}
	}

	pub fn stop_task_time_spend(&mut self, task_id: TaskId) {
		if let Some(task) = self.get_task_mut(&task_id) {
			if let Some(time_spend) = &mut task.time_spend {
				time_spend.stop();
			}
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
			} else {
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

	pub fn iter(&self) -> TaskIter {
		TaskIter {
			todo_tasks_iter: self.todo_tasks.iter(),
			done_tasks_iter: self.done_tasks.iter(),
			source_code_tasks_iter: self.source_code_todos.iter(),
		}
	}

	pub fn values_mut(&mut self) -> TaskValueIterMut {
		TaskValueIterMut {
			todo_tasks_valuse: self.todo_tasks.values_mut(),
			done_tasks_values: self.done_tasks.values_mut(),
			source_code_tasks_values: self.source_code_todos.values_mut(),
		}
	}
}

pub struct TaskIter<'a> {
	todo_tasks_iter: OrderedHashMapIter<'a, TaskId, Task>,
	done_tasks_iter: Iter<'a, TaskId, Task>,
	source_code_tasks_iter: Iter<'a, TaskId, Task>,
}

impl<'a> Iterator for TaskIter<'a> {
	type Item = (TaskId, &'a Task, TaskType);

	fn next(&mut self) -> Option<Self::Item> {
		self.todo_tasks_iter
			.next()
			.map(|(task_id, task)| (task_id, task, TaskType::Todo))
			.or(self
				.done_tasks_iter
				.next()
				.map(|(task_id, task)| (*task_id, task, TaskType::Done)))
			.or(self
				.source_code_tasks_iter
				.next()
				.map(|(task_id, task)| (*task_id, task, TaskType::SourceCodeTodo)))
	}
}

pub struct TaskValueIterMut<'a> {
	todo_tasks_valuse: std::collections::hash_map::ValuesMut<'a, TaskId, Task>,
	done_tasks_values: ValuesMut<'a, TaskId, Task>,
	source_code_tasks_values: ValuesMut<'a, TaskId, Task>,
}

impl<'a> Iterator for TaskValueIterMut<'a> {
	type Item = &'a mut Task;

	fn next(&mut self) -> Option<Self::Item> {
		self.todo_tasks_valuse
			.next()
			.or(self.done_tasks_values.next())
			.or(self.source_code_tasks_values.next())
	}
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SerializableColor(pub [u8; 3]);
