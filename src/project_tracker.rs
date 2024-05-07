use iced::{Application, Command, Subscription, Element, Theme, keyboard};
use serde::{Serialize, Deserialize};
use crate::{components::CreateNewProjectModal, project::Project};
use crate::task::Task;
use crate::page::Page;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
	pub projects: Vec<Project>,
	pub dark_mode: bool,
}

impl Default for SavedState {
	fn default() -> Self {
		Self {
			projects: Vec::new(),
			dark_mode: true,
		}
	}
}

pub struct ProjectTrackerApp {
	pub page: Page,
	pub saved_state: Option<SavedState>,
}

#[derive(Debug, Clone)]
pub enum UiMessage {
	SwitchPage(Page),
	ToggleTheme,
	Loaded(SavedState),
	Save,
	Saved,
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
}


impl Application for ProjectTrackerApp {
	type Flags = ();
	type Theme = Theme;
	type Executor = iced::executor::Default;
	type Message = UiMessage;

	fn new(_flags: ()) -> (Self, Command<UiMessage>) {
		(Self {
			page: Page::StartPage{
				create_new_project_modal: CreateNewProjectModal::new(),
			},
			saved_state: None,
		},
		Command::perform(SavedState::load(), UiMessage::Loaded))
	}

	fn title(&self) -> String {
		"Project Tracker".to_string()
	}

	fn theme(&self) -> Theme {
		if let Some(saved_state) = &self.saved_state {
			if saved_state.dark_mode {
				Theme::Dark
			}
			else {
				Theme::Light
			}
		}
		else {
			Theme::Dark
		}
	}

	fn subscription(&self) -> Subscription<Self::Message> {
		keyboard::on_key_press(|key, modifiers| match key.as_ref() {
			keyboard::Key::Character("s") if modifiers.command() => {
				Some(UiMessage::Save)
			},
			_ => None,
		})
	}

	fn update(&mut self, message: UiMessage) -> Command<UiMessage> {
		match message {
			UiMessage::SwitchPage(new_page) => { self.page = new_page; Command::none() },
			UiMessage::ToggleTheme => {
				if let Some(saved_state) = &mut self.saved_state {
					saved_state.dark_mode = !saved_state.dark_mode;
				}
				Command::none()
			},
			UiMessage::Loaded(saved_state) => { self.saved_state = Some(saved_state); Command::none() },
			UiMessage::Save => {
				if let Some(saved_state) = &self.saved_state {
					Command::perform(saved_state.clone().save(), |_| UiMessage::Saved)
				}
				else {
					Command::none()
				}
			},
			UiMessage::Saved => Command::none(),
			UiMessage::OpenCreateNewProjectModal => {
				if let Page::StartPage { create_new_project_modal } = &mut self.page {
					create_new_project_modal.open()
				}
				else {
					Command::none()
				}
			},
			UiMessage::CloseCreateNewProjectModal => {
				if let Page::StartPage { create_new_project_modal } = &mut self.page {
					create_new_project_modal.close();
				}
				Command::none()
			},
			UiMessage::ChangeCreateNewProjectName(new_project_name) => {
				if let Page::StartPage { create_new_project_modal } = &mut self.page {
					create_new_project_modal.project_name = new_project_name;
				}
				Command::none()
			},
			UiMessage::CreateProject(project_name) => {
				if let Some(saved_state) = &mut self.saved_state {
					saved_state.projects.push(Project::new(project_name, Vec::new()));
				}
				Command::batch([
					self.update(UiMessage::Save),
					self.update(UiMessage::CloseCreateNewProjectModal),
				])
			},
			UiMessage::CreateTask { project_name, task } => {
				if let Some(saved_state) = &mut self.saved_state {
					for project in saved_state.projects.iter_mut() {
						if project.name == *project_name {
							project.tasks.push(task);
							break;
						}
					}
				}
				Command::batch([
					self.update(UiMessage::Save),
					self.update(UiMessage::CloseCreateNewTaskModal)
				])
			},
			UiMessage::OpenCreateNewTaskModal => {
				if let Page::ProjectPage { create_new_task_modal, .. } = &mut self.page {
					create_new_task_modal.open();
				}
				Command::none()
			},
			UiMessage::CloseCreateNewTaskModal => {
				if let Page::ProjectPage { create_new_task_modal, .. } = &mut self.page {
					create_new_task_modal.close();
				}
				Command::none()
			},
			UiMessage::ChangeCreateNewTaskName(new_task_name) => {
				if let Page::ProjectPage { create_new_task_modal, .. } = &mut self.page {
					create_new_task_modal.task_name = new_task_name;
				}
				Command::none()
			},
		}
	}

	fn view(&self) -> Element<UiMessage> {
		self.page.view(&self)
	}
}

impl SavedState {
	async fn load() -> SavedState {
		match tokio::fs::read_to_string("save.project_tracker").await {
			Ok(file_content) => {
				serde_json::from_str(&file_content).unwrap_or_default()
			},
			Err(e) => {
				eprintln!("Failed to load previous projects: {e}");
				SavedState {
					projects: Vec::new(),
					dark_mode: true,
				}
			}
		}
	}

	async fn save(self) {
		if let Err(e) = tokio::fs::write("save.project_tracker", serde_json::to_string_pretty(&self).unwrap().as_bytes()).await {
			eprintln!("Failed to save: {e}");
		}
	}
}
