use crate::components::markdown::{self, parse_markdown};
use project_tracker_core::{Database, ProjectId, Task, TaskId};
use std::{
	cell::RefCell,
	collections::HashMap,
	hash::{DefaultHasher, Hash, Hasher},
	sync::Arc,
};

#[derive(Debug, Clone)]
struct CachedTaskDescriptionMarkdownItems {
	task_description_hash: u64,
	items: Arc<[markdown::Item]>,
}

#[derive(Debug, Clone, Default)]
pub struct TaskDescriptionMarkdownStorage {
	storage: RefCell<HashMap<TaskId, CachedTaskDescriptionMarkdownItems>>,
}

impl TaskDescriptionMarkdownStorage {
	pub fn get(
		&self,
		project_id: ProjectId,
		task_id: TaskId,
		database: Option<&Database>,
	) -> Option<Arc<[markdown::Item]>> {
		database.and_then(|db| {
			db.get_task(&project_id, &task_id)
				.map(|task| self.get_from_task(task_id, task))
		})
	}

	pub fn get_from_task(&self, task_id: TaskId, task: &Task) -> Arc<[markdown::Item]> {
		self.get_from_str(task_id, &task.description)
	}

	pub fn get_from_str(&self, task_id: TaskId, task_description: &str) -> Arc<[markdown::Item]> {
		let mut hasher = DefaultHasher::new();
		task_description.hash(&mut hasher);
		let task_description_hash = hasher.finish();

		self.storage
			.borrow_mut()
			.entry(task_id)
			.and_modify(|cached| {
				if cached.task_description_hash != task_description_hash {
					cached.items = parse_markdown(task_description);
				}
			})
			.or_insert_with(|| {
				let items = parse_markdown(task_description);
				CachedTaskDescriptionMarkdownItems {
					task_description_hash,
					items,
				}
			})
			.items
			.clone()
	}
}
