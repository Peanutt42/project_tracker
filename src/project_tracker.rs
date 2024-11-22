use crate::{
	components::{
		create_empty_database_button, import_database_button, toggle_sidebar_button, ScalarAnimation, ICON_BUTTON_WIDTH
	}, core::{
		Database, DatabaseMessage, LoadDatabaseError, LoadPreferencesError, OptionalPreference, PreferenceAction, PreferenceMessage, Preferences, ProjectId, SyncDatabaseResult, SynchronizationSetting, TaskId
	}, integrations::{sync_database_from_server, SyncServerDatabaseResponse}, modals::{ConfirmModal, ConfirmModalMessage, CreateTaskModal, CreateTaskModalAction, CreateTaskModalMessage, ErrorMsgModal, ErrorMsgModalMessage, ManageTaskTagsModal, ManageTaskTagsModalAction, ManageTaskTagsModalMessage, SettingsModal, SettingsModalMessage, TaskModal, TaskModalAction, TaskModalMessage, WaitClosingModal, WaitClosingModalMessage}, pages::{
		ContentPage, ContentPageAction, ContentPageMessage, OverviewPageMessage, ProjectPageMessage, SidebarPage, SidebarPageAction, SidebarPageMessage, StopwatchPageMessage
	}, styles::{default_background_container_style, modal_background_container_style, sidebar_background_container_style, HEADING_TEXT_SIZE, LARGE_SPACING_AMOUNT, MINIMAL_DRAG_DISTANCE}, theme_mode::{get_theme, is_system_theme_dark, system_theme_subscription, ThemeMode}
};
use iced::{
	alignment::Horizontal, clipboard, event::Status, keyboard, mouse, time, widget::{
		center, column, container, mouse_area, opaque, responsive, row, stack, text, Space, Stack
	}, window, Element, Event, Length::Fill, Padding, Point, Rectangle, Subscription, Task, Theme
};
use project_tracker_server::{get_last_modification_date_time, ServerError};
use std::{
	path::PathBuf, rc::Rc, sync::Arc, time::{Duration, Instant, SystemTime}
};

pub struct ProjectTrackerApp {
	pub sidebar_page: SidebarPage,
	pub sidebar_animation: ScalarAnimation,
	pub content_page: ContentPage,
	pub database: Option<Database>,
	pub loading_database: bool,
	pub importing_database: bool,
	pub exporting_database: bool,
	pub syncing_database: bool,
	pub syncing_database_from_server: bool,
	pub last_sync_time: Option<Instant>,
	pub preferences: Option<Preferences>,
	pub confirm_modal: ConfirmModal,
	pub error_msg_modal: ErrorMsgModal,
	pub wait_closing_modal: WaitClosingModal,
	pub settings_modal: SettingsModal,
	pub manage_tags_modal: ManageTaskTagsModal,
	pub create_task_modal: CreateTaskModal,
	pub task_modal: TaskModal,
	pub pressed_task: Option<(ProjectId, TaskId)>,
	pub dragged_task: Option<TaskId>,
	pub start_dragging_point: Option<Point>,
	pub just_minimal_dragging: bool,
	pub is_system_theme_dark: bool,
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub enum Message {
	TryClosing,
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
	WaitClosingModalMessage(WaitClosingModalMessage),
	SaveDatabase,
	DatabaseSaved(SystemTime), // begin_time since saving
	ExportDatabase(PathBuf),
	ExportDatabaseDialog,
	ExportDatabaseFailed(String),
	ExportDatabaseDialogCanceled,
	DatabaseExported,
	ImportDatabase(PathBuf),
	DatabaseImported(Result<Database, Arc<LoadDatabaseError>>),
	ImportDatabaseDialog,
	ImportDatabaseDialogCanceled,
	SyncDatabase,
	SyncDatabaseFilepath(PathBuf),
	SyncDatabaseFilepathUpload(PathBuf),
	SyncDatabaseFilepathUploaded,
	SyncDatabaseFilepathDownload(PathBuf),
	SyncDatabaseFilepathDownloaded(Result<Database, Arc<LoadDatabaseError>>),
	SyncDatabaseFilepathFailed(String), // error_msg
	LoadedDatabase(Result<Database, Arc<LoadDatabaseError>>),
	LoadedPreferences(Result<Preferences, Arc<LoadPreferencesError>>),
	SyncDatabaseFromServer,
	DatabaseDownloadedFromServer(Database),
	DatabaseUploadedToServer,
	ServerError(Arc<ServerError>),
	DatabaseMessage(DatabaseMessage),
	PreferenceMessage(PreferenceMessage),
	SwitchToUpperProject, // switches to upper project when using shortcuts
	SwitchToLowerProject, // switches to lower project when using shortcuts
	SwitchToProject {
		order: usize,
	}, // switches to project when using shortcuts
	DeleteSelectedProject,
	PressTask {
		project_id: ProjectId,
		task_id: TaskId
	},
	DragTask {
		project_id: ProjectId,
		task_id: TaskId,
		task_is_todo: bool,
		point: Point,
		rect: Rectangle,
	},
	CancelDragTask,
	LeftClickReleased,
	ContentPageMessage(ContentPageMessage),
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
				content_page: ContentPage::new(&None, &None),
				database: None,
				loading_database: true,
				importing_database: false,
				exporting_database: false,
				syncing_database: false,
				syncing_database_from_server: false,
				last_sync_time: None,
				preferences: None,
				confirm_modal: ConfirmModal::Closed,
				error_msg_modal: ErrorMsgModal::Closed,
				wait_closing_modal: WaitClosingModal::Closed,
				settings_modal: SettingsModal::Closed,
				manage_tags_modal: ManageTaskTagsModal::Closed,
				create_task_modal: CreateTaskModal::Closed,
				task_modal: TaskModal::Closed,
				pressed_task: None,
				dragged_task: None,
				start_dragging_point: None,
				just_minimal_dragging: true,
				is_system_theme_dark: is_system_theme_dark(),
			},
			Task::batch([
				Task::perform(Preferences::load(), |result| Message::LoadedPreferences(result.map_err(Arc::new))),
				Task::perform(Database::load(), |result| Message::LoadedDatabase(result.map_err(Arc::new))),
			]),
		)
	}

	pub fn theme(&self) -> Theme {
		self.get_theme().clone()
	}

	pub fn title(&self) -> String {
		let progress_str = if self.syncing_database || self.syncing_database_from_server {
			" - Syncing..."
		}
		else if self.exporting_database {
			" - Exporting..."
		}
		else if self.importing_database {
			" - Importing..."
		}
		else if let Some(last_sync_time) = &self.last_sync_time {
			if Instant::now().duration_since(*last_sync_time) <= Duration::from_millis(500) {
				" - Synced"
			}
			else {
				""
			}
		}
		else if self.content_page.project_page.as_ref()
			.map(|project_page| project_page.importing_source_code_todos)
			.unwrap_or(false)
		{
			" - Importing Todos..."
		}
		else {
			""
		};

		let synchronized_status = if !self.syncing_database && !self.syncing_database_from_server {
			if let Some(last_sync_time) = self.last_sync_time {
				if let Some(database) = &self.database {
					if let Ok(last_database_save_duration) = database.last_saved_time.elapsed() {
						if last_database_save_duration < last_sync_time.elapsed() {
							" *"
						}
						else {
							""
						}
					}
					else {
						""
					}
				}
				else {
					""
				}
			}
			else {
				" *"
			}
		}
		else {
			""
		};

		format!("Project Tracker{progress_str}{synchronized_status}")
	}

	pub fn subscription(&self) -> Subscription<Message> {
		Subscription::batch([
			keyboard::on_key_press(|key, modifiers| match key.as_ref() {
				keyboard::Key::Character("b") if modifiers.command() => {
					Some(Message::ToggleSidebar)
				}
				keyboard::Key::Character("h") if modifiers.command() => {
					Some(ContentPageMessage::OpenOverview.into())
				}
				keyboard::Key::Character("s") if modifiers.command() => {
					Some(Message::SyncDatabase)
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
			iced::event::listen_with(move |event, status, _id| match event {
				Event::Window(window::Event::CloseRequested)
					if matches!(status, Status::Ignored) =>
				{
					Some(Message::TryClosing)
				},
				Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
					Some(Message::LeftClickReleased)
				},
				_ => None,
			}),
			self.sidebar_page
				.subscription()
				.map(Message::SidebarPageMessage),
			self.sidebar_animation
				.subscription()
				.map(|_| Message::AnimateSidebar),
			self.content_page.subscription().map(Message::ContentPageMessage),
			self.settings_modal
				.subscription()
				.map(Message::SettingsModalMessage),
			self.create_task_modal.subscription(),
			time::every(Duration::from_secs(1)).map(|_| Message::SaveChangedFiles),
			system_theme_subscription(),
		])
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		let mut task = match message {
			Message::TryClosing => {
				if self.syncing_database ||
					self.exporting_database ||
					self.importing_database ||
					self.syncing_database_from_server
				{
					let waiting_reason = if self.syncing_database {
						"Synchronizing database with filepath"
					}
					else if self.exporting_database {
						"Exporting database"
					}
					else if self.importing_database {
						"Importing database"
					}
					else if self.syncing_database_from_server {
						"Synchronizing database with server"
					}
					else {
						unreachable!()
					};
					self.wait_closing_modal = WaitClosingModal::Opened { waiting_reason	};
					Task::none()
				}
				else {
					Task::batch([
						self.update(Message::SaveDatabase),
						self.update(PreferenceMessage::Save.into()),
						window::get_latest().and_then(window::close),
					])
				}
			},
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
				if self.content_page.is_project_page_opened() {
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
			Message::WaitClosingModalMessage(message) => {
				self.wait_closing_modal.update(message)
					.map(Message::WaitClosingModalMessage)
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
			Message::SyncDatabase => {
				if let Some(preferences) = &self.preferences {
					if let Some(synchronization_settings) = preferences.synchronization() {
						return match synchronization_settings {
							SynchronizationSetting::Server(_) => self.update(Message::SyncDatabaseFromServer),
							SynchronizationSetting::Filepath(filepath) => match filepath {
								Some(filepath) => self.update(Message::SyncDatabaseFilepath(filepath.clone())),
								None => Task::none()
							},
						};
					}
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
				Task::perform(Database::load_from(filepath), |result| Message::DatabaseImported(result.map_err(Arc::new)))
			}
			Message::DatabaseImported(result) => self.update(Message::LoadedDatabase(result))
				.chain(self.update(Message::SaveDatabase)),
			Message::SyncDatabaseFilepath(filepath) => {
				self.syncing_database = true;
				Task::perform(
					Database::sync(filepath.clone()),
					move |result| match result {
						SyncDatabaseResult::InvalidSynchronizationFilepath => {
							Message::SyncDatabaseFilepathFailed(format!(
								"Failed to open synchronization file in\n\"{}\"",
								filepath.display()
							))
						}
						SyncDatabaseResult::Upload => {
							Message::SyncDatabaseFilepathUpload(filepath.clone())
						}
						SyncDatabaseResult::Download => Message::SyncDatabaseFilepathDownload(filepath.clone()),
					},
				)
			}
			Message::SyncDatabaseFilepathUpload(filepath) => {
				if let Some(database) = &self.database {
					Task::perform(Database::save_to(filepath, database.to_json()), |_| {
						Message::SyncDatabaseFilepathUploaded
					})
				} else {
					Task::none()
				}
			}
			Message::SyncDatabaseFilepathUploaded => {
				self.syncing_database = false;
				self.last_sync_time = Some(Instant::now());
				Task::none()
			}
			Message::SyncDatabaseFilepathDownload(filepath) => {
				Task::perform(Database::load_from(filepath), |result| Message::SyncDatabaseFilepathDownloaded(result.map_err(Arc::new)))
			},
			Message::SyncDatabaseFilepathDownloaded(result) => match result {
				Ok(database) => {
					self.last_sync_time = Some(Instant::now());
					self.update(Message::DatabaseImported(Ok(database)))
				},
				Err(e) => self.update(Message::DatabaseImported(Err(e))),
			},
			Message::SyncDatabaseFilepathFailed(error_msg) => {
				self.syncing_database = false;
				self.show_error_msg(error_msg)
			}
			Message::LoadedDatabase(load_database_result) => {
				self.loading_database = false;
				self.importing_database = false;
				self.syncing_database = false;
				self.syncing_database_from_server = false;

				match load_database_result {
					Ok(database) => {
						self.database = Some(database);
						self.content_page.restore_from_serialized(&self.database, &mut self.preferences);
						Task::none()
					},
					Err(error) => match error.as_ref() {
						LoadDatabaseError::FailedToOpenFile { .. } => {
							if self.database.is_some() {
								self.show_error_msg(format!("{error}"))
							}
							else {
								Task::none()
							}
						},
						LoadDatabaseError::FailedToParse{ filepath, .. } => {
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
								self.show_error_msg(format!("{error}"))
							])
						},
					},
				}
			}
			Message::LoadedPreferences(load_preferences_result) => {
				match load_preferences_result {
					Ok(preferences) => {
						self.preferences = Some(preferences);
						self.update(PreferenceMessage::Save.into())
					},
					Err(error) => match error.as_ref() {
						LoadPreferencesError::FailedToOpenFile{ .. } => {
							if self.preferences.is_none() {
								self.preferences = Some(Preferences::default());
								self.update(PreferenceMessage::Save.into())
							} else {
								self.show_error_msg(format!("{error}"))
							}
						}
						LoadPreferencesError::FailedToParse{ filepath, .. } => {
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
						},
					},
				}
			},
			Message::SyncDatabaseFromServer => {
				if let Some(database) = &self.database {
					if let Some(SynchronizationSetting::Server(server_config)) = self.preferences.synchronization().cloned() {
						self.syncing_database_from_server = true;

						let database_filepath = Database::get_filepath();

						match database_filepath.metadata() {
							Ok(metadata) => {
								return Task::perform(
									sync_database_from_server(
										server_config,
										get_last_modification_date_time(&metadata),
										database.clone()
									),
									|result| match result {
										Ok(sync_response) => match sync_response {
											SyncServerDatabaseResponse::DownloadedDatabase(database) => Message::DatabaseDownloadedFromServer(database),
											SyncServerDatabaseResponse::UploadedDatabase => Message::DatabaseUploadedToServer,
										},
										Err(e) => Message::ServerError(Arc::new(e)),
									}
								);
							},
							Err(e) => return self.show_error_msg(format!(
								"failed to get metadata of database file: {}, error: {e}",
								database_filepath.display()
							)),
						}
					}
				}
				Task::none()
			},
			Message::DatabaseDownloadedFromServer(database) => {
				self.last_sync_time = Some(Instant::now());
				self.update(Message::LoadedDatabase(Ok(database)))
			},
			Message::ServerError(e) => {
				self.syncing_database_from_server = false;
				self.update(ErrorMsgModalMessage::open(format!("{e}")))
			},
			Message::DatabaseUploadedToServer => {
				self.syncing_database_from_server = false;
				self.last_sync_time = Some(Instant::now());
				Task::none()
			},
			Message::DatabaseMessage(database_message) => {
				if let Some(database) = &mut self.database {
					database.update(database_message.clone());
					if let Some(overview_page) = &mut self.content_page.overview_page {
						overview_page.update(OverviewPageMessage::RefreshCachedTaskList, &self.database, &self.preferences);
					}

					let database = self.database.as_ref().unwrap();
					match &mut self.content_page.project_page {
						Some(project_page) if database.get_project(&project_page.project_id).is_none() => {
							self.update(ContentPageMessage::OpenOverview.into())
						}
						Some(project_page) => {
							project_page.generate_cached_task_list(database, &self.preferences);
							Task::none()
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
			Message::SwitchToLowerProject => {
				if let Some(database) = &self.database {
					if let Some(project_page) = &self.content_page.project_page {
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
					if let Some(project_page) = &self.content_page.project_page {
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
							self.update(ContentPageMessage::OpenProjectPage(*project_id).into())
						} else {
							Task::none()
						},
						sidebar_snap_command,
					]);
				}
				Task::none()
			}
			Message::DeleteSelectedProject => {
				if let Some(project_page) = &self.content_page.project_page {
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
			Message::PressTask{ project_id, task_id } => {
				self.pressed_task = Some((project_id, task_id));
				Task::none()
			},
			Message::DragTask {
				project_id,
				task_id,
				task_is_todo,
				point,
				rect,
			} => {
				let is_theme_dark = self.is_theme_dark();

				let filtering_tags = self.content_page.project_page.as_ref()
					.map(|project_page| !project_page.filter_task_tags.is_empty())
					.unwrap_or(false);

				self.dragged_task = Some(task_id);
				if let Some(start_dragging_point) = self.start_dragging_point {
					if self.just_minimal_dragging {
						self.just_minimal_dragging =
							start_dragging_point.distance(point) < MINIMAL_DRAG_DISTANCE;
					}
				} else {
					self.start_dragging_point = Some(point);
					self.just_minimal_dragging = true;
				}

				let action = self.sidebar_page.update(
					SidebarPageMessage::DragTask {
						project_id,
						task_id,
						task_is_todo,
						filtering_tags,
						point,
						rect,
					},
					&self.database,
					is_theme_dark,
				);

				self.perform_sidebar_action(action)
			}
			Message::CancelDragTask => {
				let is_theme_dark = self.is_theme_dark();

				self.dragged_task = None;
				self.start_dragging_point = None;
				self.just_minimal_dragging = true;

				let action = self.sidebar_page.update(
					SidebarPageMessage::CancelDragTask,
					&self.database,
					is_theme_dark,
				);

				self.perform_sidebar_action(action)
			},
			Message::LeftClickReleased => {
				let task = if self.just_minimal_dragging {
					if let Some((project_id, task_id)) = &self.pressed_task {
						self.update(TaskModalMessage::Open {
							project_id: *project_id,
							task_id: *task_id
						}.into())
					} else {
						Task::none()
					}
				} else {
					Task::none()
				};
				self.pressed_task = None;
				self.dragged_task = None;
				self.start_dragging_point = None;
				self.just_minimal_dragging = true;
				task
			}
			Message::ContentPageMessage(message) => {
				let action = self.content_page.update(message, &self.database, &mut self.preferences);
				self.perform_content_page_action(action)
			}
			Message::SidebarPageMessage(message) => {
				let is_theme_dark = self.is_theme_dark();
				let action = self.sidebar_page.update(
					message.clone(),
					&self.database,
					is_theme_dark,
				);
				self.perform_sidebar_action(action)
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

				let manage_task_tag_modal_action = self.manage_tags_modal.update(message, &self.database);

				Task::batch([
					self.perform_manage_task_tags_modal_action(manage_task_tag_modal_action),

					deleted_task_tag_id
						.and_then(|deleted_task_tag_id| {
							let action = self.content_page.project_page.as_mut().map(|project_page| {
								project_page.update(
									ProjectPageMessage::UnsetFilterTaskTag(deleted_task_tag_id),
									&self.database,
									&self.preferences
								)
							});
							action.map(|action| self.perform_content_page_action(action))
						})
						.unwrap_or(Task::none()),
				])
			},
			Message::CreateTaskModalMessage(message) => {
				let action = self.create_task_modal.update(message, &self.preferences);
				match action {
					CreateTaskModalAction::None => Task::none(),
					CreateTaskModalAction::Task(task) => task.map(Message::CreateTaskModalMessage),
					CreateTaskModalAction::DatabaseMessage(message) => Task::batch([
						self.update(message.into()),
						self.update(ProjectPageMessage::RefreshCachedTaskList.into()),
						self.update(OverviewPageMessage::RefreshCachedTaskList.into()),
					]),
				}
			},
			Message::OpenCreateTaskModal => if let Some(project_id) =
				self.content_page.project_page.as_ref().map(|project_page| project_page.project_id)
			{
				self.update(CreateTaskModalMessage::Open(project_id).into())
			}
			else {
				Task::none()
			},
			Message::TaskModalMessage(message) => match self.task_modal.update(message, &self.database) {
				TaskModalAction::None => Task::none(),
				TaskModalAction::Task(task) => task.map(Message::TaskModalMessage),
				TaskModalAction::DatabaseMessage(message) => self.update(message.into()),
			},
		};

		if matches!(self.wait_closing_modal, WaitClosingModal::Opened { .. }) &&
			matches!(self.error_msg_modal, ErrorMsgModal::Closed) &&
			!self.syncing_database &&
			!self.exporting_database &&
			!self.importing_database &&
			!self.syncing_database_from_server
		{
			self.wait_closing_modal = WaitClosingModal::Closed;
			task = Task::batch([
				task,
				self.update(Message::TryClosing)
			]);
		}

		if matches!(self.error_msg_modal, ErrorMsgModal::Open { .. }) {
			self.wait_closing_modal = WaitClosingModal::Closed;
		}

		task
	}

	fn perform_content_page_action(&mut self, action: ContentPageAction) -> Task<Message> {
		match action {
			ContentPageAction::None => Task::none(),
			ContentPageAction::Actions(actions) => Task::batch(
				actions.into_iter().map(|action| self.perform_content_page_action(action))
			),
			ContentPageAction::Task(task) => task,
			ContentPageAction::DatabaseMessage(message) => self.update(message.into()),
			ContentPageAction::OpenManageTaskTagsModal(project_id) => self.update(
				ManageTaskTagsModalMessage::Open { project_id }.into()
			),
			ContentPageAction::ConfirmDeleteProject { project_id, project_name } => self.update(ConfirmModalMessage::open(
				format!("Delete Project '{project_name}'?"),
				DatabaseMessage::DeleteProject(project_id),
			)),
			ContentPageAction::OpenTaskModal { project_id, task_id } => self.update(TaskModalMessage::Open {
				project_id,
				task_id
			}.into()),
			ContentPageAction::CloseTaskModal => self.update(TaskModalMessage::Close.into()),
			ContentPageAction::OpenStopwatch => self.update(ContentPageMessage::OpenStopwatch.into()),
		}
	}

	fn perform_sidebar_action(&mut self, action: SidebarPageAction) -> Task<Message> {
		match action {
			SidebarPageAction::None => Task::none(),
			SidebarPageAction::Actions(actions) => Task::batch(
				actions.into_iter().map(|action| self.perform_sidebar_action(action))
			),
			SidebarPageAction::Task(task) => task,
			SidebarPageAction::DatabaseMessage(message) => self.update(message.into()),
			SidebarPageAction::StopwatchPageMessage(message) => self.update(message.into()),
			SidebarPageAction::SelectProject(project_id) => self.update(ContentPageMessage::OpenProjectPage(project_id).into()),
		}
	}

	fn perform_preference_action(&mut self, action: PreferenceAction) -> Task<Message> {
		match action {
			PreferenceAction::None => Task::none(),
			PreferenceAction::Task(task) => task,
			PreferenceAction::RefreshCachedTaskList => {
				if let Some(project_page) = &mut self.content_page.project_page {
					if let Some(database) = &self.database {
						project_page.generate_cached_task_list(database, &self.preferences);
					}
				}
				Task::none()
			},
		}
	}

	fn perform_manage_task_tags_modal_action(&mut self, action: ManageTaskTagsModalAction) -> Task<Message> {
		match action {
			ManageTaskTagsModalAction::None => Task::none(),
			ManageTaskTagsModalAction::Task(task) => task,
			ManageTaskTagsModalAction::DatabaseMessage(message) => self.update(message.into()),
		}
	}

	fn modal<Message: 'static + Clone>(
		content: Option<Element<Message>>,
		on_close: Message,
	) -> Option<Element<Message>> {
		content.map(|content| {
			opaque(
				mouse_area(
					center(opaque(content))
						.style(modal_background_container_style)
				)
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

		let underlay: Element<Message> = if self.database.is_some() || self.loading_database {
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
						.style(sidebar_background_container_style)
						.into()
				} else {
					Space::new(Fill, Fill).into()
				};

				stack![
					sidebar,
					container(
						container(
							row![
								if show_sidebar || sidebar_animation_value.is_some() {
									Space::with_width(empty_toggle_sidebar_button_layout_width)
										.into()
								} else {
									toggle_sidebar_button(false)
								},
								arc_self.content_page.view(&arc_self),
							]
						)
						.style(default_background_container_style)
					)
					.width(Fill)
					.padding(Padding::default().left(size.width * sidebar_layout_percentage))
				]
				.into()
			})
			.into()
		}
		else {
			container(
				column![
					text("Create new database or import existing?").size(HEADING_TEXT_SIZE),
					row![
						create_empty_database_button(),
						import_database_button(self.importing_database),
					]
					.spacing(LARGE_SPACING_AMOUNT)
				]
				.spacing(LARGE_SPACING_AMOUNT)
				.align_x(Horizontal::Center)
			)
			.center(Fill)
			.into()
		};

		Stack::new()
			.push(underlay)
			.push_maybe(Self::modal(
				self.create_task_modal.view(&self.database, &self.preferences),
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
				self.wait_closing_modal.view(),
				WaitClosingModalMessage::Close
			)
			.map(|element| element.map(Message::WaitClosingModalMessage)))
			.push_maybe(Self::modal(
				self.error_msg_modal.view(),
				ErrorMsgModalMessage::Close.into(),
			))
			.into()
	}
}
