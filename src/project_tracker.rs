use std::collections::HashMap;

use iced::{keyboard, window, font, Application, Command, Element, Event, Subscription, Theme};
use iced_aw::{Split, SplitStyles, core::icons::BOOTSTRAP_FONT_BYTES};
use crate::{
	pages::{OverviewPage, ProjectPage, ProjectPageMessage, SettingsPage, SidebarPage, SidebarPageMessage}, project::{generate_task_id, Project, ProjectId, Task, TaskId, TaskState}, saved_state::SavedState, styles::SplitStyle, theme_mode::{get_theme, is_system_theme_dark, system_theme_subscription, ThemeMode}
};

pub struct ProjectTrackerApp {
	pub sidebar_page: SidebarPage,
	pub content_page: ContentPage,
	pub selected_project_id: Option<ProjectId>,
	pub sidebar_position: Option<u16>,
	pub saved_state: Option<SavedState>,
	pub is_system_theme_dark: bool,
}

#[derive(Debug, Clone)]
pub enum UiMessage {
	Event(Event),
	EscapePressed,
	FontLoaded(Result<(), font::Error>),
	SystemTheme { is_dark: bool },
	Loaded(SavedState),
	Save,
	Saved,
	SidebarMoved(u16),
	SelectProject(ProjectId),
	CreateProject {
		project_id: ProjectId,
		project_name: String,
	},
	DeleteProject(ProjectId),
	CreateTask {
		project_id: ProjectId,
		task_name: String,
	},
	SetTaskState {
		project_id: ProjectId,
		task_id: TaskId,
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
		(
			Self {
				sidebar_page: SidebarPage::new(),
				content_page: ContentPage::Overview(OverviewPage::new()),
				selected_project_id: None,
				sidebar_position: Some(300),
				saved_state: None,
				is_system_theme_dark: is_system_theme_dark(),
			},
			Command::batch([
				Command::perform(SavedState::load(), UiMessage::Loaded),
				font::load(BOOTSTRAP_FONT_BYTES).map(UiMessage::FontLoaded),
			])
		)
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
				UiMessage::FontLoaded(_) => Command::none(),
				UiMessage::SystemTheme{ is_dark } => { self.is_system_theme_dark = is_dark; Command::none() },
				UiMessage::Loaded(saved_state) => { self.saved_state = Some(saved_state); Command::none() },
				UiMessage::Save => Command::perform(saved_state.clone().save(), |_| UiMessage::Saved),
				UiMessage::Saved => Command::none(),
				UiMessage::SidebarMoved(position) => { self.sidebar_position = Some(position); Command::none() },
				UiMessage::OpenOverview => {
					self.content_page = ContentPage::Overview(OverviewPage::new());
					self.selected_project_id = None;
					Command::none()
				},
				UiMessage::OpenSettings => {
					self.content_page = ContentPage::Settings(SettingsPage::new());
					self.selected_project_id = None;
					Command::none()
				},
				UiMessage::SelectProject(project_id) => {
					self.selected_project_id = Some(project_id);
					self.content_page = ContentPage::Project(ProjectPage::new(project_id));
					Command::none()
				},
				UiMessage::CreateProject{ project_id, project_name } => {
					saved_state.projects.insert(project_id, Project::new(project_name.clone(), HashMap::new()));
					Command::batch([
						self.update(UiMessage::Save),
						self.update(UiMessage::SelectProject(project_id)),
						self.sidebar_page.update(SidebarPageMessage::CloseCreateNewProject),
					])
				},
				UiMessage::DeleteProject(project_id) => {
					saved_state.projects.remove(&project_id);
					
					match self.selected_project_id {
						Some(selected_project_id) => {
							if selected_project_id == project_id {
								Command::batch([
									self.update(UiMessage::Save),
									self.update(UiMessage::OpenOverview),								
								])
							}
							else {
								self.update(UiMessage::Save)
							}
						},
						None => {
							self.update(UiMessage::Save)
						},
					}
				},
				UiMessage::CreateTask { project_id, task_name } => {
					if let Some(project) = saved_state.projects.get_mut(&project_id) {
						project.tasks.insert(generate_task_id(), Task::new(task_name, TaskState::Todo));
					}

					Command::batch([
						self.update(UiMessage::Save),
						self.update(ProjectPageMessage::ChangeCreateNewTaskName(String::new()).into()),
					])
				},
				UiMessage::SetTaskState { project_id, task_id, task_state } => {
					if let Some(project) = saved_state.projects.get_mut(&project_id) {
						if let Some(task) = project.tasks.get_mut(&task_id) {
							task.state = task_state;
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
					self.selected_project_id = None;
					Command::none()
				},
				UiMessage::OpenSettings => {
					self.content_page = ContentPage::Settings(SettingsPage::new());
					self.selected_project_id = None;
					Command::none()
				},
				UiMessage::SelectProject(project_id) => {
					self.selected_project_id = Some(project_id);
					self.content_page = ContentPage::Project(ProjectPage::new(project_id));
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
				UiMessage::FontLoaded(_) |
				UiMessage::CreateProject{ .. } |
				UiMessage::DeleteProject(_) |
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