use iced::{Application, Command, Subscription, Element, Theme, keyboard};
use crate::{components::{CreateNewProjectModal, CreateNewProjectModalMessage, CreateNewTaskModalMessage}, project::Project};
use crate::task::Task;
use crate::page::Page;
use crate::saved_state::SavedState;

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
	CreateProject(String),
	CreateTask {
		project_name: String,
		task: Task,
	},
	CreateNewProjectModalMessage(CreateNewProjectModalMessage),
	CreateNewTaskModalMessage(CreateNewTaskModalMessage),
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
		if let Some(saved_state) = &mut self.saved_state {
			match message {
				UiMessage::SwitchPage(new_page) => { self.page = new_page; Command::none() },
				UiMessage::ToggleTheme => { saved_state.dark_mode = !saved_state.dark_mode; Command::none() },
				UiMessage::Loaded(saved_state) => { self.saved_state = Some(saved_state); Command::none() },
				UiMessage::Save => Command::perform(saved_state.clone().save(), |_| UiMessage::Saved),
				UiMessage::Saved => Command::none(),
				UiMessage::CreateProject(project_name) => {
					saved_state.projects.push(Project::new(project_name, Vec::new()));

					Command::batch([
						self.update(UiMessage::Save),
						self.update(UiMessage::CreateNewProjectModalMessage(CreateNewProjectModalMessage::Close)),
					])
				},
				UiMessage::CreateTask { project_name, task } => {
					for project in saved_state.projects.iter_mut() {
						if project.name == *project_name {
							project.tasks.push(task);
							break;
						}
					}
					Command::batch([
						self.update(UiMessage::Save),
						self.update(UiMessage::CreateNewTaskModalMessage(CreateNewTaskModalMessage::Close))
					])
				},
				UiMessage::CreateNewProjectModalMessage(message) => {
					self.page.update_create_new_project_modal_message(message);
					Command::none()
				},
				UiMessage::CreateNewTaskModalMessage(message) => {
					self.page.update_create_new_task_modal_message(message);
					Command::none()
				},
			}
		}
		else {
			match message {
				UiMessage::SwitchPage(new_page) => { self.page = new_page; Command::none() },
				UiMessage::ToggleTheme => Command::none(),
				UiMessage::Loaded(saved_state) => { self.saved_state = Some(saved_state); Command::none() },
				UiMessage::Save => Command::none(),
				UiMessage::Saved => Command::none(),
				UiMessage::CreateProject(_) => self.update(UiMessage::CreateNewProjectModalMessage(CreateNewProjectModalMessage::Close)),
				UiMessage::CreateTask { .. } => self.update(UiMessage::CreateNewTaskModalMessage(CreateNewTaskModalMessage::Close)),
				UiMessage::CreateNewProjectModalMessage(message) => {
					self.page.update_create_new_project_modal_message(message);
					Command::none()
				},
				UiMessage::CreateNewTaskModalMessage(message) => {
					self.page.update_create_new_task_modal_message(message);
					Command::none()
				},
			}
		}
	}

	fn view(&self) -> Element<UiMessage> {
		self.page.view(&self)
	}
}
