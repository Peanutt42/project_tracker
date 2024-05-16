use iced::{keyboard, Application, Command, Element, Subscription, Theme};
use crate::{
	components::{CreateNewProjectModalMessage, CreateNewTaskModalMessage},
	project::Project,
	task::Task,
	pages::{Page, StartPage, ProjectPage},
	saved_state::SavedState,
	theme_mode::{ThemeMode, is_system_theme_dark, get_theme, system_theme_subscription},
};

pub struct ProjectTrackerApp {
	pub page: Box<dyn Page>,
	pub saved_state: Option<SavedState>,
	pub is_system_theme_dark: bool,
}

#[derive(Debug, Clone)]
pub enum UiMessage {
	SystemTheme { is_dark: bool },
	GotoStartPage,
	GotoProjectPage(String),
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

impl ProjectTrackerApp {
	pub fn is_dark_mode(&self) -> bool {
		if let Some(saved_state) = &self.saved_state {
			match &saved_state.theme_mode {
				ThemeMode::Dark => true,
				ThemeMode::Light => false,
				ThemeMode::System => self.is_system_theme_dark
			}
		}
		else {
			true
		}
	}
}

impl Application for ProjectTrackerApp {
	type Flags = ();
	type Theme = Theme;
	type Executor = iced::executor::Default;
	type Message = UiMessage;

	fn new(_flags: ()) -> (Self, Command<UiMessage>) {
		(Self {
			page: Box::new(StartPage::new()),
			saved_state: None,
			is_system_theme_dark: is_system_theme_dark(),
		},
		Command::perform(SavedState::load(), UiMessage::Loaded))
	}

	fn title(&self) -> String {
		"Project Tracker".to_string()
	}

	fn theme(&self) -> Theme {
		if let Some(saved_state) = &self.saved_state {
			match saved_state.theme_mode {
				ThemeMode::System => get_theme(self.is_system_theme_dark),
				ThemeMode::Dark => get_theme(true),
				ThemeMode::Light => get_theme(false),
			}
		}
		else {
			get_theme(self.is_system_theme_dark)
		}
	}

	fn subscription(&self) -> Subscription<Self::Message> {
		Subscription::batch([
			keyboard::on_key_press(|key, modifiers| match key.as_ref() {
				keyboard::Key::Character("s") if modifiers.command() => {
					Some(UiMessage::Save)
				},
				_ => None,
			}),
			system_theme_subscription(),
		])
	}

	fn update(&mut self, message: UiMessage) -> Command<UiMessage> {
		if let Some(saved_state) = &mut self.saved_state {
			match message {
				UiMessage::SystemTheme{ is_dark } => { self.is_system_theme_dark = is_dark; Command::none() },
				UiMessage::GotoStartPage => { self.page = Box::new(StartPage::new()); Command::none() },
				UiMessage::GotoProjectPage(project_name) => { self.page = Box::new(ProjectPage::new(project_name)); Command::none() },
				UiMessage::Loaded(saved_state) => { self.saved_state = Some(saved_state); Command::none() },
				UiMessage::Save => Command::perform(saved_state.clone().save(), |_| UiMessage::Saved),
				UiMessage::Saved => Command::none(),
				UiMessage::CreateProject(project_name) => {
					saved_state.projects.push(Project::new(project_name, Vec::new()));

					Command::batch([
						self.update(UiMessage::Save),
						self.update(CreateNewProjectModalMessage::Close.into()),
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
						self.update(CreateNewTaskModalMessage::Close.into())
					])
				},
				UiMessage::CreateNewProjectModalMessage(message) => {
					self.page.update_create_new_project_modal(message);
					Command::none()
				},
				UiMessage::CreateNewTaskModalMessage(message) => {
					self.page.update_create_new_task_modal(message);
					Command::none()
				},
			}
		}
		else {
			match message {
				UiMessage::SystemTheme{ is_dark } => { self.is_system_theme_dark = is_dark; Command::none() },
				UiMessage::GotoStartPage => { self.page = Box::new(StartPage::new()); Command::none() },
				UiMessage::GotoProjectPage(project_name) => { self.page = Box::new(ProjectPage::new(project_name)); Command::none() },
				UiMessage::Loaded(saved_state) => { self.saved_state = Some(saved_state); Command::none() },
				UiMessage::Save => Command::none(),
				UiMessage::Saved => Command::none(),
				UiMessage::CreateProject(_) => self.update(CreateNewProjectModalMessage::Close.into()),
				UiMessage::CreateTask { .. } => self.update(CreateNewTaskModalMessage::Close.into()),
				UiMessage::CreateNewProjectModalMessage(message) => {
					self.page.update_create_new_project_modal(message);
					Command::none()
				},
				UiMessage::CreateNewTaskModalMessage(message) => {
					self.page.update_create_new_task_modal(message);
					Command::none()
				},
			}
		}
	}

	fn view(&self) -> Element<UiMessage> {
		self.page.view(self)
	}
}