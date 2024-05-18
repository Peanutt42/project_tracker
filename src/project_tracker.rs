use iced::{keyboard, widget::{container, text}, Application, Command, Element, Length, Subscription, Theme};
use iced_aw::{Split, SplitStyles, modal};
use crate::{
	components::{CreateNewProjectModal, CreateNewProjectModalMessage, CreateNewTaskModal, CreateNewTaskModalMessage}, pages::{ProjectListPage, ProjectPage}, project::{Project, Task}, saved_state::SavedState, styles::SplitStyle, theme_mode::{get_theme, is_system_theme_dark, system_theme_subscription, ThemeMode}
};

pub struct ProjectTrackerApp {
	pub project_list_page: ProjectListPage,
	pub project_page: Option<ProjectPage>,
	pub sidebar_position: Option<u16>,
	create_new_project_modal: CreateNewProjectModal,
	create_new_task_modal: CreateNewTaskModal,
	pub saved_state: Option<SavedState>,
	pub is_system_theme_dark: bool,
}

#[derive(Debug, Clone)]
pub enum UiMessage {
	SystemTheme { is_dark: bool },
	Loaded(SavedState),
	Save,
	Saved,
	SidebarMoved(u16),
	SelectProject(String),
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
			project_list_page: ProjectListPage::new(),
			project_page: None,
			sidebar_position: Some(250),
			create_new_project_modal: CreateNewProjectModal::new(),
			create_new_task_modal: CreateNewTaskModal::new(),
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
				UiMessage::Loaded(saved_state) => { self.saved_state = Some(saved_state); Command::none() },
				UiMessage::Save => Command::perform(saved_state.clone().save(), |_| UiMessage::Saved),
				UiMessage::Saved => Command::none(),
				UiMessage::SidebarMoved(position) => { self.sidebar_position = Some(position); Command::none() },
				UiMessage::SelectProject(project_name) => { self.project_page = Some(ProjectPage::new(project_name)); Command::none() },
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
					self.create_new_project_modal.update(message);
					Command::none()
				},
				UiMessage::CreateNewTaskModalMessage(message) => {
					self.create_new_task_modal.update(message);
					Command::none()
				},
			}
		}
		else {
			match message {
				UiMessage::SystemTheme{ is_dark } => { self.is_system_theme_dark = is_dark; Command::none() },
				UiMessage::Loaded(saved_state) => { self.saved_state = Some(saved_state); Command::none() },
				UiMessage::Save => Command::none(),
				UiMessage::Saved => Command::none(),
				UiMessage::SidebarMoved(position) => { self.sidebar_position = Some(position); Command::none() },
				UiMessage::SelectProject(project_name) => { self.project_page = Some(ProjectPage::new(project_name)); Command::none() },
				UiMessage::CreateProject(_) => self.update(CreateNewProjectModalMessage::Close.into()),
				UiMessage::CreateTask { .. } => self.update(CreateNewTaskModalMessage::Close.into()),
				UiMessage::CreateNewProjectModalMessage(message) => {
					self.create_new_project_modal.update(message);
					Command::none()
				},
				UiMessage::CreateNewTaskModalMessage(message) => {
					self.create_new_task_modal.update(message);
					Command::none()
				},
			}
		}
	}

	fn view(&self) -> Element<UiMessage> {
		let project_page = if let Some(project_page) = &self.project_page {
			project_page.view(self)
		}
		else {
			container(text("select project"))
				.width(Length::Fill)
				.height(Length::Fill)
				.center_x()
				.center_y()
				.into()
		};

		let underlay: Element<UiMessage> = Split::new(
			self.project_list_page.view(self),
			project_page,
			self.sidebar_position,
			iced_aw::split::Axis::Vertical,
			UiMessage::SidebarMoved
		)
		.style(SplitStyles::custom(SplitStyle))
		.into();

		let is_dark_mode = self.is_dark_mode();

		let selected_project_name = if let Some(project_page) = &self.project_page { project_page.project_name.clone() } else { String::new() };
		let modal_view = self.create_new_project_modal.view(is_dark_mode).or(self.create_new_task_modal.view(selected_project_name, is_dark_mode));
		let close_modal_message: UiMessage = if self.create_new_project_modal.is_opened() { CreateNewProjectModalMessage::Close.into() } else { CreateNewTaskModalMessage::Close.into() };
		modal(underlay, modal_view)
				.backdrop(close_modal_message.clone())
				.on_esc(close_modal_message)
				.into()
	}
}