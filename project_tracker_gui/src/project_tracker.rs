use std::path::PathBuf;
use iced::{event::Status, font, keyboard, window, Application, Command, Element, Event, Subscription, Theme};
use iced_aw::{core::icons::BOOTSTRAP_FONT_BYTES, modal, ModalStyles, Split, SplitStyles};
use crate::{
	core::{Database, DatabaseMessage, LoadDatabaseResult, LoadPreferencesResult, PreferenceMessage, Preferences, ProjectId, ProjectMessage},
	pages::{OverviewPage, ProjectPage, ProjectPageMessage, SettingsPage, SidebarPage, SidebarPageMessage},
	styles::{SplitStyle, ConfirmModalStyle},
	theme_mode::{get_theme, is_system_theme_dark, system_theme_subscription, ThemeMode},
	components::{ConfirmModal, ConfirmModalMessage},
};

pub struct ProjectTrackerApp {
	pub sidebar_page: SidebarPage,
	pub content_page: ContentPage,
	pub selected_project_id: Option<ProjectId>,
	pub database: Option<Database>,
	pub preferences: Option<Preferences>,
	pub confirm_modal: ConfirmModal,
	pub is_system_theme_dark: bool,
}

#[derive(Clone, Debug)]
pub enum UiMessage {
	Nothing,
	CloseWindowRequested(window::Id),
	EscapePressed,
	EnterPressed,
	OpenFolderLocation(PathBuf),
	FontLoaded(Result<(), font::Error>),
	SystemTheme { is_dark: bool },
	SetThemeMode(ThemeMode),
	ConfirmModalMessage(ConfirmModalMessage),
	ConfirmModalConfirmed(Box<UiMessage>),
	LoadedDatabase(LoadDatabaseResult),
	LoadedPreferences(LoadPreferencesResult),
	DatabaseMessage(DatabaseMessage),
	PreferenceMessage(PreferenceMessage),
	SelectProject(ProjectId),
	OpenOverview,
	OpenSettings,
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
				database: None,
				preferences: None,
				confirm_modal: ConfirmModal::Closed,
				is_system_theme_dark: is_system_theme_dark(),
			},
			Command::batch([
				font::load(include_bytes!("../../assets/FiraSans-Regular.ttf")).map(UiMessage::FontLoaded),
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
				},
				keyboard::Key::Named(keyboard::key::Named::Enter) => {
					Some(UiMessage::EnterPressed)
				},
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
				self.update(ConfirmModalMessage::Close.into()),
			]),
			UiMessage::OpenFolderLocation(filepath) => {
				let _ = open::that(filepath);
				Command::none()
			},
			UiMessage::FontLoaded(_) => Command::none(),
			UiMessage::SystemTheme{ is_dark } => { self.is_system_theme_dark = is_dark; Command::none() },
			UiMessage::ConfirmModalConfirmed(message) => {
				Command::batch([
					self.update(*message),
					self.update(ConfirmModalMessage::Close.into()),
				])
			},
			UiMessage::EnterPressed => {
				match &self.confirm_modal {
					ConfirmModal::Opened{ on_confirmed, .. } => {
						self.update(UiMessage::ConfirmModalConfirmed(Box::new(on_confirmed.clone())))
					},
					ConfirmModal::Closed => Command::none()
				}
			}
			UiMessage::ConfirmModalMessage(message) => {
				self.confirm_modal.update(message);
				Command::none()
			},
			UiMessage::LoadedDatabase(load_database_result) => {
				match load_database_result {
					LoadDatabaseResult::Ok(database) => { self.database = Some(database); Command::none() },
					LoadDatabaseResult::FailedToReadFile(filepath) => {
						println!("Could not find previous projects in {}", filepath.display());
						self.database = Some(Database::new());
						self.update(DatabaseMessage::Save.into())
					},
					LoadDatabaseResult::FailedToParse(filepath) => {
						eprintln!("Failed to load previous projects in {}", filepath.display());
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
				let command = match &database_message {
					DatabaseMessage::CreateProject { project_id, .. } => Command::batch([
						self.update(UiMessage::SelectProject(*project_id)),
						self.sidebar_page.update(SidebarPageMessage::CloseCreateNewProject),
					]),
					DatabaseMessage::DeleteProject(project_id) => {
						match self.selected_project_id {
							Some(selected_project_id) if selected_project_id == *project_id => {
								self.update(UiMessage::OpenOverview)
							},
							Some(_) | None => {
								Command::none()
							},
						}
					},

					DatabaseMessage::ProjectMessage { message, .. } => {
						if let ProjectMessage::CreateTask{ .. } = message {
							self.update(ProjectPageMessage::OpenCreateNewTask.into())
						}
						else {
							Command::none()
						}
					},
					_ => Command::none(),
				};

				if let Some(database) = &mut self.database {
					Command::batch([
						command,
						database.update(database_message),
					])
				}
				else {
					command
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
		modal(
			Split::new(
				self.sidebar_page.view(self),
				self.content_page.view(self),
				Some(self.sidebar_page.dividor_position),
				iced_aw::split::Axis::Vertical,
				|pos| SidebarPageMessage::SidebarMoved(pos).into()
			)
			.style(SplitStyles::custom(SplitStyle)),

			self.confirm_modal.view()
		)
		.style(ModalStyles::custom(ConfirmModalStyle))
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
