use crate::{
	components::{
		toggle_sidebar_button, vertical_seperator,
		ScalarAnimation, ICON_BUTTON_WIDTH,
	}, core::{
		Database, DatabaseMessage, LoadDatabaseResult, LoadPreferencesResult, OptionalPreference, PreferenceAction, PreferenceMessage, Preferences, ProjectId, SerializedContentPage, SyncDatabaseResult, TaskId
	}, integrations::{download_database_from_server, sync_database_from_server, upload_database_to_server, SyncServerDatabaseResponse}, modals::{ConfirmModal, ConfirmModalMessage, CreateTaskModal, CreateTaskModalAction, CreateTaskModalMessage, ErrorMsgModal, ErrorMsgModalMessage, ManageTaskTagsModal, ManageTaskTagsModalMessage, SettingsModal, SettingsModalMessage, TaskModal, TaskModalMessage}, pages::{
		ProjectPage, ProjectPageAction, ProjectPageMessage, SidebarPage, SidebarPageAction, SidebarPageMessage, StopwatchPage, StopwatchPageMessage
	}, theme_mode::{get_theme, is_system_theme_dark, system_theme_subscription, ThemeMode}
};
use iced::{
	clipboard,
	event::Status,
	keyboard, time,
	widget::{
		center, container, mouse_area, opaque, responsive, row, stack,
		Space, Stack,
	},
	window, Color, Element, Event,
	Length::Fill,
	Padding, Point, Rectangle, Subscription, Task, Theme,
};
use project_tracker_server::ServerError;
use std::{
	path::PathBuf, sync::Arc, rc::Rc, time::{Duration, SystemTime}
};

pub struct ProjectTrackerApp {
	pub sidebar_page: SidebarPage,
	pub sidebar_animation: ScalarAnimation,
	pub stopwatch_page: StopwatchPage,
	pub project_page: Option<ProjectPage>,
	pub database: Option<Database>,
	pub importing_database: bool,
	pub exporting_database: bool,
	pub syncing_database: bool,
	pub syncing_database_from_server: bool,
	pub preferences: Option<Preferences>,
	pub confirm_modal: ConfirmModal,
	pub error_msg_modal: ErrorMsgModal,
	pub settings_modal: SettingsModal,
	pub manage_tags_modal: ManageTaskTagsModal,
	pub create_task_modal: CreateTaskModal,
	pub task_modal: TaskModal,
	pub is_system_theme_dark: bool,
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub enum Message {
	CloseWindowRequested(window::Id),
	EscapePressed,
	EnterPressed,
	CopyToClipboard(String),
	OpenUrl(String),
	SaveChangedFiles,
	OpenFolderLocation(PathBuf),
	SystemTheme {
		is_dark: bool,
	},
	ConfirmModalMessage(ConfirmModalMessage),
	ConfirmModalConfirmed(Box<Message>),
	ErrorMsgModalMessage(ErrorMsgModalMessage),
	SaveDatabase,
	DatabaseSaved(SystemTime), // begin_time since saving
	ExportDatabase(PathBuf),
	ExportDatabaseDialog,
	ExportDatabaseFailed(String),
	ExportDatabaseDialogCanceled,
	DatabaseExported,
	ImportDatabase(PathBuf),
	ImportDatabaseDialog,
	ImportDatabaseDialogCanceled,
	SyncDatabase(PathBuf),
	SyncDatabaseUpload(PathBuf),
	SyncDatabaseUploaded,
	SyncDatabaseFailed(String), // error_msg
	LoadedDatabase(LoadDatabaseResult),
	LoadedPreferences(LoadPreferencesResult),
	SyncDatabaseFromServer,
	DownloadDatabaseFromServer,
	UploadDatabaseToServer,
	DatabaseUploadedToServer,
	ServerError(Arc<ServerError>),
	DatabaseMessage(DatabaseMessage),
	PreferenceMessage(PreferenceMessage),
	SelectProject(Option<ProjectId>),
	SwitchToUpperProject, // switches to upper project when using shortcuts
	SwitchToLowerProject, // switches to lower project when using shortcuts
	SwitchToProject {
		order: usize,
	}, // switches to project when using shortcuts
	DeleteSelectedProject,
	DragTask {
		project_id: ProjectId,
		task_id: TaskId,
		task_is_todo: bool,
		point: Point,
		rect: Rectangle,
	},
	CancelDragTask,
	OpenStopwatch,
	StopwatchPageMessage(StopwatchPageMessage),
	ProjectPageMessage(ProjectPageMessage),
	SidebarPageMessage(SidebarPageMessage),
	ToggleSidebar,
	AnimateSidebar,
	SettingsModalMessage(SettingsModalMessage),
	OpenCreateTaskModal,
	CreateTaskModalMessage(CreateTaskModalMessage),
	TaskModalMessage(TaskModalMessage),
	ManageTaskTagsModalMessage(ManageTaskTagsModalMessage),
}

impl ProjectTrackerApp {
	fn show_error_msg(&mut self, error_msg: String) -> Task<Message> {
		self.update(ErrorMsgModalMessage::open(error_msg))
	}

	pub fn is_theme_dark(&self) -> bool {
		if let Some(preferences) = &self.preferences {
			match preferences.theme_mode() {
				ThemeMode::System => self.is_system_theme_dark,
				ThemeMode::Dark => true,
				ThemeMode::Light => false,
			}
		} else {
			self.is_system_theme_dark
		}
	}

	pub fn get_theme(&self) -> &'static Theme {
		get_theme(self.is_theme_dark())
	}

	pub fn new() -> (Self, Task<Message>) {
		(
			Self {
				sidebar_page: SidebarPage::new(),
				sidebar_animation: ScalarAnimation::Idle,
				stopwatch_page: StopwatchPage::default(),
				project_page: None,
				database: None,
				importing_database: false,
				exporting_database: false,
				syncing_database: false,
				syncing_database_from_server: false,
				preferences: None,
				confirm_modal: ConfirmModal::Closed,
				error_msg_modal: ErrorMsgModal::Closed,
				settings_modal: SettingsModal::Closed,
				manage_tags_modal: ManageTaskTagsModal::Closed,
				create_task_modal: CreateTaskModal::Closed,
				task_modal: TaskModal::Closed,
				is_system_theme_dark: is_system_theme_dark(),
			},
			Task::batch([
				Task::perform(Preferences::load(), Message::LoadedPreferences),
				Task::perform(Database::load(), Message::LoadedDatabase),
			]),
		)
	}

	pub fn theme(&self) -> Theme {
		self.get_theme().clone()
	}

	pub fn subscription(&self) -> Subscription<Message> {
		Subscription::batch([
			keyboard::on_key_press(|key, modifiers| match key.as_ref() {
				keyboard::Key::Character("b") if modifiers.command() => {
					Some(Message::ToggleSidebar)
				}
				keyboard::Key::Character("h") if modifiers.command() => {
					Some(Message::OpenStopwatch)
				}
				keyboard::Key::Named(keyboard::key::Named::Escape) => {
					Some(Message::EscapePressed)
				}
				keyboard::Key::Named(keyboard::key::Named::Enter) => Some(Message::EnterPressed),
				keyboard::Key::Named(keyboard::key::Named::Delete) if modifiers.command() => {
					Some(Message::DeleteSelectedProject)
				},
				keyboard::Key::Named(keyboard::key::Named::Tab) if modifiers.command() => Some(
					if modifiers.shift() {
						Message::SwitchToUpperProject
					} else {
						Message::SwitchToLowerProject
					}
				),
				_ => None,
			}),
			iced::event::listen_with(move |event, status, id| match event {
				Event::Window(window::Event::CloseRequested)
					if matches!(status, Status::Ignored) =>
				{
					Some(Message::CloseWindowRequested(id))
				}
				_ => None,
			}),
			self.sidebar_page
				.subscription()
				.map(Message::SidebarPageMessage),
			self.sidebar_animation
				.subscription()
				.map(|_| Message::AnimateSidebar),
			self.stopwatch_page
				.subscription(self.project_page.is_none())
				.map(Message::StopwatchPageMessage),
			if let Some(project_page) = &self.project_page {
				project_page
					.subscription()
					.map(Message::ProjectPageMessage)
			} else {
				Subscription::none()
			},
			self.settings_modal
				.subscription()
				.map(Message::SettingsModalMessage),
			self.create_task_modal.subscription(),
			time::every(Duration::from_secs(1)).map(|_| Message::SaveChangedFiles),
			system_theme_subscription(),
		])
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::CloseWindowRequested(id) => Task::batch([
				self.update(Message::SaveDatabase),
				self.update(PreferenceMessage::Save.into()),
				window::close(id),
			]),
			Message::EscapePressed => {
				if matches!(self.error_msg_modal, ErrorMsgModal::Open { .. }) {
					return self.update(ErrorMsgModalMessage::Close.into());
				}
				if matches!(self.confirm_modal, ConfirmModal::Opened { .. }) {
					return self.update(ConfirmModalMessage::Close.into());
				}
				if matches!(self.settings_modal, SettingsModal::Opened { .. }) {
					return self.update(SettingsModalMessage::Close.into());
				}
				if matches!(self.manage_tags_modal, ManageTaskTagsModal::Opened { .. }) {
					return self.update(ManageTaskTagsModalMessage::Close.into());
				}
				if matches!(self.create_task_modal, CreateTaskModal::Opened { .. }) {
					return self.update(CreateTaskModalMessage::Close.into());
				}
				if matches!(self.task_modal, TaskModal::Opened { .. }) {
					return self.update(TaskModalMessage::Close.into());
				}
				if self.project_page.is_some() {
					self.update(ProjectPageMessage::HideColorPicker.into())
				} else {
					self.update(StopwatchPageMessage::Stop.into())
				}
			}
			Message::EnterPressed => {
				if matches!(self.error_msg_modal, ErrorMsgModal::Open { .. }) {
					self.error_msg_modal = ErrorMsgModal::Closed;
					Task::none()
				} else {
					match &self.confirm_modal {
						ConfirmModal::Opened { on_confirmed, .. } => self.update(
							Message::ConfirmModalConfirmed(Box::new(on_confirmed.clone())),
						),
						ConfirmModal::Closed => Task::none(),
					}
				}
			}
			Message::CopyToClipboard(copied_text) => clipboard::write(copied_text),
			Message::OpenUrl(url) => {
				let _ = open::that_detached(url.as_str());
				Task::none()
			}
			Message::SaveChangedFiles => {
				let mut commands = Vec::new();
				if let Some(database) = &mut self.database {
					if database.has_unsaved_changes() {
						commands.push(self.update(Message::SaveDatabase));
					}
				}
				if let Some(preferences) = &mut self.preferences {
					if preferences.has_unsaved_changes() {
						let action = preferences.update(PreferenceMessage::Save);
						commands.push(self.perform_preference_action(action));
					}
				}
				Task::batch(commands)
			}
			Message::OpenFolderLocation(filepath) => {
				let _ = open::that(filepath);
				Task::none()
			}
			Message::SystemTheme { is_dark } => {
				self.is_system_theme_dark = is_dark;
				Task::none()
			}
			Message::ConfirmModalConfirmed(message) => Task::batch([
				self.update(*message),
				self.update(ConfirmModalMessage::Close.into()),
			]),
			Message::ConfirmModalMessage(message) => {
				self.confirm_modal.update(message);
				Task::none()
			}
			Message::ErrorMsgModalMessage(message) => {
				self.error_msg_modal.update(message);
				Task::none()
			}
			Message::SaveDatabase => {
				if let Some(database) = &self.database {
					Task::perform(Database::save(database.to_json()), |result| match result {
						Ok(begin_time) => Message::DatabaseSaved(begin_time),
						Err(error_msg) => ErrorMsgModalMessage::open(error_msg),
					})
				} else {
					Task::none()
				}
			}
			Message::DatabaseSaved(saved_time) => {
				if let Some(database) = &mut self.database {
					database.last_saved_time = saved_time;
				}
				Task::none()
			}
			Message::ExportDatabaseDialog => {
				Task::perform(Database::export_file_dialog(), |filepath| match filepath {
					Some(filepath) => Message::ExportDatabase(filepath),
					None => Message::ExportDatabaseDialogCanceled,
				})
			}
			Message::ExportDatabaseDialogCanceled => {
				self.exporting_database = false;
				Task::none()
			}
			Message::ExportDatabase(filepath) => {
				if let Some(database) = &self.database {
					self.exporting_database = true;
					Task::perform(Database::save_to(filepath, database.to_json()), |result| {
						match result {
							Ok(_) => Message::DatabaseExported,
							Err(e) => Message::ExportDatabaseFailed(e),
						}
					})
				} else {
					Task::none()
				}
			}
			Message::ExportDatabaseFailed(error_msg) => {
				self.exporting_database = false;
				self.show_error_msg(error_msg)
			}
			Message::DatabaseExported => {
				self.exporting_database = false;
				Task::none()
			}
			Message::ImportDatabaseDialog => {
				Task::perform(Database::import_file_dialog(), |filepath| {
					if let Some(filepath) = filepath {
						Message::ImportDatabase(filepath)
					} else {
						Message::ImportDatabaseDialogCanceled
					}
				})
			}
			Message::ImportDatabaseDialogCanceled => {
				self.importing_database = false;
				Task::none()
			}
			Message::ImportDatabase(filepath) => {
				self.importing_database = true;
				Task::perform(Database::load_from(filepath), Message::LoadedDatabase)
			}
			Message::SyncDatabase(filepath) => {
				self.syncing_database = true;
				Task::perform(
					Database::sync(filepath.clone()),
					move |result| match result {
						SyncDatabaseResult::InvalidSynchronizationFilepath => {
							Message::SyncDatabaseFailed(format!(
								"Failed to open synchronization file in\n\"{}\"",
								filepath.display()
							))
						}
						SyncDatabaseResult::Upload => {
							Message::SyncDatabaseUpload(filepath.clone())
						}
						SyncDatabaseResult::Download => Message::ImportDatabase(filepath.clone()),
					},
				)
			}
			Message::SyncDatabaseUpload(filepath) => {
				if let Some(database) = &self.database {
					Task::perform(Database::save_to(filepath, database.to_json()), |_| {
						Message::SyncDatabaseUploaded
					})
				} else {
					Task::none()
				}
			}
			Message::SyncDatabaseUploaded => {
				self.syncing_database = false;
				Task::none()
			}
			Message::SyncDatabaseFailed(error_msg) => {
				self.syncing_database = false;
				self.show_error_msg(error_msg)
			}
			Message::LoadedDatabase(load_database_result) => {
				match load_database_result {
					LoadDatabaseResult::Ok(database) => {
						self.database = Some(database);
						self.importing_database = false;
						self.syncing_database = false;
						self.syncing_database_from_server = false;
						let task = if let Some(preferences) = &self.preferences {
							let stopwatch_progress_message: Option<Message> =
								preferences.stopwatch_progress().as_ref().map(|progress| {
									StopwatchPageMessage::StartupAgain {
										task: progress.task,
										elapsed_time: Duration::from_secs(
											progress.elapsed_time_seconds,
										),
										paused: progress.paused,
										finished_notification_sent: progress
											.finished_notification_sent,
									}
									.into()
								});

							let selected_content_page = *preferences.selected_content_page();

							Task::batch([
								if let Some(stopwatch_progress_message) = stopwatch_progress_message
								{
									self.update(stopwatch_progress_message)
								} else {
									Task::none()
								},
								match selected_content_page {
									SerializedContentPage::Stopwatch => {
										self.update(Message::OpenStopwatch)
									}
									SerializedContentPage::Project(project_id) => {
										match &self.project_page {
											Some(project_page) => {
												self.update(Message::SelectProject(Some(
													project_page.project_id,
												)))
											}
											None => self
												.update(Message::SelectProject(Some(project_id))),
										}
									}
								},
							])
						} else {
							Task::none()
						};

						Task::batch([task, self.update(Message::SaveDatabase)])
					}
					LoadDatabaseResult::FailedToOpenFile(filepath) => {
						self.importing_database = false;
						self.syncing_database = false;
						let command = if self.database.is_none() {
							self.database = Some(Database::default());
							self.update(Message::SaveDatabase)
						} else {
							Task::none()
						};

						Task::batch([
							command,
							self.show_error_msg(format!(
								"Failed to open database file:\n{}",
								filepath.display()
							)),
						])
					}
					LoadDatabaseResult::FailedToParse(filepath) => {
						// saves the corrupted database, just so we don't lose the progress and can correct it afterwards
						let saved_corrupted_filepath = Database::get_filepath()
							.parent()
							.unwrap()
							.join("corrupted - database.json");
						let _ = std::fs::copy(filepath.clone(), saved_corrupted_filepath.clone());
						if self.database.is_none() {
							self.database = Some(Database::default());
						}
						Task::batch([
							self.update(Message::SaveDatabase),
							self.show_error_msg(format!("Parsing Error:\nFailed to load previous projects in\n\"{}\"\n\nOld corrupted database saved into\n\"{}\"", filepath.display(), saved_corrupted_filepath.display()))
						])
					},
				}
			}
			Message::LoadedPreferences(load_preferences_result) => {
				match load_preferences_result {
					LoadPreferencesResult::Ok(preferences) => {
						self.preferences = Some(preferences);
						self.update(PreferenceMessage::Save.into())
					}
					LoadPreferencesResult::FailedToOpenFile(_) => {
						if self.preferences.is_none() {
							self.preferences = Some(Preferences::default());
							self.update(PreferenceMessage::Save.into())
						} else {
							Task::none()
						}
					}
					LoadPreferencesResult::FailedToParse(filepath) => {
						// saves the corrupted preferences, just so we don't lose the progress and can correct it afterwards
						let saved_corrupted_filepath = Preferences::get_filepath()
							.parent()
							.unwrap()
							.join("corrupted - preferences.json");
						let _ = std::fs::copy(filepath.clone(), saved_corrupted_filepath.clone());
						if self.preferences.is_none() {
							self.preferences = Some(Preferences::default());
						}
						Task::batch([
							self.update(PreferenceMessage::Save.into()),
							self.show_error_msg(format!("Parsing Error:\nFailed to load preferences in\n\"{}\"\n\nOld corrupted preferences saved into\n\"{}\"", filepath.display(), saved_corrupted_filepath.display()))
						])
					}
				}
			},
			Message::SyncDatabaseFromServer => {
				if let Some(server_config) = self.preferences.server_synchronization().cloned() {
					if let Some(database) = &self.database {
						self.syncing_database_from_server = true;

						return Task::perform(
							sync_database_from_server(
								server_config,
								(*database.last_changed_time()).into()
							),
							|result| match result {
								Ok(sync_response) => match sync_response {
									SyncServerDatabaseResponse::DownloadDatabase => Message::DownloadDatabaseFromServer,
									SyncServerDatabaseResponse::UploadDatabase => Message::UploadDatabaseToServer,
								},
								Err(e) => Message::ServerError(Arc::new(e)),
							}
						);
					}
				}
				Task::none()
			},
			Message::DownloadDatabaseFromServer => {
				if let Some(server_config) = self.preferences.server_synchronization().cloned() {
					Task::perform(
						download_database_from_server(server_config),
						|result| match result {
							Ok(database) => Message::LoadedDatabase(LoadDatabaseResult::Ok(database)),
							Err(e) => Message::ServerError(Arc::new(e)),
						}
					)
				}
				else {
					Task::none()
				}
			},
			Message::UploadDatabaseToServer => {
				if let Some(database) = &self.database {
					if let Some(server_config) = self.preferences.server_synchronization().cloned() {
						return Task::perform(
							upload_database_to_server(server_config, database.to_json()),
							|result| match result {
								Ok(_) => Message::DatabaseUploadedToServer,
								Err(e) => Message::ServerError(Arc::new(e)),
							}
						);
					}
				}
				Task::none()
			},
			Message::ServerError(e) => {
				self.syncing_database_from_server = false;
				self.update(ErrorMsgModalMessage::from_server_error(e.as_ref()))
			},
			Message::DatabaseUploadedToServer => {
				self.syncing_database_from_server = false;
				Task::none()
			},
			Message::DatabaseMessage(database_message) => {
				if let Some(database) = &mut self.database {
					database.update(database_message.clone());
					match database_message {
						DatabaseMessage::DeleteProject(project_id) => match &self.project_page {
							Some(project_page) if project_page.project_id == project_id => {
								self.update(Message::OpenStopwatch)
							}
							_ => Task::none(),
						},
						_ => Task::none(),
					}
				} else {
					Task::none()
				}
			}
			Message::PreferenceMessage(preference_message) => {
				let action = if let Some(preferences) = &mut self.preferences {
					preferences.update(preference_message)
				} else {
					PreferenceAction::None
				};
				self.perform_preference_action(action)
			}
			Message::OpenStopwatch => self.update(Message::SelectProject(None)),
			Message::StopwatchPageMessage(message) => {
				if matches!(message, StopwatchPageMessage::Start { .. }) {
					self.task_modal = TaskModal::Closed;
				}
				let messages = self.stopwatch_page.update(
					message,
					&self.database,
					self.project_page.is_none(),
				);
				if let Some(messages) = messages {
					let tasks: Vec<Task<Message>> =
						messages.into_iter().map(|msg| self.update(msg)).collect();
					Task::batch(tasks)
				} else {
					Task::none()
				}
			}
			Message::SelectProject(project_id) => {
				let open_project_info = if let Some(database) = &self.database {
					project_id.and_then(|project_id| {
						database
							.get_project(&project_id)
							.map(|project| (project_id, project))
					})
				} else {
					None
				};
				if let Some((project_id, project)) = open_project_info {
					self.project_page = Some(ProjectPage::new(project_id, project, &self.preferences));
					self.update(
						PreferenceMessage::SetContentPage(SerializedContentPage::Project(
							project_id,
						))
						.into(),
					)
				} else {
					self.project_page = None;
					self.update(
						PreferenceMessage::SetContentPage(SerializedContentPage::Stopwatch).into(),
					)
				}
			}
			Message::SwitchToLowerProject => {
				if let Some(database) = &self.database {
					if let Some(project_page) = &self.project_page {
						if let Some(order) = database.projects().get_order(&project_page.project_id)
						{
							let lower_order = order + 1;
							let order_to_switch_to = if lower_order < database.projects().len() {
								lower_order
							} else {
								0
							};
							return self.update(Message::SwitchToProject {
								order: order_to_switch_to,
							});
						}
					}
				}
				self.update(Message::SwitchToProject { order: 0 })
			}
			Message::SwitchToUpperProject => {
				if let Some(database) = &self.database {
					if let Some(project_page) = &self.project_page {
						if let Some(order) = database.projects().get_order(&project_page.project_id)
						{
							let order_to_switch_to = if order > 0 {
								order - 1
							} else {
								database.projects().len() - 1 // switches to the last project
							};
							return self.update(Message::SwitchToProject {
								order: order_to_switch_to,
							});
						}
					}
					return self.update(Message::SwitchToProject {
						order: database.projects().len() - 1,
					});
				}
				self.update(Message::SwitchToProject { order: 0 })
			}
			Message::SwitchToProject { order } => {
				if let Some(database) = &self.database {
					let switched_project_id = database.projects().get_key_at_order(order);
					let sidebar_snap_command = self.sidebar_page.snap_to_project(order, database);
					return Task::batch([
						if let Some(project_id) = switched_project_id {
							self.update(Message::SelectProject(Some(*project_id)))
						} else {
							Task::none()
						},
						sidebar_snap_command,
					]);
				}
				Task::none()
			}
			Message::DeleteSelectedProject => {
				if let Some(project_page) = &self.project_page {
					if let Some(database) = &self.database {
						if let Some(project) = database.get_project(&project_page.project_id) {
							return self.update(ConfirmModalMessage::open(
								format!("Delete Project '{}'?", project.name),
								DatabaseMessage::DeleteProject(project_page.project_id),
							));
						}
					}
				}
				Task::none()
			}
			Message::DragTask {
				project_id,
				task_id,
				task_is_todo,
				point,
				rect,
			} => {
				let is_theme_dark = self.is_theme_dark();

				Task::batch([
					self.project_page
						.as_mut()
						.map(|project_page| {
							project_page.update(
								ProjectPageMessage::DragTask { task_id, point },
								&mut self.database,
								&self.preferences
							)
						})
						.map(|action| self.perform_project_page_action(action))
						.unwrap_or(Task::none()),
					match self.sidebar_page.update(
						SidebarPageMessage::DragTask {
							project_id,
							task_id,
							task_is_todo,
							point,
							rect,
						},
						&mut self.database,
						&mut self.stopwatch_page,
						is_theme_dark,
					) {
						SidebarPageAction::None => Task::none(),
						SidebarPageAction::Task(task) => task,
						SidebarPageAction::SelectProject(project_id) => {
							self.update(Message::SelectProject(Some(project_id)))
						}
					},
				])
			}
			Message::CancelDragTask => {
				let is_theme_dark = self.is_theme_dark();

				Task::batch([
					self.project_page
						.as_mut()
						.map(|project_page| {
							project_page.update(
								ProjectPageMessage::CancelDragTask,
								&mut self.database,
								&self.preferences
							)
						})
						.map(|action| self.perform_project_page_action(action))
						.unwrap_or(Task::none()),
					match self.sidebar_page.update(
						SidebarPageMessage::CancelDragTask,
						&mut self.database,
						&mut self.stopwatch_page,
						is_theme_dark,
					) {
						SidebarPageAction::None => Task::none(),
						SidebarPageAction::Task(task) => task,
						SidebarPageAction::SelectProject(project_id) => {
							self.update(Message::SelectProject(Some(project_id)))
						}
					},
				])
			}
			Message::ProjectPageMessage(message) => match &mut self.project_page {
				Some(project_page) => {
					let action = project_page.update(message, &mut self.database, &self.preferences);
					self.perform_project_page_action(action)
				},
				None => Task::none(),
			},
			Message::SidebarPageMessage(message) => {
				let is_theme_dark = self.is_theme_dark();
				let sidebar_action = self.sidebar_page.update(
					message.clone(),
					&mut self.database,
					&mut self.stopwatch_page,
					is_theme_dark,
				);

				match sidebar_action {
					SidebarPageAction::None => Task::none(),
					SidebarPageAction::Task(task) => task,
					SidebarPageAction::SelectProject(project_id) => {
						self.update(Message::SelectProject(Some(project_id)))
					}
				}
			}
			Message::ToggleSidebar => {
				self.sidebar_animation = if self.preferences.show_sidebar() {
					ScalarAnimation::start(SidebarPage::SPLIT_LAYOUT_PERCENTAGE, 0.0, 0.15)
				} else {
					ScalarAnimation::start(0.0, SidebarPage::SPLIT_LAYOUT_PERCENTAGE, 0.15)
				};

				self.update(PreferenceMessage::ToggleShowSidebar.into())
			}
			Message::AnimateSidebar => {
				self.sidebar_animation.update();
				Task::none()
			}
			Message::SettingsModalMessage(message) => {
				let action = self.settings_modal.update(message, &mut self.preferences);
				self.perform_preference_action(action)
			},
			Message::ManageTaskTagsModalMessage(message) => {
				let deleted_task_tag_id =
					if let ManageTaskTagsModalMessage::DeleteTaskTag(task_tag_id) = &message {
						Some(*task_tag_id)
					} else {
						None
					};

				Task::batch([
					self.manage_tags_modal.update(message, &mut self.database),
					deleted_task_tag_id
						.and_then(|deleted_task_tag_id| {
							let action = self.project_page.as_mut().map(|project_page| {
								project_page.update(
									ProjectPageMessage::UnsetFilterTaskTag(deleted_task_tag_id),
									&mut self.database,
									&self.preferences
								)
							});
							action.map(|action| self.perform_project_page_action(action))
						})
						.unwrap_or(Task::none()),
				])
			},
			Message::CreateTaskModalMessage(message) => {
				let action = self.create_task_modal.update(message);
				match action {
					CreateTaskModalAction::None => Task::none(),
					CreateTaskModalAction::Task(task) => task.map(Message::CreateTaskModalMessage),
					CreateTaskModalAction::CreateTask{ project_id, task_id, task_name, task_description, task_tags } => Task::batch([
						self.update(DatabaseMessage::CreateTask {
							project_id,
							task_id,
							task_name,
							task_description,
							task_tags,
							create_at_top: self.preferences.create_new_tasks_at_top(),
						}.into()),
						self.update(ProjectPageMessage::RefreshCachedTaskList.into()),
					]),
				}
			},
			Message::OpenCreateTaskModal => if let Some(project_id) = self.project_page.as_ref().map(|project_page| project_page.project_id) {
				self.update(CreateTaskModalMessage::Open(project_id).into())
			}
			else {
				Task::none()
			},
			Message::TaskModalMessage(message) => self.task_modal.update(message, &mut self.database).map(Message::TaskModalMessage),
		}
	}

	fn perform_project_page_action(&mut self, action: ProjectPageAction) -> Task<Message> {
		match action {
			ProjectPageAction::None => Task::none(),
			ProjectPageAction::Task(task) => task,
			ProjectPageAction::OpenManageTaskTagsModal(project_id) => self.update(
				ManageTaskTagsModalMessage::Open { project_id }.into()
			),
			ProjectPageAction::ConfirmDeleteProject { project_id, project_name } => self.update(ConfirmModalMessage::open(
				format!("Delete Project '{project_name}'?"),
				DatabaseMessage::DeleteProject(project_id),
			)),
			ProjectPageAction::OpenTaskModal { project_id, task_id } => self.update(TaskModalMessage::Open {
				project_id,
				task_id
			}.into()),
		}
	}

	fn perform_preference_action(&mut self, action: PreferenceAction) -> Task<Message> {
		match action {
			PreferenceAction::None => Task::none(),
			PreferenceAction::Task(task) => task,
			PreferenceAction::RefreshCachedTaskList => {
				if let Some(project_page) = &mut self.project_page {
					if let Some(database) = &self.database {
						project_page.generate_cached_task_list(database, &self.preferences);
					}
				}
				Task::none()
			},
		}
	}

	fn modal<Message: 'static + Clone>(
		content: Option<Element<Message>>,
		on_close: Message,
	) -> Option<Element<Message>> {
		content.map(|content| {
			opaque(
				mouse_area(center(opaque(content)).style(|_theme| {
					container::Style {
						background: Some(
							Color {
								a: 0.75,
								..Color::BLACK
							}
							.into(),
						),
						..Default::default()
					}
				}))
				.on_press(on_close),
			)
		})
	}

	pub fn view(&self) -> Element<Message> {
		let show_sidebar = if let Some(preferences) = &self.preferences {
			preferences.show_sidebar()
		} else {
			true
		};

		let sidebar_animation_value = self.sidebar_animation.get_value();
		// 0.0..1.0
		let sidebar_animation_percentage = sidebar_animation_value
			.map(|value| (1.0 - value / SidebarPage::SPLIT_LAYOUT_PERCENTAGE));

		let underlay: Element<Message> = {
			let sidebar_layout_percentage = sidebar_animation_value.unwrap_or(if show_sidebar {
				SidebarPage::SPLIT_LAYOUT_PERCENTAGE
			} else {
				0.0
			});
			// 0.0..1.0
			let sidebar_animation_percentage =
				sidebar_animation_percentage.unwrap_or(if show_sidebar { 0.0 } else { 1.0 });

			let arc_self = Rc::new(self);

			responsive(move |size| {
				let empty_toggle_sidebar_button_layout_width =
					ICON_BUTTON_WIDTH * sidebar_animation_percentage;

				let sidebar: Element<Message> = if show_sidebar
					|| sidebar_animation_value.is_some()
				{
					container(arc_self.sidebar_page.view(*arc_self))
						.width(Fill)
						.padding(
							Padding::default()
								.right(size.width * (1.0 - SidebarPage::SPLIT_LAYOUT_PERCENTAGE)),
						)
						.into()
				} else {
					Space::new(Fill, Fill).into()
				};

				let seperator: Element<Message> =
					if show_sidebar || sidebar_animation_value.is_some() {
						vertical_seperator().into()
					} else {
						Space::new(0.0, 0.0).into()
					};

				stack![
					sidebar,
					container(
						container(row![
							seperator,
							row![
								if show_sidebar || sidebar_animation_value.is_some() {
									Space::with_width(empty_toggle_sidebar_button_layout_width)
										.into()
								} else {
									toggle_sidebar_button(false)
								},
								arc_self.content_view(),
								Space::with_width(empty_toggle_sidebar_button_layout_width),
							]
						])
						.style(|t| container::Style {
							background: Some(t.extended_palette().background.base.color.into()),
							..Default::default()
						})
					)
					.width(Fill)
					.padding(Padding::default().left(size.width * sidebar_layout_percentage))
				]
				.into()
			})
			.into()
		};

		Stack::new()
			.push(underlay)
			.push_maybe(Self::modal(
				self.create_task_modal.view(&self.database),
				CreateTaskModalMessage::Close,
			)
			.map(|element| element.map(Message::CreateTaskModalMessage)))
			.push_maybe(Self::modal(
				self.task_modal.view(self),
				TaskModalMessage::Close.into(),
			))
			.push_maybe(Self::modal(
				self.manage_tags_modal.view(self),
				ManageTaskTagsModalMessage::Close.into(),
			))
			.push_maybe(Self::modal(
				self.settings_modal.view(self),
				SettingsModalMessage::Close.into(),
			))
			.push_maybe(Self::modal(
				self.confirm_modal.view(),
				ConfirmModalMessage::Close.into(),
			))
			.push_maybe(Self::modal(
				self.error_msg_modal.view(),
				ErrorMsgModalMessage::Close.into(),
			))
			.into()
	}

	fn content_view(&self) -> Element<Message> {
		self.project_page
			.as_ref()
			.map(|project_page| project_page.view(self))
			.unwrap_or(self.stopwatch_page.view(self))
	}
}
