use iced::{event::Status, font, keyboard, window, Application, Command, Element, Event, Subscription, Theme};
use iced_aw::{Split, SplitStyles, core::icons::BOOTSTRAP_FONT_BYTES};
use crate::{
	core::{Database, LoadDatabaseResult, DatabaseMessage, LoadPreferencesResult, Preferences, PreferenceMessage, Project, ProjectId, TaskId, TaskState}, pages::{OverviewPage, ProjectPage, ProjectPageMessage, SettingsPage, SidebarPage, SidebarPageMessage}, styles::SplitStyle, theme_mode::{get_theme, is_system_theme_dark, system_theme_subscription, ThemeMode}
};

pub struct ProjectTrackerApp {
	pub sidebar_page: SidebarPage,
	pub content_page: ContentPage,
	pub selected_project_id: Option<ProjectId>,
	pub sidebar_position: Option<u16>,
	pub database: Option<Database>,
	pub preferences: Option<Preferences>,
	pub is_system_theme_dark: bool,
}

#[derive(Debug, Clone)]
pub enum UiMessage {
	Nothing,
	CloseWindowRequested(window::Id),
	EscapePressed,
	FontLoaded(Result<(), font::Error>),
	SystemTheme { is_dark: bool },
	LoadedPreferences(LoadPreferencesResult),
	LoadedDatabase(LoadDatabaseResult),
	DatabaseMessage(DatabaseMessage),
	PreferenceMessage(PreferenceMessage),
	SidebarMoved(u16),
	SelectProject(ProjectId),
	CreateProject {
		project_id: ProjectId,
		project_name: String,
	},
	ChangeProjectName {
		project_id: ProjectId,
		new_project_name: String,
	},
	MoveProjectUp(ProjectId),
	MoveProjectDown(ProjectId),
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
	ChangeTaskName {
		project_id: ProjectId,
		task_id: TaskId,
		new_task_name: String,
	},
	MoveTaskUp {
		project_id: ProjectId,
		task_id: TaskId,
	},
	MoveTaskDown {
		project_id: ProjectId,
		task_id: TaskId,
	},
	DeleteTask {
		project_id: ProjectId,
		task_id: TaskId,
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
				database: None,
				preferences: None,
				is_system_theme_dark: is_system_theme_dark(),
			},
			Command::batch([
				font::load(include_bytes!("../assets/FiraSans-Regular.ttf")).map(UiMessage::FontLoaded),
				font::load(BOOTSTRAP_FONT_BYTES).map(UiMessage::FontLoaded),
				Command::perform(Database::load(), UiMessage::LoadedDatabase),
				Command::perform(Preferences::load(), UiMessage::LoadedPreferences),
			])
		)
	}

	fn title(&self) -> String {
		"Project Tracker".to_string()
	}

	fn theme(&self) -> Theme {
		if let Some(preferences) = &self.preferences {
			match preferences.theme_mode {
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

			iced::event::listen_with(move |event, status| {
				match status {
					Status::Ignored => {
						if let Event::Window(id, window::Event::CloseRequested) = event {
							Some(id)
						}
						else {
							None
						}
					},
					Status::Captured => None,
				}
			})
			.map(UiMessage::CloseWindowRequested),

			system_theme_subscription(),
		])
	}

	fn update(&mut self, message: UiMessage) -> Command<UiMessage> {
		match message {
			UiMessage::Nothing => Command::none(),
			UiMessage::CloseWindowRequested(id) => {
				Command::batch([
					self.update(DatabaseMessage::Save.into()),
					self.update(PreferenceMessage::Save.into()),
					window::close(id),
				])
			},
			UiMessage::EscapePressed => Command::batch([
				self.update(SidebarPageMessage::CloseCreateNewProject.into()),
				self.update(SidebarPageMessage::StopEditingProject.into()),
				self.update(ProjectPageMessage::CloseCreateNewTask.into()),
				self.update(ProjectPageMessage::StopEditing.into()),
			]),
			UiMessage::FontLoaded(_) => Command::none(),
			UiMessage::SystemTheme{ is_dark } => { self.is_system_theme_dark = is_dark; Command::none() },
			UiMessage::LoadedDatabase(load_database_result) => {
				match load_database_result {
					LoadDatabaseResult::Ok(database) => {
						self.database = Some(database);
						Command::none()
					},
					LoadDatabaseResult::FailedToReadFile(filepath) => {
						println!("Could not find previous projects in {}", filepath.display());
						self.database = Some(Database::new());
						self.update(DatabaseMessage::Save.into())
					},
					LoadDatabaseResult::FailedToParse(filepath) => {
						println!("Failed to load previous projects in {}", filepath.display());
						Command::none()
					}
				}
			},
			UiMessage::LoadedPreferences(load_preferences_result) => {
				match load_preferences_result {
					LoadPreferencesResult::Ok(preferences) => {
						self.preferences = Some(preferences);
						self.update(PreferenceMessage::Save.into())
					},
					LoadPreferencesResult::FailedToReadFile(filepath) => {
						println!("Could not find preferences in {}", filepath.display());
						self.preferences = Some(Preferences::default());
						self.update(PreferenceMessage::Save.into())
					},
					LoadPreferencesResult::FailedToParse(filepath) => {
						println!("Failed to load preferences in {}", filepath.display());
						Command::none()
					}
				}
			},
			UiMessage::DatabaseMessage(database_message) => {
				if let Some(database) = &mut self.database {
					database.update(database_message)
				}
				else {
					Command::none()
				}
			},
			UiMessage::PreferenceMessage(preference_message) => {
				if let Some(preferences) = &mut self.preferences {
					preferences.update(preference_message)
				}
				else {
					Command::none()
				}
			},
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
				self.sidebar_page.project_being_edited = match self.sidebar_page.project_being_edited {
					Some(project_being_edited_id) => if project_being_edited_id == project_id { Some(project_being_edited_id) } else { None },
					None => None,
				};
				Command::none()
			},
			UiMessage::CreateProject{ project_id, project_name } => {
				if let Some(database) = &mut self.database {
					database.projects.insert(project_id, Project::new(project_name));

					Command::batch([
						self.update(DatabaseMessage::Save.into()),
						self.update(UiMessage::SelectProject(project_id)),
						self.sidebar_page.update(SidebarPageMessage::CloseCreateNewProject),
					])
				}
				else {
					Command::none()
				}
			},
			UiMessage::ChangeProjectName { project_id, new_project_name } => {
				if let Some(database) = &mut self.database {
					if let Some(project) = database.projects.get_mut(&project_id) {
						project.name = new_project_name;
					}
					self.update(DatabaseMessage::Save.into())
				}
				else {
					Command::none()
				}
			}
			UiMessage::MoveProjectUp(project_id) => {
				if let Some(database) = &mut self.database {
					database.projects.move_up(&project_id);
					self.update(DatabaseMessage::Save.into())
				}
				else {
					Command::none()
				}
			},
			UiMessage::MoveProjectDown(project_id) => {
				if let Some(database) = &mut self.database {
					database.projects.move_down(&project_id);
					self.update(DatabaseMessage::Save.into())
				}
				else {
					Command::none()
				}
			},
			UiMessage::DeleteProject(project_id) => {
				if let Some(database) = &mut self.database {
					database.projects.remove(&project_id);

					match self.selected_project_id {
						Some(selected_project_id) => {
							if selected_project_id == project_id {
								Command::batch([
									self.update(DatabaseMessage::Save.into()),
									self.update(UiMessage::OpenOverview),
								])
							}
							else {
								self.update(DatabaseMessage::Save.into())
							}
						},
						None => {
							self.update(DatabaseMessage::Save.into())
						},
					}
				}
				else {
					Command::none()
				}
			},
			UiMessage::CreateTask { project_id, task_name } => {
				if let Some(database) = &mut self.database {
					if let Some(project) = database.projects.get_mut(&project_id) {
						project.add_task(task_name);
					}

					Command::batch([
						self.update(DatabaseMessage::Save.into()),
						self.update(ProjectPageMessage::OpenCreateNewTask.into()),
					])
				}
				else {
					Command::none()
				}
			},
			UiMessage::SetTaskState { project_id, task_id, task_state } => {
				if let Some(database) = &mut self.database {
					if let Some(project) = database.projects.get_mut(&project_id) {
						if let Some(task) = project.tasks.get_mut(&task_id) {
							task.state = task_state;
						}
						// reorder
						match task_state {
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
					}

					self.update(DatabaseMessage::Save.into())
				}
				else {
					Command::none()
				}
			},
			UiMessage::ChangeTaskName { project_id, task_id, new_task_name } => {
				if let Some(database) = &mut self.database {
					if let Some(project) = database.projects.get_mut(&project_id) {
						if let Some(task) = project.tasks.get_mut(&task_id) {
							task.name = new_task_name;
							return self.update(DatabaseMessage::Save.into());
						}
					}
				}

				Command::none()
			},
			UiMessage::MoveTaskUp { project_id, task_id } => {
				if let Some(database) = &mut self.database {
					if let Some(project) = database.projects.get_mut(&project_id) {
						project.tasks.move_up(&task_id);
						return self.update(DatabaseMessage::Save.into());
					}
				}
				Command::none()
			},
			UiMessage::MoveTaskDown { project_id, task_id } => {
				if let Some(database) = &mut self.database {
					if let Some(project) = database.projects.get_mut(&project_id) {
						project.tasks.move_down(&task_id);
						return self.update(DatabaseMessage::Save.into());
					}
				}
				Command::none()
			},
			UiMessage::DeleteTask { project_id, task_id } => {
				if let Some(database) = &mut self.database {
					if let Some(project) = database.projects.get_mut(&project_id) {
						project.tasks.remove(&task_id);
						return self.update(DatabaseMessage::Save.into());
					}
				}

				Command::none()
			},
			UiMessage::SetThemeMode(theme_mode) => {
				if let Some(preferences) = &mut self.preferences {
					preferences.theme_mode = theme_mode;
					self.update(PreferenceMessage::Save.into())
				}
				else {
					Command::none()
				}
			},
			UiMessage::ProjectPageMessage(message) => {
				if let ContentPage::Project(project_page) = &mut self.content_page {
					project_page.update(message.clone())
				}
				else {
					Command::none()
				}
			},
			UiMessage::SidebarPageMessage(message) => self.sidebar_page.update(message),
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

	pub fn is_settings_page(&self) -> bool {
		matches!(self, ContentPage::Settings(_))
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
