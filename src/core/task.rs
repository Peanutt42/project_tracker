use iced::{Command, Application};
use serde::{Serialize, Deserialize};
use crate::{core::{ProjectId, TaskState, DatabaseMessage}, project_tracker::{ProjectTrackerApp, UiMessage}, pages::ProjectPageMessage};

pub type TaskId = usize;

pub fn generate_task_id() -> TaskId {
	rand::random()
}

#[derive(Debug, Clone)]
pub enum TaskMessage {
	Create(String),
	ChangeName(String),
	ChangeState(TaskState),
	MoveUp,
	MoveDown,
	Delete,
}

impl TaskMessage {
	pub fn to_ui_message(self, project_id: ProjectId, task_id: TaskId) -> UiMessage {
		UiMessage::TaskMessage { project_id, task_id, message: self }
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
	pub name: String,
	pub state: TaskState,
}

impl Task {
	pub fn new(name: String, state: TaskState) -> Self {
		Self {
			name,
			state,
		}
	}

	pub fn is_done(&self) -> bool {
		self.state.is_done()
	}

	pub fn is_todo(&self) -> bool {
		self.state.is_todo()
	}
}

impl ProjectTrackerApp {
	pub fn update_task(&mut self, project_id: ProjectId, task_id: TaskId, message: TaskMessage) -> Command<UiMessage> {
		if let Some(database) = &mut self.database {
			if let Some(project) = database.projects.get_mut(&project_id) {
				let command = match message {
					TaskMessage::Create(name) => {
						project.add_task(task_id, name);
						self.update(ProjectPageMessage::OpenCreateNewTask.into())
					},
					TaskMessage::ChangeName(new_name) => {
						if let Some(task) = project.tasks.get_mut(&task_id) {
							task.name = new_name;
						}
						Command::none()
					},
					TaskMessage::ChangeState(new_state) => {
						if let Some(task) = project.tasks.get_mut(&task_id) {
							task.state = new_state;
						}
						// reorder
						match new_state {
							TaskState::Todo => {
								if let Some(task_order_index) = project.tasks.get_order(&task_id) {
									// put new todo task at the top of the done tasks / at the end of all todo tasks
									for (i, task_id) in project.tasks.iter().enumerate() {
										if project.tasks.get(task_id).unwrap().is_done() {
											if i == 0 {
												project.tasks.order.insert(0, *task_id);
											}
											else {
												project.tasks.order.swap(task_order_index, i - 1);
											}
											break;
										}
									}
								}
							},
							TaskState::Done => {
								project.tasks.move_to_bottom(&task_id);
							},
						}
						Command::none()
					},
					TaskMessage::MoveUp => {
						project.tasks.move_up(&task_id);
						Command::none()
					},
					TaskMessage::MoveDown => {
						project.tasks.move_down(&task_id);
						Command::none()
					},
					TaskMessage::Delete => {
						project.tasks.remove(&task_id);
						Command::none()
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
		else {
			Command::none()
		}
	}
}
