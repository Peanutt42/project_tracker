use std::io::{Read, Write};

use iced::{Element, Sandbox, Theme};
use crate::{components::CreateNewProjectModal, project::Project};
use crate::task::Task;
use crate::page::Page;

pub struct ProjectTrackerApp {
	pub page: Page,
	pub projects: Vec<Project>,
	pub dark_mode: bool,
}

#[derive(Debug, Clone)]
pub enum UiMessage {
	SwitchPage(Page),
	ToggleTheme,
	OpenCreateNewProjectModal,
	CloseCreateNewProjectModal,
	ChangeCreateNewProjectName(String),
	CreateProject(String),
	CreateTask {
		project_name: String,
		task: Task,
	},
	OpenCreateNewTaskModal,
	CloseCreateNewTaskModal,
	ChangeCreateNewTaskName(String),
	Save,
}


impl Sandbox for ProjectTrackerApp {
	type Message = UiMessage;

	fn new() -> Self {
		let projects = match std::fs::File::open("save.project_tracker") {
			Ok(mut file) => {
				let mut file_content = String::new();
				if file.read_to_string(&mut file_content).is_ok() {
					serde_json::from_str(&file_content).unwrap_or_default()
				}
				else {
					Vec::new()
				}
			},
			Err(e) => {
				eprintln!("Failed to load previous projects: {e}");
				Vec::new()
			}
		};

		Self {
			page: Page::StartPage{
				create_new_project_modal: CreateNewProjectModal::new(),
			},
			projects,
			dark_mode: true,
		}
	}

	fn title(&self) -> String {
		"Project Tracker".to_string()
	}

	fn theme(&self) -> Theme {
		if self.dark_mode {
			Theme::Dark
		}
		else {
			Theme::Light
		}
	}

	fn update(&mut self, message: UiMessage) {
		match message {
			UiMessage::SwitchPage(new_page) => self.page = new_page,
			UiMessage::ToggleTheme => self.dark_mode = !self.dark_mode,
			UiMessage::Save => {
				match std::fs::File::create("save.project_tracker") {
					Ok(mut file) => {
						if let Err(e) = file.write_all(serde_json::to_string_pretty(&self.projects).unwrap().as_bytes()) {
							eprintln!("Failed to write to save file: {e}");
						}
					},
					Err(e) => eprintln!("Failed to save: {e}"),
				}
			},
			UiMessage::OpenCreateNewProjectModal => {
				if let Page::StartPage { create_new_project_modal } = &mut self.page {
					create_new_project_modal.open();
				}
			},
			UiMessage::CloseCreateNewProjectModal => {
				if let Page::StartPage { create_new_project_modal } = &mut self.page {
					create_new_project_modal.close();
				}
			},
			UiMessage::ChangeCreateNewProjectName(new_project_name) => {
				if let Page::StartPage { create_new_project_modal } = &mut self.page {
					create_new_project_modal.project_name = new_project_name;
				}
			},
			UiMessage::CreateProject(project_name) => {
				self.projects.push(Project::new(project_name, Vec::new()));
				self.update(UiMessage::CloseCreateNewProjectModal);
			},
			UiMessage::CreateTask { project_name, task } => {
				for project in self.projects.iter_mut() {
					if project.name == *project_name {
						project.tasks.push(task);
						break;
					}
				}
				self.update(UiMessage::CloseCreateNewTaskModal);
			},
			UiMessage::OpenCreateNewTaskModal => {
				if let Page::ProjectPage { create_new_task_modal, .. } = &mut self.page {
					create_new_task_modal.open();
				}
			},
			UiMessage::CloseCreateNewTaskModal => {
				if let Page::ProjectPage { create_new_task_modal, .. } = &mut self.page {
					create_new_task_modal.close();
				}
			},
			UiMessage::ChangeCreateNewTaskName(new_task_name) => {
				if let Page::ProjectPage { create_new_task_modal, .. } = &mut self.page {
					create_new_task_modal.task_name = new_task_name;
				}
			},
		}
	}

	fn view(&self) -> Element<UiMessage> {
		self.page.view(&self.projects)
	}
}

impl Drop for ProjectTrackerApp {
	fn drop(&mut self) {
		self.update(UiMessage::Save);
	}
}
