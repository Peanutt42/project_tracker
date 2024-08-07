use std::{path::PathBuf, time::Duration};
use iced::{clipboard, event::Status, font, keyboard, mouse, time, widget::{container, row}, window, Application, Command, Element, Event, Padding, Subscription, Theme};
use iced_aw::{core::icons::BOOTSTRAP_FONT_BYTES, split::Axis, modal, Split, SplitStyles};
use crate::{
	components::{invisible_toggle_sidebar_button, toggle_sidebar_button, ConfirmModal, ConfirmModalMessage, ErrorMsgModal, ErrorMsgModalMessage, ManageTaskTagsModal, ManageTaskTagsModalMessage, SettingsModal, SettingsModalMessage, SwitchProjectModal, SwitchProjectModalMessage},
	core::{Database, DatabaseMessage, LoadDatabaseResult, LoadPreferencesResult, PreferenceMessage, Preferences, ProjectId, SerializedContentPage},
	pages::{ContentPage, OverviewPage, ProjectPage, ProjectPageMessage, SidebarPage, SidebarPageMessage},
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
	pub settings_modal: SettingsModal,
	pub manage_tags_modal: ManageTaskTagsModal,
	pub is_system_theme_dark: bool,
}

#[derive(Clone, Debug)]
pub enum UiMessage {
	CloseWindowRequested(window::Id),
	EscapePressed,
	EnterPressed,
	ControlReleased,
	LeftClickReleased,
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
	SwitchToUpperProject, // switches to upper project when using shortcuts
	SwitchToLowerProject, // switches to lower project when using shortcuts
	SwitchToProject{ order: usize }, // switches to project when using shortcuts
	DeleteSelectedProject,
	OpenOverview,
	ProjectPageMessage(ProjectPageMessage),
	SidebarPageMessage(SidebarPageMessage),
	SwitchProjectModalMessage(SwitchProjectModalMessage),
	SettingsModalMessage(SettingsModalMessage),
	ManageTaskTagsModalMessage(ManageTaskTagsModalMessage),
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
				settings_modal: SettingsModal::Closed,
				manage_tags_modal: ManageTaskTagsModal::Closed,
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
			match preferences.theme_mode() {
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
				keyboard::Key::Character("b") if modifiers.command() => Some(PreferenceMessage::ToggleShowSidebar.into()),
				keyboard::Key::Character("h") if modifiers.command() => Some(UiMessage::OpenOverview),
				keyboard::Key::Character("r") if modifiers.command() => Some(ProjectPageMessage::EditProjectName.into()),
				keyboard::Key::Character(",") if modifiers.command() => Some(SettingsModalMessage::Open.into()),
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
				match event {
					Event::Window(id, window::Event::CloseRequested) if matches!(status, Status::Ignored) => Some(UiMessage::CloseWindowRequested(id)),
					Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => Some(UiMessage::LeftClickReleased),
					_ => None,
				}
			}),

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
				self.update(ProjectPageMessage::HideColorPicker.into()),
				self.update(ConfirmModalMessage::Close.into()),
				self.update(ErrorMsgModalMessage::Close.into()),
				self.update(SwitchProjectModalMessage::Close.into()),
				self.update(SettingsModalMessage::Close.into()),
				self.update(ManageTaskTagsModalMessage::Close.into()),
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
			UiMessage::LeftClickReleased => {
				let select_project_command = self.sidebar_page
					.should_select_project()
        			.map(|project_id| self.update(UiMessage::SelectProject(Some(project_id))))
               		.unwrap_or(Command::none());

				Command::batch([
					self.update(ProjectPageMessage::LeftClickReleased.into()),
					select_project_command,
				])
			},
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
							match preferences.selected_content_page() {
								SerializedContentPage::Overview => self.update(UiMessage::OpenOverview),
								SerializedContentPage::Project(project_id) => {
									match self.selected_project_id {
										Some(selected_project_id) => self.update(UiMessage::SelectProject(Some(selected_project_id))),
										None => self.update(UiMessage::SelectProject(Some(*project_id))),
									}
								},
							}
						}
						else {
							Command::none()
						}
					},
					LoadDatabaseResult::FailedToOpenFile(_) => {
						if self.database.is_none() {
							self.database = Some(Database::new());
							self.update(DatabaseMessage::Save.into())
						}
						else {
							Command::none()
						}
					},
					LoadDatabaseResult::FailedToParse(filepath) => {
						// saves the corrupted database, just so we don't lose the progress and can correct it afterwards
						let saved_corrupted_filepath = Database::get_filepath().parent().unwrap().join("corrupted - database.json");
						let _ = std::fs::copy(filepath.clone(), saved_corrupted_filepath.clone());
						if self.database.is_none() {
							self.database = Some(Database::new());
						}
						Command::batch([
							self.update(DatabaseMessage::Save.into()),
							self.show_error_msg(format!("Parsing Error:\nFailed to load previous projects in\n\"{}\"\n\nOld corrupted database saved into\n\"{}\"", filepath.display(), saved_corrupted_filepath.display()))
						])
					},
				}
			},
			UiMessage::LoadedPreferences(load_preferences_result) => {
				match load_preferences_result {
					LoadPreferencesResult::Ok(preferences) => {
						self.preferences = Some(preferences);
						self.update(PreferenceMessage::Save.into())
					},
					LoadPreferencesResult::FailedToOpenFile(_) => {
						if self.preferences.is_none() {
							self.preferences = Some(Preferences::default());
							self.update(PreferenceMessage::Save.into())
						}
						else {
							Command::none()
						}
					},
					LoadPreferencesResult::FailedToParse(filepath) => {
						// saves the corrupted preferences, just so we don't lose the progress and can correct it afterwards
						let saved_corrupted_filepath = Preferences::get_filepath().parent().unwrap().join("corrupted - preferences.json");
						let _ = std::fs::copy(filepath.clone(), saved_corrupted_filepath.clone());
						if self.preferences.is_none() {
							self.preferences = Some(Preferences::default());
						}
						Command::batch([
							self.update(PreferenceMessage::Save.into()),
							self.show_error_msg(format!("Parsing Error:\nFailed to load preferences in\n\"{}\"\n\nOld corrupted preferences saved into\n\"{}\"", filepath.display(), saved_corrupted_filepath.display()))
						])
					},
				}
			},
			UiMessage::DatabaseMessage(database_message) => {
				if let Some(database) = &mut self.database {
					let database_command = database.update(database_message.clone());
					let command = match database_message {
						DatabaseMessage::DeleteProject(project_id) => {
							match self.selected_project_id {
								Some(selected_project_id) if selected_project_id == project_id => {
									self.update(UiMessage::OpenOverview)
								},
								_ => Command::none(),
							}
						},
						DatabaseMessage::ChangeProjectColor { .. } => {
							self.update(ProjectPageMessage::HideColorPicker.into())
						},
						DatabaseMessage::SyncFailed(error_msg) => {
							self.show_error_msg(error_msg.clone())
						},
						_ => Command::none(),
					};

					Command::batch([
						command,
						database_command,
					])
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
			UiMessage::OpenOverview => self.update(UiMessage::SelectProject(None)),
			UiMessage::SelectProject(project_id) => {
				self.selected_project_id = project_id;
				let open_project_info = if let Some(database) = &self.database {
					project_id.and_then(|project_id| {
						database.projects()
							.get(&project_id)
							.map(|project| (project_id, project))
					})
				}
				else {
					None
				};
				if let Some((project_id, project)) = open_project_info {
					self.content_page = ContentPage::Project(Box::new(ProjectPage::new(project_id, project)));
					self.update(PreferenceMessage::SetContentPage(SerializedContentPage::Project(project_id)).into())
				}
				else {
					self.content_page = ContentPage::Overview(OverviewPage::new());
					self.update(PreferenceMessage::SetContentPage(SerializedContentPage::Overview).into())
				}
			},
			UiMessage::SwitchToLowerProject => {
				if let Some(database) = &self.database {
					if let Some(selected_project_id) = self.selected_project_id {
						if let Some(order) = database.projects().get_order(&selected_project_id) {
							let lower_order = order + 1;
							let order_to_switch_to = if lower_order < database.projects().len() {
								lower_order
							}
							else {
								0
							};
							return self.update(UiMessage::SwitchToProject { order: order_to_switch_to });
						}
					}
					return self.update(UiMessage::SwitchToProject { order: database.projects().len() - 1 });
				}
				Command::none()
			},
			UiMessage::SwitchToUpperProject => {
				if let Some(selected_project_id) = self.selected_project_id {
					if let Some(database) = &self.database {
						if let Some(order) = database.projects().get_order(&selected_project_id) {
							let order_to_switch_to = if order > 0 {
								order - 1
							}
							else {
								database.projects().len() - 1 // switches to the last project
							};
							return self.update(UiMessage::SwitchToProject { order: order_to_switch_to });
						}
					}
				}
				self.update(UiMessage::SwitchToProject { order: 0 })
			},
			UiMessage::SwitchToProject { order } => {
				if let Some(database) = &self.database {
					let switched_project_id = database.projects().get_key_at_order(order);
					let sidebar_snap_command = self.sidebar_page.snap_to_project(order, database);
					return Command::batch([
						if let Some(project_id) = switched_project_id {
							self.update(UiMessage::SelectProject(Some(*project_id)))
						}
						else {
							Command::none()
						},
						sidebar_snap_command,
						self.update(SwitchProjectModalMessage::Open.into()),
					]);
				}
				Command::none()
			},
			UiMessage::DeleteSelectedProject => {
				if let Some(selected_project_id) = self.selected_project_id {
					if let Some(database) = &self.database {
						if let Some(project) = database.projects().get(&selected_project_id) {
							return self.update(ConfirmModalMessage::open(format!("Delete Project '{}'?", project.name), DatabaseMessage::DeleteProject(selected_project_id)));
						}
					}
				}
				Command::none()
			}
			UiMessage::ProjectPageMessage(message) => {
				match &mut self.content_page {
					ContentPage::Project(project_page) => project_page.update(message, &mut self.database),
					_ => Command::none()
				}
			},
			UiMessage::SidebarPageMessage(message) => {
				let sidebar_command = self.sidebar_page.update(message.clone(), &mut self.database);
				let command = match message {
					SidebarPageMessage::CreateNewProject(project_id) => self.update(UiMessage::SelectProject(Some(project_id))),
					SidebarPageMessage::DragTask { task_id, .. } => {
						match &mut self.content_page {
							ContentPage::Project(project_page) => project_page.update(ProjectPageMessage::DragTask(task_id), &mut self.database),
							_ => Command::none()
						}
					},
					_ => Command::none(),
				};
				Command::batch([
					sidebar_command,
					command
				])
			},
			UiMessage::SettingsModalMessage(message) => self.settings_modal.update(message),
			UiMessage::ManageTaskTagsModalMessage(message) => self.manage_tags_modal.update(message, &mut self.database),
			UiMessage::SwitchProjectModalMessage(message) => self.switch_project_modal.update(message, &self.database, self.selected_project_id),
		}
	}

	fn view(&self) -> Element<UiMessage> {
		let show_sidebar =
			if let Some(preferences) = &self.preferences {
				preferences.show_sidebar()
			}
			else {
				true
			};

		let underlay: Element<UiMessage> = if show_sidebar {
			let sidebar_dividor_position =
				if let Some(preferences) = &self.preferences {
					preferences.sidebar_dividor_position()
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
				container(invisible_toggle_sidebar_button()).padding(Padding { right: PADDING_AMOUNT, top: PADDING_AMOUNT, ..Padding::ZERO }),
			]
			.into()
		};

		let switch_project_modal_view = || {
			if let Some(preferences) = &self.preferences {
				if preferences.show_sidebar() {
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
			.or(self.settings_modal.view(self))
			.or(self.manage_tags_modal.view(self))
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