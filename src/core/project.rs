use iced::{Command, Application};
use serde::{Serialize, Deserialize};
use crate::{core::{OrderedHashMap, DatabaseMessage, Task, TaskId, TaskState}, project_tracker::{ProjectTrackerApp, UiMessage}, pages::SidebarPageMessage};

pub type ProjectId = usize;

pub fn generate_project_id() -> ProjectId {
	rand::random()
}

#[derive(Debug, Clone)]
pub enum ProjectMessage {
	Create(String),
	ChangeName(String),
	MoveUp,
	MoveDown,
	Delete
}

impl ProjectMessage {
	pub fn to_ui_message(self, project_id: ProjectId) -> UiMessage {
		UiMessage::ProjectMessage { project_id, message: self }
	}
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
}

impl ProjectTrackerApp {
	pub fn update_project(&mut self, project_id: ProjectId, message: ProjectMessage) -> Command<UiMessage> {
		if let Some(database) = &mut self.database {
			let command = match message {
				ProjectMessage::Create(name) => {
					database.projects.insert(project_id, Project::new(name));
					Command::batch([
						self.update(UiMessage::SelectProject(project_id)),
						self.sidebar_page.update(SidebarPageMessage::CloseCreateNewProject),
					])
				},
				ProjectMessage::ChangeName(new_name) => {
					if let Some(project) = database.projects.get_mut(&project_id) {
						project.name = new_name;
					}
					Command::none()
				},
				ProjectMessage::MoveUp => {
					database.projects.move_up(&project_id);
					Command::none()
				},
				ProjectMessage::MoveDown => {
					database.projects.move_down(&project_id);
					Command::none()
				},
				ProjectMessage::Delete => {
					database.projects.remove(&project_id);

					match self.selected_project_id {
						Some(selected_project_id) if selected_project_id == project_id => {
							self.update(UiMessage::OpenOverview)
						},
						Some(_) | None => {
							Command::none()
						},
					}
				},
			};

			Command::batch([
				command,
				self.update(DatabaseMessage::Save.into()),
			])
		}
		else {
			Command::none()
		}
	}
}
