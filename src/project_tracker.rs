use iced::{keyboard, Application, Command, Element, Subscription, Theme};
use iced_aw::{Split, SplitStyles, modal};
use crate::{
	components::{CreateNewProjectModal, CreateNewProjectModalMessage, CreateNewTaskModal, CreateNewTaskModalMessage}, pages::{SidebarPage, ProjectPage, OverviewPage}, project::{Project, Task}, saved_state::SavedState, styles::SplitStyle, theme_mode::{get_theme, is_system_theme_dark, system_theme_subscription, ThemeMode}
};

pub struct ProjectTrackerApp {
	pub sidebar_page: SidebarPage,
	pub content_page: ContentPage,
	pub selected_page_name: String,
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
	OpenOverview,
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
			sidebar_page: SidebarPage::new(),
			content_page: ContentPage::OverviewPage(OverviewPage::new()),
			selected_page_name: String::new(),
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
				UiMessage::OpenOverview => {
					self.content_page = ContentPage::OverviewPage(OverviewPage::new());
					self.selected_page_name = String::new();
					Command::none()
				},
				UiMessage::SelectProject(project_name) => {
					self.selected_page_name = project_name.clone();
					self.content_page = ContentPage::ProjectPage(ProjectPage::new(project_name));
					Command::none()
				},
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
				UiMessage::OpenOverview => {
					self.content_page = ContentPage::OverviewPage(OverviewPage::new());
					self.selected_page_name = String::new();
					Command::none()
				},
				UiMessage::SelectProject(project_name) => {
					self.selected_page_name = project_name.clone();
					self.content_page = ContentPage::ProjectPage(ProjectPage::new(project_name));
					Command::none()
				},
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
		let underlay: Element<UiMessage> = Split::new(
			self.sidebar_page.view(self),
			self.content_page.view(self),
			self.sidebar_position,
			iced_aw::split::Axis::Vertical,
			UiMessage::SidebarMoved
		)
		.style(SplitStyles::custom(SplitStyle))
		.into();

		let is_dark_mode = self.is_dark_mode();

		let modal_view = self.create_new_project_modal.view(is_dark_mode).or(self.create_new_task_modal.view(self.selected_page_name.clone(), is_dark_mode));
		let close_modal_message: UiMessage = if self.create_new_project_modal.is_opened() { CreateNewProjectModalMessage::Close.into() } else { CreateNewTaskModalMessage::Close.into() };
		modal(underlay, modal_view)
				.backdrop(close_modal_message.clone())
				.on_esc(close_modal_message)
				.into()
	}
}

pub enum ContentPage {
	ProjectPage(ProjectPage),
	OverviewPage(OverviewPage),
}

impl ContentPage {
	pub fn is_overview_page(&self) -> bool {
		matches!(self, ContentPage::OverviewPage(_))
	}
}

impl ContentPage {
	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		match self {
			ContentPage::ProjectPage(project_page) => project_page.view(app),
			ContentPage::OverviewPage(todo_overview_page) => todo_overview_page.view(app),
		}
	}
}