use std::{path::PathBuf, time::Duration};
use iced::{clipboard, event::Status, font, keyboard, time, widget::{container, row}, window, Application, Command, Element, Event, Padding, Subscription, Theme};
use iced_aw::{core::icons::BOOTSTRAP_FONT_BYTES, split::Axis, modal, Split, SplitStyles};
use crate::{
	components::{toggle_sidebar_button, ConfirmModal, ConfirmModalMessage, ErrorMsgModal, ErrorMsgModalMessage, SwitchProjectModal, SwitchProjectModalMessage},
	core::{Database, DatabaseMessage, LoadDatabaseResult, LoadPreferencesResult, PreferenceMessage, Preferences, ProjectId},
	pages::{ContentPage, OverviewPage, ProjectPage, ProjectPageMessage, SettingsPage, SidebarPage, SidebarPageMessage},
	styles::{SplitStyle, PADDING_AMOUNT},
	theme_mode::{get_theme, is_system_theme_dark, system_theme_subscription, ThemeMode},
};

pub struct ProjectTrackerApp {
	pub selected_project_id: Option<ProjectId>,
	pub sidebar_page: SidebarPage,
	pub content_page: ContentPage,
	pub database: Option<Database>,
	pub preferences: Option<Preferences>,
	pub confirm_modal: ConfirmModal,
	pub error_msg_modal: ErrorMsgModal,
	pub switch_project_modal: SwitchProjectModal,
	pub is_system_theme_dark: bool,
}

#[derive(Clone, Debug)]
pub enum UiMessage {
	CloseWindowRequested(window::Id),
	EscapePressed,
	EnterPressed,
	ControlReleased,
	CopyToClipboard(String),
	SaveChangedFiles,
	OpenFolderLocation(PathBuf),
	FontLoaded(Result<(), font::Error>),
	SystemTheme { is_dark: bool },
	ConfirmModalMessage(ConfirmModalMessage),
	ConfirmModalConfirmed(Box<UiMessage>),
	ErrorMsgModalMessage(ErrorMsgModalMessage),
	LoadedDatabase(LoadDatabaseResult),
	LoadedPreferences(LoadPreferencesResult),
	DatabaseMessage(DatabaseMessage),
	PreferenceMessage(PreferenceMessage),
	SelectProject(Option<ProjectId>),
	SwitchToUpperProject,
	SwitchToLowerProject,
	DeleteSelectedProject,
	OpenOverview,
	OpenSettings,
	ProjectPageMessage(ProjectPageMessage),
	SidebarPageMessage(SidebarPageMessage),
	SwitchProjectModalMessage(SwitchProjectModalMessage),
}

impl Application for ProjectTrackerApp {
	type Flags = ();
	type Theme = Theme;
	type Executor = iced::executor::Default;
	type Message = UiMessage;

	fn new(_flags: ()) -> (Self, Command<UiMessage>) {
		(
			Self {
				selected_project_id: None,
				sidebar_page: SidebarPage::new(),
				content_page: ContentPage::Overview(OverviewPage::new()),
				database: None,
				preferences: None,
				confirm_modal: ConfirmModal::Closed,
				error_msg_modal: ErrorMsgModal::Closed,
				switch_project_modal: SwitchProjectModal::Closed,
				is_system_theme_dark: is_system_theme_dark(),
			},
			Command::batch([
				font::load(include_bytes!("../../assets/FiraSans-Regular.ttf")).map(UiMessage::FontLoaded),
				font::load(BOOTSTRAP_FONT_BYTES).map(UiMessage::FontLoaded),
				Command::perform(Preferences::load(), UiMessage::LoadedPreferences),
				Command::perform(Database::load(), UiMessage::LoadedDatabase),
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
				keyboard::Key::Character("h") if modifiers.command() => Some(PreferenceMessage::ToggleShowSidebar.into()),
				keyboard::Key::Character("r") if modifiers.command() => Some(ProjectPageMessage::EditProjectName.into()),
				keyboard::Key::Character(",") if modifiers.command() => Some(UiMessage::OpenSettings),
				keyboard::Key::Named(keyboard::key::Named::Escape) => Some(UiMessage::EscapePressed),
				keyboard::Key::Named(keyboard::key::Named::Enter) => Some(UiMessage::EnterPressed),
				keyboard::Key::Named(keyboard::key::Named::Delete) if modifiers.command() => Some(UiMessage::DeleteSelectedProject),
				keyboard::Key::Named(keyboard::key::Named::Tab) if modifiers.command() => Some(
					if modifiers.shift() {
						UiMessage::SwitchToUpperProject
					}
					else {
						UiMessage::SwitchToLowerProject
					}
				),
				_ => None,
			}),

			keyboard::on_key_release(|key, _modifiers| match key.as_ref() {
				keyboard::Key::Named(keyboard::key::Named::Control) => Some(UiMessage::ControlReleased),
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

			time::every(Duration::from_secs(1))
				.map(|_| UiMessage::SaveChangedFiles),

			system_theme_subscription(),
		])
	}

	fn update(&mut self, message: UiMessage) -> Command<UiMessage> {
		match message {
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
				self.update(ProjectPageMessage::StopEditingProjectName.into()),
				self.update(ProjectPageMessage::StopEditingTask.into()),
				self.update(ConfirmModalMessage::Close.into()),
				self.update(ErrorMsgModalMessage::Close.into()),
				self.update(SwitchProjectModalMessage::Close.into()),
			]),
			UiMessage::EnterPressed => {
				self.error_msg_modal = ErrorMsgModal::Closed;

				match &self.confirm_modal {
					ConfirmModal::Opened{ on_confirmed, .. } => {
						self.update(UiMessage::ConfirmModalConfirmed(Box::new(on_confirmed.clone())))
					},
					ConfirmModal::Closed => Command::none()
				}
			},
			UiMessage::ControlReleased => self.update(SwitchProjectModalMessage::Close.into()),
			UiMessage::CopyToClipboard(copied_text) => clipboard::write(copied_text),
			UiMessage::SaveChangedFiles => {
				let mut commands = Vec::new();
				if let Some(database) = &mut self.database {
					if database.has_unsaved_changes() {
						commands.push(database.update(DatabaseMessage::Save));
					}
				}
				if let Some(preferences) = &mut self.preferences {
					if preferences.has_unsaved_changes() {
						commands.push(preferences.update(PreferenceMessage::Save));
					}
				}
				Command::batch(commands)
			},
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
			UiMessage::ConfirmModalMessage(message) => {
				self.confirm_modal.update(message);
				Command::none()
			},
			UiMessage::ErrorMsgModalMessage(message) => {
				self.error_msg_modal.update(message);
				Command::none()
			},
			UiMessage::LoadedDatabase(load_database_result) => {
				match load_database_result {
					LoadDatabaseResult::Ok(database) => {
						self.database = Some(database);
						if let Some(preferences) = &self.preferences {
							if self.selected_project_id.is_none() {
								if let Some(selected_project_id) = preferences.selected_project_id {
									return self.update(UiMessage::SelectProject(Some(selected_project_id)));
								}
							}
						}
						Command::none()
					},
					LoadDatabaseResult::FailedToOpenFile(filepath) => {
						if self.database.is_none() {
							self.database = Some(Database::new());
						}
						Command::batch([
							self.update(DatabaseMessage::Save.into()),
							self.show_error_msg(format!("Could not open previous projects in \"{}\" (doesn't exist / permission issue)", filepath.display())),
						])
					},
					LoadDatabaseResult::FailedToParse(filepath) => self.show_error_msg(format!("Failed to load previous projects in \"{}\" (parsing error)", filepath.display())),
				}
			},
			UiMessage::LoadedPreferences(load_preferences_result) => {
				match load_preferences_result {
					LoadPreferencesResult::Ok(preferences) => {
						self.preferences = Some(preferences);
						self.update(PreferenceMessage::Save.into())
					},
					LoadPreferencesResult::FailedToOpenFile(filepath) => {
						self.preferences = Some(Preferences::default());
						Command::batch([
							self.update(PreferenceMessage::Save.into()),
							self.show_error_msg(format!("Could not open preferences in \"{}\" (doesn't exist / permission issue)", filepath.display())),
						])
					},
					LoadPreferencesResult::FailedToParse(filepath) => self.show_error_msg(format!("Failed to load preferences in \"{}\" (parsing error)", filepath.display())),
				}
			},
			UiMessage::DatabaseMessage(database_message) => {
				let command = match &database_message {
					DatabaseMessage::CreateProject { project_id, .. } => Command::batch([
						self.update(UiMessage::SelectProject(Some(*project_id))),
						self.sidebar_page.update(SidebarPageMessage::CloseCreateNewProject),
					]),
					DatabaseMessage::DeleteProject(project_id) => {
						match self.selected_project_id {
							Some(selected_project_id) if selected_project_id == *project_id => {
								self.update(UiMessage::OpenOverview)
							},
							_ => Command::none(),
						}
					},

					DatabaseMessage::CreateTask { .. } => {
						self.update(ProjectPageMessage::OpenCreateNewTask.into())
					},
					// TODO: close task page and go back to prev: DatabaseMessage::DeleteTask { .. } => {},
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
				self.update(UiMessage::SelectProject(None))
			},
			UiMessage::OpenSettings => {
				self.content_page = ContentPage::Settings(SettingsPage::new());
				self.update(UiMessage::SelectProject(None))
			},
			UiMessage::SelectProject(project_id) => {
				self.selected_project_id = project_id;
				if let Some(project_id) = project_id {
					self.content_page = ContentPage::Project(ProjectPage::new(project_id));
					self.sidebar_page.project_being_edited = match self.sidebar_page.project_being_edited {
						Some(project_being_edited_id) => if project_being_edited_id == project_id { Some(project_being_edited_id) } else { None },
						None => None,
					};
				}
				self.update(PreferenceMessage::SetSelectedProjectId(project_id).into())
			},
			UiMessage::SwitchToLowerProject => {
				if let Some(selected_project_id) = self.selected_project_id {
					if let Some(database) = &self.database {
						if let Some(order) = database.projects.get_order(&selected_project_id) {
							let lower_order = order + 1;
							if lower_order < database.projects.len() {
								if let Some(lower_project_id) = database.projects.get_key_at_order(lower_order) {
									let lower_project_id = *lower_project_id;
									let sidebar_snap_command = self.sidebar_page.snap_to_project(lower_order, database);
									return Command::batch([
										self.update(UiMessage::SelectProject(Some(lower_project_id))),
										sidebar_snap_command,
										self.update(SwitchProjectModalMessage::Open.into()),
									]);
								}
							}
							else {
								return Command::batch([
									self.sidebar_page.snap_to_project(order, database),
									self.update(SwitchProjectModalMessage::Open.into()),
								]);
							}
						}
					}
				}
				Command::none()
			},
			UiMessage::SwitchToUpperProject => {
				if let Some(selected_project_id) = self.selected_project_id {
					if let Some(database) = &self.database {
						if let Some(order) = database.projects.get_order(&selected_project_id) {
							if order > 0 {
								let upper_order = order - 1;
								if let Some(upper_project_id) = database.projects.get_key_at_order(upper_order) {
									let upper_project_id = *upper_project_id;
									let sidebar_snap_command = self.sidebar_page.snap_to_project(upper_order, database);
									return Command::batch([
										self.update(UiMessage::SelectProject(Some(upper_project_id))),
										sidebar_snap_command,
										self.update(SwitchProjectModalMessage::Open.into()),
									]);
								}
							}
							else {
								return Command::batch([
									self.sidebar_page.snap_to_project(order, database),
									self.update(SwitchProjectModalMessage::Open.into()),
								]);
							}
						}
					}
				}
				Command::none()
			},
			UiMessage::DeleteSelectedProject => {
				match self.selected_project_id {
					Some(selected_project_id) => self.update(ConfirmModalMessage::open("Delete this Project?", DatabaseMessage::DeleteProject(selected_project_id))),
					None => Command::none(),
				}
			}
			UiMessage::ProjectPageMessage(message) => {
				match &mut self.content_page {
					ContentPage::Project(project_page) => project_page.update(message),
					_ => Command::none()
				}
			},
			UiMessage::SidebarPageMessage(message) => self.sidebar_page.update(message),
			UiMessage::SwitchProjectModalMessage(message) => self.switch_project_modal.update(message, &self.database, self.selected_project_id),
		}
	}

	fn view(&self) -> Element<UiMessage> {
		let show_sidebar =
			if let Some(preferences) = &self.preferences {
				preferences.show_sidebar
			}
			else {
				true
			};

		let underlay: Element<UiMessage> = if show_sidebar {
			let sidebar_dividor_position =
				if let Some(preferences) = &self.preferences {
					preferences.sidebar_dividor_position
				}
				else {
					300
				};

			Split::new(
				self.sidebar_page.view(self),
				self.content_page.view(self),
				Some(sidebar_dividor_position),
				Axis::Vertical,
				|pos| PreferenceMessage::SetSidebarDividorPosition(pos).into()
			)
			.style(SplitStyles::custom(SplitStyle))
			.into()
		}
		else {
			row![
				container(toggle_sidebar_button()).padding(Padding { left: PADDING_AMOUNT, top: PADDING_AMOUNT, ..Padding::ZERO }),
				self.content_page.view(self),
			]
			.into()
		};

		let switch_project_modal_view = || {
			if let Some(preferences) = &self.preferences {
				if preferences.show_sidebar {
					None
				}
				else {
					self.switch_project_modal.view(&self.database, self.selected_project_id)
				}
			}
			else {
				None
			}
		};

		if let Some((modal_element, modal_style)) = self.error_msg_modal.view()
			.or(self.confirm_modal.view())
			.or(switch_project_modal_view())
		{
			modal(
				underlay,
				Some(modal_element)
			)
			.style(modal_style)
			.on_esc(UiMessage::EscapePressed)
			.into()
		}
		else {
			modal(
				underlay,
				None as Option<Element<UiMessage>>
			)
			.on_esc(UiMessage::EscapePressed)
			.into()
		}


	}
}

impl ProjectTrackerApp {
	fn show_error_msg(&mut self, error_msg: String) -> Command<UiMessage> {
		self.update(ErrorMsgModalMessage::open(error_msg))
	}
}