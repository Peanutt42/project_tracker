use iced::{keyboard, window, Application, Command, Element, Event, Subscription, Theme};
use iced_aw::{Split, SplitStyles};
use crate::{
	pages::{OverviewPage, ProjectPage, ProjectPageMessage, SettingsPage, SidebarPage, SidebarPageMessage}, project::{Project, Task, TaskState}, saved_state::SavedState, styles::SplitStyle, theme_mode::{get_theme, is_system_theme_dark, system_theme_subscription, ThemeMode}
};

pub struct ProjectTrackerApp {
	pub sidebar_page: SidebarPage,
	pub content_page: ContentPage,
	pub selected_page_name: String,
	pub sidebar_position: Option<u16>,
	pub saved_state: Option<SavedState>,
	pub is_system_theme_dark: bool,
}

#[derive(Debug, Clone)]
pub enum UiMessage {
	Event(Event),
	EscapePressed,
	SystemTheme { is_dark: bool },
	Loaded(SavedState),
	Save,
	Saved,
	SidebarMoved(u16),
	SelectProject(String),
	CreateProject(String),
	CreateTask {
		project_name: String,
		task_name: String,
	},
	SetTaskState {
		project_name: String,
		task_name: String,
		task_state: TaskState,
	},
	OpenOverview,
	OpenSettings,
	SetThemeMode(ThemeMode),
	ProjectPageMessage(ProjectPageMessage),
	SidebarPageMessage(SidebarPageMessage),
}

impl Application for ProjectTrackerApp {
	type Flags = ();
	type Theme = Theme;
	type Executor = iced::executor::Default;
	type Message = UiMessage;

	fn new(_flags: ()) -> (Self, Command<UiMessage>) {
		(Self {
			sidebar_page: SidebarPage::new(),
			content_page: ContentPage::Overview(OverviewPage::new()),
			selected_page_name: String::new(),
			sidebar_position: Some(300),
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
				keyboard::Key::Character("n") if modifiers.command() => {
					Some(
						if modifiers.shift() {
							SidebarPageMessage::OpenCreateNewProject.into()
						}
						else {
							ProjectPageMessage::OpenCreateNewTask.into()
						}
					)
				},
				keyboard::Key::Named(keyboard::key::Named::Escape) => {
					Some(UiMessage::EscapePressed)
				}
				_ => None,
			}),
			iced::event::listen().map(UiMessage::Event),
			system_theme_subscription(),
		])
	}

	fn update(&mut self, message: UiMessage) -> Command<UiMessage> {
		if let Some(saved_state) = &mut self.saved_state {
			match message {
				UiMessage::Event(event) => {
					if let Event::Window(id, window::Event::CloseRequested) = event {
						Command::batch([
							self.update(UiMessage::Save),
							window::close(id),
						])
					}
					else {
						Command::none()
					}
				},
				UiMessage::EscapePressed => Command::batch([
					self.update(SidebarPageMessage::CloseCreateNewProject.into()),
					self.update(ProjectPageMessage::CloseCreateNewTask.into())
				]),
				UiMessage::SystemTheme{ is_dark } => { self.is_system_theme_dark = is_dark; Command::none() },
				UiMessage::Loaded(saved_state) => { self.saved_state = Some(saved_state); Command::none() },
				UiMessage::Save => Command::perform(saved_state.clone().save(), |_| UiMessage::Saved),
				UiMessage::Saved => Command::none(),
				UiMessage::SidebarMoved(position) => { self.sidebar_position = Some(position); Command::none() },
				UiMessage::OpenOverview => {
					self.content_page = ContentPage::Overview(OverviewPage::new());
					self.selected_page_name.clear();
					Command::none()
				},
				UiMessage::OpenSettings => {
					self.content_page = ContentPage::Settings(SettingsPage::new());
					self.selected_page_name.clear();
					Command::none()
				},
				UiMessage::SelectProject(project_name) => {
					self.selected_page_name = project_name.clone();
					self.content_page = ContentPage::Project(ProjectPage::new(project_name));
					Command::none()
				},
				UiMessage::CreateProject(project_name) => {
					saved_state.projects.push(Project::new(project_name.clone(), Vec::new()));
					Command::batch([
						self.update(UiMessage::Save),
						self.update(UiMessage::SelectProject(project_name)),
						self.sidebar_page.update(SidebarPageMessage::CloseCreateNewProject),
					])
				},
				UiMessage::CreateTask { project_name, task_name } => {
					for project in saved_state.projects.iter_mut() {
						if project.name == project_name {
							project.tasks.push(Task::new(task_name, TaskState::Todo));
							break;
						}
					}

					Command::batch([
						self.update(UiMessage::Save),
						self.update(ProjectPageMessage::ChangeCreateNewTaskName(String::new()).into()),
					])
				},
				UiMessage::SetTaskState { project_name, task_name, task_state } => {
					for project in saved_state.projects.iter_mut() {
						if project.name == project_name {
							for task in project.tasks.iter_mut() {
								if task.name == task_name {
									task.state = task_state;
									break;
								}
							}
							break;
						}
					}

					self.update(UiMessage::Save)
				},
				UiMessage::SetThemeMode(theme_mode) => { saved_state.theme_mode = theme_mode; self.update(UiMessage::Save) }
				UiMessage::ProjectPageMessage(message) => {
					if let ContentPage::Project(project_page) = &mut self.content_page {
						project_page.update(message.clone())
					}
					else {
						Command::none()
					}
				},
				UiMessage::SidebarPageMessage(message) => self.sidebar_page.update(message.clone()),
			}
		}
		else {
			match message {
				UiMessage::EscapePressed => Command::batch([
					self.update(SidebarPageMessage::CloseCreateNewProject.into()),
					self.update(ProjectPageMessage::CloseCreateNewTask.into())
				]),
				UiMessage::SystemTheme{ is_dark } => { self.is_system_theme_dark = is_dark; Command::none() },
				UiMessage::Loaded(saved_state) => { self.saved_state = Some(saved_state); Command::none() },
				UiMessage::SidebarMoved(position) => { self.sidebar_position = Some(position); Command::none() },
				UiMessage::OpenOverview => {
					self.content_page = ContentPage::Overview(OverviewPage::new());
					self.selected_page_name.clear();
					Command::none()
				},
				UiMessage::OpenSettings => {
					self.content_page = ContentPage::Settings(SettingsPage::new());
					self.selected_page_name.clear();
					Command::none()
				},
				UiMessage::SelectProject(project_name) => {
					self.selected_page_name = project_name.clone();
					self.content_page = ContentPage::Project(ProjectPage::new(project_name));
					Command::none()
				},
				UiMessage::ProjectPageMessage(message) => {
					if let ContentPage::Project(project_page) = &mut self.content_page {
						project_page.update(message)
					}
					else {
						Command::none()
					}
				},
				UiMessage::SidebarPageMessage(message) => self.sidebar_page.update(message),

				UiMessage::Event(_) |
				UiMessage::CreateProject(_) |
				UiMessage::CreateTask { .. } |
				UiMessage::SetTaskState { .. } |
				UiMessage::SetThemeMode(_) |
				UiMessage::Save |
				UiMessage::Saved => Command::none(),
			}
		}
	}

	fn view(&self) -> Element<UiMessage> {
		Split::new(
			self.sidebar_page.view(self),
			self.content_page.view(self),
			self.sidebar_position,
			iced_aw::split::Axis::Vertical,
			UiMessage::SidebarMoved
		)
		.style(SplitStyles::custom(SplitStyle))
		.into()
	}
}

pub enum ContentPage {
	Project(ProjectPage),
	Overview(OverviewPage),
	Settings(SettingsPage),
}

impl ContentPage {
	pub fn is_overview_page(&self) -> bool {
		matches!(self, ContentPage::Overview(_))
	}
}

impl ContentPage {
	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		match self {
			ContentPage::Project(project_page) => project_page.view(app),
			ContentPage::Overview(overview_page) => overview_page.view(app),
			ContentPage::Settings(settings_page) => settings_page.view(app),
		}
	}
}