mod task_tag;
use std::collections::HashMap;

use project_tracker_core::{ProjectId, TaskId};
pub use task_tag::TASK_TAG_QUAD_HEIGHT;

mod date;
pub use date::{formatted_date_time, SerializableDateConversion};

mod project;
pub use project::{IcedColorConversion, SortModeUI};

mod database;
pub use database::{
	export_database_as_json_file_dialog, export_database_as_markdown_file_dialog,
	export_database_file_dialog, import_database_file_dialog, import_json_database_file_dialog,
};

mod source_code_todo;
pub use source_code_todo::import_source_code_todos;

use crate::project_tracker::Message;
use iced::{advanced::widget, widget::container::Id, Element};

pub trait View {
	fn view(&self) -> Element<Message>;
}

#[derive(Debug, Clone)]
struct ProjectUiIds {
	project_dropzone_id: Id,
	task_dropzone_id: Id,
}
impl Default for ProjectUiIds {
	fn default() -> Self {
		Self {
			project_dropzone_id: Id::unique(),
			task_dropzone_id: Id::unique(),
		}
	}
}

#[derive(Debug, Default)]
pub struct ProjectUiIdMap {
	project_ids: HashMap<ProjectId, ProjectUiIds>,
}

impl ProjectUiIdMap {
	pub fn get_project_dropzone_id(&self, project_id: ProjectId) -> Id {
		self.project_ids
			.get(&project_id)
			.map(|ids| ids.project_dropzone_id.clone())
			.unwrap_or(Id::unique())
	}

	pub fn get_task_dropzone_id(&self, project_id: ProjectId) -> Id {
		self.project_ids
			.get(&project_id)
			.map(|ids| ids.task_dropzone_id.clone())
			.unwrap_or(Id::unique())
	}

	pub fn get_project_dropzone_id_mut(&mut self, project_id: ProjectId) -> Id {
		self.project_ids
			.entry(project_id)
			.or_default()
			.project_dropzone_id
			.clone()
	}

	pub fn get_task_dropzone_id_mut(&mut self, project_id: ProjectId) -> Id {
		self.project_ids
			.entry(project_id)
			.or_default()
			.task_dropzone_id
			.clone()
	}

	pub fn get_project_task_dropzone_ids(&self, project_id: ProjectId) -> (Id, Id) {
		let project_ui_ids = self
			.project_ids
			.get(&project_id)
			.cloned()
			.unwrap_or_default();
		(
			project_ui_ids.project_dropzone_id,
			project_ui_ids.task_dropzone_id,
		)
	}
}

#[derive(Debug, Clone)]
struct TaskUiIds {
	dropzone_id: Id,
	droppable_id: widget::Id,
}
impl Default for TaskUiIds {
	fn default() -> Self {
		Self {
			dropzone_id: Id::unique(),
			droppable_id: widget::Id::unique(),
		}
	}
}

#[derive(Debug, Default)]
pub struct TaskUiIdMap {
	task_ids: HashMap<TaskId, TaskUiIds>,
}

impl TaskUiIdMap {
	pub fn get_dropzone_id(&self, task_id: TaskId) -> Option<Id> {
		self.task_ids
			.get(&task_id)
			.map(|ids| ids.dropzone_id.clone())
	}

	pub fn get_droppable_id(&self, task_id: TaskId) -> Option<widget::Id> {
		self.task_ids
			.get(&task_id)
			.map(|ids| ids.droppable_id.clone())
	}

	pub fn get_dropzone_id_mut(&mut self, task_id: TaskId) -> Id {
		self.task_ids
			.entry(task_id)
			.or_default()
			.dropzone_id
			.clone()
	}

	pub fn get_droppable_id_mut(&mut self, task_id: TaskId) -> widget::Id {
		self.task_ids
			.entry(task_id)
			.or_default()
			.droppable_id
			.clone()
	}
}
