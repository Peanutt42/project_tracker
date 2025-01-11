use crate::synchronization::{SynchronizationError, SynchronizationMessage};
use crate::{
	components::{
		create_empty_database_button, generate_task_description_markdown, import_database_button,
		markdown, retry_loading_database_button, settings_button, toggle_sidebar_button,
		ScalarAnimation, ICON_BUTTON_WIDTH,
	},
	core::{
		export_database_as_json_file_dialog, export_database_file_dialog, formatted_date_time,
		import_database_file_dialog, import_json_database_file_dialog, ProjectUiIdMap, TaskUiIdMap,
	},
	modals::{
		ConfirmModal, ConfirmModalMessage, CreateTaskModal, CreateTaskModalAction,
		CreateTaskModalMessage, ErrorMsgModal, ErrorMsgModalMessage, ManageTaskTagsModal,
		ManageTaskTagsModalAction, ManageTaskTagsModalMessage, SettingsModal, SettingsModalMessage,
		TaskModal, TaskModalAction, TaskModalMessage, WaitClosingModal, WaitClosingModalMessage,
	},
	pages::{
		ContentPage, ContentPageAction, ContentPageMessage, OverviewPageMessage, ProjectPage,
		ProjectPageMessage, SidebarPage, SidebarPageAction, SidebarPageMessage,
		StopwatchPageMessage,
	},
	styles::{
		default_background_container_style, modal_background_container_style,
		sidebar_background_container_style, HEADING_TEXT_SIZE, LARGE_SPACING_AMOUNT,
		MINIMAL_DRAG_DISTANCE, PADDING_AMOUNT,
	},
	synchronization::{BaseSynchronization, Synchronization},
	theme_mode::{get_theme, is_system_theme_dark, system_theme_subscription, ThemeMode},
};
use crate::{
	LoadPreferencesError, OptionalPreference, PreferenceAction, PreferenceMessage, Preferences,
};
use chrono::Utc;
use iced::{
	alignment::{Horizontal, Vertical},
	clipboard,
	event::Status,
	keyboard, mouse, time,
	widget::{
		center, column, container, mouse_area, opaque, responsive, row, stack, text, Space, Stack,
	},
	window, Element, Event,
	Length::Fill,
	Padding, Point, Rectangle, Subscription, Task, Theme,
};
use project_tracker_core::{
	Database, DatabaseMessage, LoadDatabaseError, ProjectId, SaveDatabaseError, TaskId,
};
use std::{
	collections::HashMap,
	path::PathBuf,
	rc::Rc,
	sync::Arc,
	time::{Duration, Instant, SystemTime},
};
use tracing::{error, info};

#[derive(Debug)]
pub enum DatabaseState {
	NotLoaded,
	Error,
	Loaded(Database),
}

impl DatabaseState {
	pub fn ok(&self) -> Option<&Database> {
		match self {
			Self::Loaded(database) => Some(database),
			_ => None,
		}
	}

	pub fn is_loaded(&self) -> bool {
		matches!(self, Self::Loaded(_))
	}

	pub fn error_loading(&self) -> bool {
		matches!(self, Self::Error)
	}
}

#[derive(Debug, Clone, Default)]
pub struct AppFlags {
	custom_database_filepath: Option<PathBuf>,
	custom_preferences_filepath: Option<PathBuf>,
}

impl AppFlags {
	pub fn custom(custom_database_filepath: PathBuf, custom_preferences_filepath: PathBuf) -> Self {
		Self {
			custom_database_filepath: Some(custom_database_filepath),
			custom_preferences_filepath: Some(custom_preferences_filepath),
		}
	}

	pub fn get_database_filepath(&self) -> Option<PathBuf> {
		Database::get_filepath(self.custom_database_filepath.clone())
	}

	pub fn get_preferences_filepath(&self) -> Option<PathBuf> {
		Preferences::get_filepath(self.custom_preferences_filepath.clone())
	}
}

pub struct ProjectTrackerApp {
	pub flags: AppFlags,
	pub sidebar_page: SidebarPage,
	pub sidebar_animation: ScalarAnimation,
	pub content_page: ContentPage,
	pub database: DatabaseState,
	pub project_ui_id_map: ProjectUiIdMap,
	pub task_ui_id_map: TaskUiIdMap,
	pub task_description_markdown_items: HashMap<TaskId, Vec<markdown::Item>>,
	pub loading_database: bool,
	pub importing_database: bool,
	pub exporting_database: bool,
	pub last_sync_start_time: Option<Instant>,
	pub last_sync_finish_time: Option<Instant>,
	pub synchronization: Option<Synchronization>,
	pub preferences: Option<Preferences>,
	pub confirm_modal: Option<ConfirmModal>,
	pub error_msg_modal: ErrorMsgModal,
	pub wait_closing_modal: WaitClosingModal,
	pub settings_modal: SettingsModal,
	pub manage_tags_modal: Option<ManageTaskTagsModal>,
	pub create_task_modal: Option<CreateTaskModal>,
	pub task_modal: Option<TaskModal>,
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
	OpenInCodeEditor(String), // file_location
	SaveChangedFiles,
	SyncIfChanged,
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
	ExportDatabaseAsJson(PathBuf),
	ExportDatabaseDialog,
	ExportDatabaseAsJsonDialog,
	ExportDatabaseFailed(Arc<SaveDatabaseError>),
	ExportDatabaseDialogCanceled,
	DatabaseExported,
	ImportDatabase(PathBuf),
	ImportJsonDatabase(PathBuf),
	DatabaseImported(Result<Database, Arc<LoadDatabaseError>>),
	ImportDatabaseDialog,
	ImportJsonDatabaseDialog,
	ImportDatabaseDialogCanceled,
	RequestAdminInfos,
	SyncDatabase,
	SyncedDatabase(Result<(), Arc<SynchronizationError>>),
	SynchronizationMessage(SynchronizationMessage),
	LoadDatabase,
	LoadedDatabase(Result<Database, Arc<LoadDatabaseError>>),
	SavePreferences,
	LoadedPreferences(Result<Preferences, Arc<LoadPreferencesError>>),
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
		task_id: TaskId,
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
	OpenCreateTaskModal(ProjectId),
	OpenCreateTaskModalCurrent,
	CloseCreateTaskModal,
	CreateTaskModalMessage(CreateTaskModalMessage),
	OpenTaskModal {
		project_id: ProjectId,
		task_id: TaskId,
	},
	TaskModalMessage(TaskModalMessage),
	CloseTaskModal,
	ManageTaskTagsModalMessage(ManageTaskTagsModalMessage),
	OpenManageTaskTagsModal(ProjectId),
	CloseManageTaskTagsModal,
}

impl ProjectTrackerApp {
	fn show_error_msg(&mut self, error_msg: impl Into<String>) -> Task<Message> {
		self.update(ErrorMsgModalMessage::open(error_msg.into()))
	}

	fn show_error<E: std::error::Error>(&mut self, error: E) -> Task<Message> {
		self.update(ErrorMsgModalMessage::open_error(error))
	}

	fn has_unsynched_changes(&self) -> bool {
		match self.last_sync_start_time {
			Some(last_sync_time) => match &self.database {
				DatabaseState::Loaded(database) => {
					match (Utc::now() - database.last_changed_time()).abs().to_std() {
						Ok(last_database_save_duration) => {
							last_database_save_duration < last_sync_time.elapsed()
						}
						Err(_) => false,
					}
				}
				_ => false,
			},
			None => true,
		}
	}

	pub fn is_theme_dark(&self) -> bool {
		match &self.preferences {
			Some(preferences) => match preferences.theme_mode() {
				ThemeMode::System => self.is_system_theme_dark,
				ThemeMode::Dark => true,
				ThemeMode::Light => false,
			},
			None => self.is_system_theme_dark,
		}
	}

	pub fn get_theme(&self) -> &'static Theme {
		get_theme(self.is_theme_dark())
	}

	pub fn new(flags: AppFlags) -> (Self, Task<Message>) {
		(
			Self {
				flags: flags.clone(),
				sidebar_page: SidebarPage::new(),
				sidebar_animation: ScalarAnimation::Idle,
				content_page: ContentPage::new(None, &None),
				database: DatabaseState::NotLoaded,
				project_ui_id_map: ProjectUiIdMap::default(),
				task_ui_id_map: TaskUiIdMap::default(),
				task_description_markdown_items: HashMap::new(),
				loading_database: true,
				importing_database: false,
				exporting_database: false,
				last_sync_start_time: None,
				last_sync_finish_time: None,
				synchronization: None,
				preferences: None,
				confirm_modal: None,
				error_msg_modal: ErrorMsgModal::Closed,
				wait_closing_modal: WaitClosingModal::Closed,
				settings_modal: SettingsModal::Closed,
				manage_tags_modal: None,
				create_task_modal: None,
				task_modal: None,
				pressed_task: None,
				dragged_task: None,
				start_dragging_point: None,
				just_minimal_dragging: true,
				is_system_theme_dark: is_system_theme_dark(),
			},
			Task::batch([
				match flags.get_preferences_filepath() {
					Some(preferences_filepath) => {
						Task::perform(Preferences::load(preferences_filepath), |result| {
							Message::LoadedPreferences(result.map_err(Arc::new))
						})
					}
					None => Task::done(ErrorMsgModalMessage::open(
						"failed to get preferences filepath".to_string(),
					)),
				},
				Task::done(Message::LoadDatabase),
			]),
		)
	}

	pub fn theme(&self) -> Theme {
		self.get_theme().clone()
	}

	pub fn is_syncing(&self) -> bool {
		match (&self.last_sync_start_time, &self.last_sync_finish_time) {
			(Some(last_sync_start_time), Some(last_sync_finish_time)) => {
				*last_sync_start_time > *last_sync_finish_time
			}
			(Some(_last_sync_start_time), None) => true,
			(None, Some(_last_sync_finish_time)) => false,
			(None, None) => false,
		}
	}

	pub fn title(&self) -> String {
		let mut title = "Project Tracker".to_string();

		if self.exporting_database {
			title += " - Exporting...";
		}
		if self.importing_database {
			title += " - Importing...";
		}
		if let Some(last_finish_time) = &self.last_sync_finish_time {
			if Instant::now().duration_since(*last_finish_time) <= Duration::from_millis(250) {
				title += " - Synced";
			} else if let Some(last_start_time) = &self.last_sync_start_time {
				if *last_start_time > *last_finish_time {
					title += " - Syncing...";
				}
			}
		} else if self.last_sync_start_time.is_some() {
			title += " - Syncing...";
		}
		if self
			.content_page
			.project_page
			.as_ref()
			.map(|project_page| project_page.importing_source_code_todos)
			.unwrap_or(false)
		{
			title += " - Importing Todos...";
		}

		title
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
				keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Message::EscapePressed),
				keyboard::Key::Named(keyboard::key::Named::Enter) => Some(Message::EnterPressed),
				keyboard::Key::Named(keyboard::key::Named::Delete) if modifiers.command() => {
					Some(Message::DeleteSelectedProject)
				}
				keyboard::Key::Named(keyboard::key::Named::Tab) if modifiers.command() => {
					Some(if modifiers.shift() {
						Message::SwitchToUpperProject
					} else {
						Message::SwitchToLowerProject
					})
				}
				keyboard::Key::Character("n") if modifiers.command() && !modifiers.shift() => {
					Some(Message::OpenCreateTaskModalCurrent)
				}
				_ => None,
			}),
			iced::event::listen_with(move |event, status, _id| match event {
				Event::Window(window::Event::CloseRequested)
					if matches!(status, Status::Ignored) =>
				{
					Some(Message::TryClosing)
				}
				Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
					Some(Message::LeftClickReleased)
				}
				_ => None,
			}),
			self.sidebar_page
				.subscription()
				.map(Message::SidebarPageMessage),
			self.sidebar_animation
				.subscription()
				.map(|_| Message::AnimateSidebar),
			self.content_page
				.subscription()
				.map(Message::ContentPageMessage),
			self.settings_modal.subscription(),
			self.synchronization
				.as_ref()
				.map(Synchronization::subscription)
				.unwrap_or(Subscription::none()),
			time::every(Duration::from_secs(1)).map(|_| Message::SaveChangedFiles),
			time::every(Duration::from_secs(1)).map(|_| Message::SyncIfChanged),
			system_theme_subscription(),
		])
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		let mut task = match message {
			Message::TryClosing => {
				if self.exporting_database || self.importing_database {
					let waiting_reason = if self.exporting_database {
						"Exporting database"
					} else if self.importing_database {
						"Importing database"
					} else {
						"UNREACHABLE"
					};
					self.wait_closing_modal = WaitClosingModal::Opened { waiting_reason };
					Task::none()
				} else {
					match self.flags.get_preferences_filepath() {
						Some(preferences_filepath) => self
							.update(StopwatchPageMessage::SaveTaskTimeSpendBeforeClosing.into())
							.chain(self.update(Message::SaveDatabase))
							.chain(
								self.update(PreferenceMessage::Save(preferences_filepath).into()),
							)
							.chain(window::get_latest().and_then(window::close)),
						None => self.show_error_msg("failed to get preferences filepath!"),
					}
				}
			}
			Message::EscapePressed => {
				if matches!(self.error_msg_modal, ErrorMsgModal::Open { .. }) {
					return self.update(ErrorMsgModalMessage::Close.into());
				}
				if self.confirm_modal.is_some() {
					return self.update(ConfirmModalMessage::Close.into());
				}
				if matches!(self.settings_modal, SettingsModal::Opened { .. }) {
					return self.update(SettingsModalMessage::Close.into());
				}
				if self.manage_tags_modal.is_some() {
					return self.update(Message::CloseManageTaskTagsModal);
				}
				if self.create_task_modal.is_some() {
					return self.update(Message::CloseCreateTaskModal);
				}
				if self.task_modal.is_some() {
					return self.update(Message::CloseTaskModal);
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
						Some(confirm_modal) => self.update(Message::ConfirmModalConfirmed(
							Box::new(confirm_modal.on_confirmed.clone()),
						)),
						None => Task::none(),
					}
				}
			}
			Message::CopyToClipboard(copied_text) => clipboard::write(copied_text),
			Message::OpenUrl(url) => {
				let _ = open::that_detached(url.as_str());
				Task::none()
			}
			Message::OpenInCodeEditor(file_location) => {
				if let Some(code_editor) = self.preferences.code_editor() {
					if let Err(e) = code_editor.generate_command(&file_location).spawn() {
						error!("failed to open source code todo in code editor\n{e}\ncode editor: {code_editor:?}, file_location: {file_location}");
					}
				}
				Task::none()
			}
			Message::SaveChangedFiles => {
				let mut tasks = Vec::new();
				if let DatabaseState::Loaded(database) = &mut self.database {
					if database.has_unsaved_changes() {
						tasks.push(self.update(Message::SaveDatabase));
					}
				}
				if let Some(preferences) = &mut self.preferences {
					if preferences.has_unsaved_changes() {
						tasks.push(self.update(Message::SavePreferences));
					}
				}
				Task::batch(tasks)
			}
			Message::SyncIfChanged => {
				if self.has_unsynched_changes() &&
					matches!(self.error_msg_modal, ErrorMsgModal::Closed) && // dont auto sync when an error happened
					self.confirm_modal.is_none() && // dont auto sync while user needs to confirm something
					matches!(self.settings_modal, SettingsModal::Closed)
				// or user is configuring the sync options
				{
					self.update(Message::SyncDatabase)
				} else {
					Task::none()
				}
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
				match message {
					ConfirmModalMessage::Open {
						title,
						on_confirmed,
						custom_ok_label,
						custom_cancel_label,
					} => {
						self.confirm_modal = Some(ConfirmModal::new(
							title,
							*on_confirmed,
							custom_ok_label,
							custom_cancel_label,
						))
					}
					ConfirmModalMessage::Close => self.confirm_modal = None,
				}
				Task::none()
			}
			Message::ErrorMsgModalMessage(message) => {
				self.error_msg_modal.update(message);
				Task::none()
			}
			Message::WaitClosingModalMessage(message) => self
				.wait_closing_modal
				.update(message)
				.map(Message::WaitClosingModalMessage),
			Message::SaveDatabase => match &self.database {
				DatabaseState::Loaded(database) => match self.flags.get_database_filepath() {
					Some(database_filepath) => {
						if let Some(database_binary) = database.clone().to_binary() {
							info!("saving database");
							Task::perform(
								Database::save(database_filepath, database_binary),
								|result| match result {
									Ok(begin_time) => Message::DatabaseSaved(begin_time),
									Err(error) => ErrorMsgModalMessage::open_error(error),
								},
							)
						} else {
							self.show_error_msg("failed to serialize database to save to file!")
						}
					}
					None => self.show_error_msg("failed to get database filepath!"),
				},
				_ => Task::none(),
			},
			Message::DatabaseSaved(saved_time) => {
				if let DatabaseState::Loaded(database) = &mut self.database {
					database.saved(saved_time);
				}
				Task::none()
			}
			Message::ExportDatabaseDialog => {
				Task::perform(export_database_file_dialog(), |filepath| match filepath {
					Some(filepath) => Message::ExportDatabase(filepath),
					None => Message::ExportDatabaseDialogCanceled,
				})
			}
			Message::ExportDatabaseAsJsonDialog => Task::perform(
				export_database_as_json_file_dialog(),
				|filepath| match filepath {
					Some(filepath) => Message::ExportDatabaseAsJson(filepath),
					None => Message::ExportDatabaseDialogCanceled,
				},
			),
			Message::ExportDatabaseDialogCanceled => {
				self.exporting_database = false;
				Task::none()
			}
			Message::ExportDatabase(filepath) => {
				if let DatabaseState::Loaded(database) = &self.database {
					match database.clone().to_binary() {
						Some(database_binary) => {
							self.exporting_database = true;
							Task::perform(Database::save(filepath, database_binary), |result| {
								match result {
									Ok(_) => Message::DatabaseExported,
									Err(e) => Message::ExportDatabaseFailed(Arc::new(e)),
								}
							})
						}
						None => self.show_error_msg(
							"failed to serialize database to binary to export to file!".to_string(),
						),
					}
				} else {
					Task::none()
				}
			}
			Message::ExportDatabaseAsJson(filepath) => {
				if let DatabaseState::Loaded(database) = &self.database {
					match database.clone().to_json() {
						Some(database_json) => {
							self.exporting_database = true;
							Task::perform(
								Database::export_as_json(filepath, database_json),
								|result| match result {
									Ok(_) => Message::DatabaseExported,
									Err(e) => Message::ExportDatabaseFailed(Arc::new(e)),
								},
							)
						}
						None => self.show_error_msg(
							"failed to serialize database to json to export to file!".to_string(),
						),
					}
				} else {
					Task::none()
				}
			}
			Message::ExportDatabaseFailed(error) => {
				self.exporting_database = false;
				self.show_error(error)
			}
			Message::DatabaseExported => {
				self.exporting_database = false;
				Task::none()
			}
			Message::ImportDatabaseDialog => {
				Task::perform(import_database_file_dialog(), |filepath| match filepath {
					Some(filepath) => Message::ImportDatabase(filepath),
					None => Message::ImportDatabaseDialogCanceled,
				})
			}
			Message::ImportJsonDatabaseDialog => Task::perform(
				import_json_database_file_dialog(),
				|filepath| match filepath {
					Some(filepath) => Message::ImportJsonDatabase(filepath),
					None => Message::ImportDatabaseDialogCanceled,
				},
			),
			Message::ImportDatabaseDialogCanceled => {
				self.importing_database = false;
				Task::none()
			}
			Message::ImportDatabase(filepath) => {
				self.importing_database = true;
				Task::perform(Database::load(filepath), |result| {
					Message::DatabaseImported(result.map_err(Arc::new))
				})
			}
			Message::ImportJsonDatabase(filepath) => {
				self.importing_database = true;
				Task::perform(Database::load_json(filepath), |result| {
					Message::DatabaseImported(result.map_err(Arc::new))
				})
			}
			Message::DatabaseImported(result) => self
				.update(Message::LoadedDatabase(result))
				.chain(self.update(Message::SaveDatabase)),
			Message::LoadDatabase => match self.flags.get_database_filepath() {
				Some(database_filepath) => {
					Task::perform(Database::load(database_filepath), |result| {
						Message::LoadedDatabase(result.map_err(Arc::new))
					})
				}
				None => self.show_error_msg("failed to get database filepath"),
			},
			Message::LoadedDatabase(load_database_result) => {
				self.loading_database = false;
				self.importing_database = false;

				match load_database_result {
					Ok(database) => {
						// generate task description markdown parsed items
						for project in database.projects().values() {
							for (task_id, task, _task_type) in project.iter() {
								self.task_description_markdown_items.insert(
									task_id,
									generate_task_description_markdown(&task.description),
								);
							}
						}
						self.database = DatabaseState::Loaded(database);
						let action = self
							.content_page
							.restore_from_serialized(self.database.ok(), &mut self.preferences);
						self.perform_content_page_action(action)
					}
					Err(error) => match error.as_ref() {
						LoadDatabaseError::FailedToOpenFile { .. } => {
							if self.database.is_loaded() {
								self.show_error(error)
							} else {
								Task::none() // no previous db --> fresh install --> no error
							}
						}
						LoadDatabaseError::FailedToParseBinary { filepath, .. }
						| LoadDatabaseError::FailedToParseJson { filepath, .. } => {
							// saves the corrupted database, just so we don't lose the progress and can correct it afterwards
							match self.flags.get_database_filepath() {
								Some(mut saved_corrupted_filepath) => {
									let formatted_date_time =
										formatted_date_time(self.preferences.date_formatting());
									saved_corrupted_filepath.set_file_name(format!(
										"corrupted_database_{formatted_date_time}.project_tracker"
									));

									if let Err(e) =
										std::fs::copy(filepath, saved_corrupted_filepath.clone())
									{
										error!(
											"failed to copy corrupted database file to {}: {e}",
											saved_corrupted_filepath.display()
										);
									}
								}
								None => error!("failed to save a copy of the corrupted database!"),
							}
							if !self.database.is_loaded() {
								self.database = DatabaseState::Error;
							}
							Task::batch([
								self.update(Message::SaveDatabase),
								self.show_error(error),
							])
						}
					},
				}
			}
			Message::RequestAdminInfos => {
				if let Some(Synchronization::ServerSynchronization(server_synchronization)) =
					&mut self.synchronization
				{
					server_synchronization.request_admin_infos();
				}
				Task::none()
			}
			Message::SyncDatabase => {
				if let DatabaseState::Loaded(database) = &self.database {
					if let Some(synchronization) = &mut self.synchronization {
						self.last_sync_start_time = Some(Instant::now());
						return synchronization.synchronize(database);
					}
				}
				Task::none()
			}
			Message::SyncedDatabase(synchronization_result) => {
				self.last_sync_finish_time = Some(Instant::now());
				self.sidebar_page.synchronization_error = match synchronization_result {
					Ok(()) => None,
					Err(e) => {
						error!("failed to synchronize: {e}");
						Some(e)
					}
				};
				Task::none()
			}
			Message::SynchronizationMessage(message) => {
				if let Some(synchronization) = &mut self.synchronization {
					synchronization.update(message)
				} else {
					Task::none()
				}
			}
			Message::SavePreferences => match &mut self.preferences {
				Some(preferences) => match self.flags.get_preferences_filepath() {
					Some(preferences_filepath) => {
						let action =
							preferences.update(PreferenceMessage::Save(preferences_filepath));
						self.perform_preference_action(action)
					}
					None => Task::done(ErrorMsgModalMessage::open(
						"failed to get preferences filepath".to_string(),
					)),
				},
				None => Task::none(),
			},
			Message::LoadedPreferences(load_preferences_result) => {
				match load_preferences_result {
					Ok(preferences) => {
						self.synchronization = preferences.synchronization().clone();
						self.preferences = Some(preferences);
						let content_page_action = self
							.content_page
							.restore_from_serialized(self.database.ok(), &mut self.preferences);
						Task::batch([
							self.update(Message::SavePreferences),
							self.perform_content_page_action(content_page_action),
						])
					}
					Err(error) => match error.as_ref() {
						LoadPreferencesError::FailedToOpenFile { .. } => {
							if self.preferences.is_none() {
								self.preferences = Some(Preferences::default());
								self.update(Message::SavePreferences)
							} else {
								self.show_error(error)
							}
						}
						LoadPreferencesError::FailedToParse { filepath, .. } => {
							// saves the corrupted preferences, just so we don't lose the progress and can correct it afterwards
							match self.flags.get_preferences_filepath() {
								Some(mut saved_corrupted_filepath) => {
									let formatted_date_time =
										formatted_date_time(self.preferences.date_formatting());
									saved_corrupted_filepath.set_file_name(format!(
										"corrupted_preferences_{formatted_date_time}.toml"
									));
									if let Err(e) = std::fs::copy(
										filepath.clone(),
										saved_corrupted_filepath.clone(),
									) {
										error!(
											"failed to copy corrupted preferences file to {}: {e}",
											saved_corrupted_filepath.display()
										);
									}
								}
								None => error!("failed to save copy of corrupted preferences!"),
							}

							if self.preferences.is_none() {
								self.preferences = Some(Preferences::default());
							}
							Task::batch([
								self.update(Message::SavePreferences),
								self.show_error(error),
							])
						}
					},
				}
			}
			Message::DatabaseMessage(database_message) => match &mut self.database {
				DatabaseState::Loaded(database) => {
					match &database_message {
						DatabaseMessage::CreateTask {
							task_id,
							task_description,
							..
						} => {
							self.task_description_markdown_items.insert(
								*task_id,
								generate_task_description_markdown(task_description),
							);
						}
						DatabaseMessage::ChangeTaskDescription {
							task_id,
							new_task_description,
							..
						} => {
							self.task_description_markdown_items.insert(
								*task_id,
								generate_task_description_markdown(new_task_description),
							);
						}
						DatabaseMessage::ImportProjects(projects) => {
							for project in projects.iter() {
								for (task_id, task, _task_type) in project.iter() {
									self.task_description_markdown_items.insert(
										task_id,
										generate_task_description_markdown(&task.description),
									);
								}
							}
						}
						DatabaseMessage::ImportSourceCodeTodos {
							source_code_todo_tasks,
							..
						} => {
							for (task_id, task) in source_code_todo_tasks.iter() {
								self.task_description_markdown_items.insert(
									task_id,
									generate_task_description_markdown(&task.description),
								);
							}
						}
						_ => {}
					}
					database.update(database_message);
					if let Some(overview_page) = &mut self.content_page.overview_page {
						overview_page.update(
							OverviewPageMessage::RefreshCachedTaskList,
							Some(database),
							&self.preferences,
						);
					}

					let should_save = database
						.last_saved_time()
						.elapsed()
						.map(|last_save_duration| last_save_duration >= Duration::from_secs(1))
						.unwrap_or(false);
					let should_sync = match self.last_sync_start_time {
						Some(last_sync_time) => {
							Instant::now().duration_since(last_sync_time) >= Duration::from_secs(1)
						}
						None => true,
					};

					let project_page_task = match &mut self.content_page.project_page {
						Some(project_page)
							if database.get_project(&project_page.project_id).is_none() =>
						{
							self.update(ContentPageMessage::OpenOverview.into())
						}
						Some(project_page) => {
							project_page.generate_cached_task_list(database, &self.preferences);
							Task::none()
						}
						_ => Task::none(),
					};

					let mut tasks = vec![project_page_task];
					if should_save {
						tasks.push(self.update(Message::SaveDatabase));
					}
					if should_sync {
						tasks.push(self.update(Message::SyncDatabase));
					}
					Task::batch(tasks)
				}
				_ => Task::none(),
			},
			Message::PreferenceMessage(preference_message) => {
				if let PreferenceMessage::SetSynchronization(new_synchronization) =
					preference_message.clone()
				{
					self.synchronization = new_synchronization;
				}
				let action = match &mut self.preferences {
					Some(preferences) => preferences.update(preference_message),
					None => PreferenceAction::None,
				};
				self.perform_preference_action(action)
			}
			Message::SwitchToLowerProject => {
				if let DatabaseState::Loaded(database) = &self.database {
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
				if let DatabaseState::Loaded(database) = &self.database {
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
				if let DatabaseState::Loaded(database) = &self.database {
					let switched_project_id = database.projects().get_key_at_order(order);
					let sidebar_snap_command = self.sidebar_page.snap_to_project(order, database);
					return Task::batch([
						match switched_project_id {
							Some(project_id) => {
								self.update(ContentPageMessage::OpenProjectPage(*project_id).into())
							}
							None => Task::none(),
						},
						sidebar_snap_command,
					]);
				}
				Task::none()
			}
			Message::DeleteSelectedProject => {
				if let Some(project_page) = &self.content_page.project_page {
					if let DatabaseState::Loaded(database) = &self.database {
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
			Message::PressTask {
				project_id,
				task_id,
			} => {
				self.pressed_task = Some((project_id, task_id));
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

				let filtering_tasks = self
					.content_page
					.project_page
					.as_ref()
					.map(ProjectPage::filtering_tasks)
					.unwrap_or(false);

				self.dragged_task = Some(task_id);
				match self.start_dragging_point {
					Some(start_dragging_point) => {
						if self.just_minimal_dragging {
							self.just_minimal_dragging =
								start_dragging_point.distance(point) < MINIMAL_DRAG_DISTANCE;
						}
					}
					None => {
						self.start_dragging_point = Some(point);
						self.just_minimal_dragging = true;
					}
				}

				let action = self.sidebar_page.update(
					SidebarPageMessage::DragTask {
						project_id,
						task_id,
						task_is_todo,
						filtering_tasks,
						point,
						rect,
					},
					self.database.ok(),
					&mut self.project_ui_id_map,
					&mut self.task_ui_id_map,
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
					self.database.ok(),
					&mut self.project_ui_id_map,
					&mut self.task_ui_id_map,
					is_theme_dark,
				);

				self.perform_sidebar_action(action)
			}
			Message::LeftClickReleased => {
				let task = if self.just_minimal_dragging {
					match &self.pressed_task {
						Some((project_id, task_id)) => {
							let (task_modal, task) = TaskModal::new(*project_id, *task_id);
							self.task_modal = Some(task_modal);
							task
						}
						None => Task::none(),
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
				let action =
					self.content_page
						.update(message, self.database.ok(), &mut self.preferences);
				self.perform_content_page_action(action)
			}
			Message::SidebarPageMessage(message) => {
				let is_theme_dark = self.is_theme_dark();
				let action = self.sidebar_page.update(
					message.clone(),
					self.database.ok(),
					&mut self.project_ui_id_map,
					&mut self.task_ui_id_map,
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
			}
			Message::ManageTaskTagsModalMessage(message) => match &mut self.manage_tags_modal {
				Some(manage_tags_modal) => {
					let deleted_task_tag_id = match &message {
						ManageTaskTagsModalMessage::DeleteTaskTag(task_tag_id) => {
							Some(*task_tag_id)
						}
						_ => None,
					};

					let manage_task_tag_modal_action =
						manage_tags_modal.update(message, self.database.ok());

					Task::batch([
						self.perform_manage_task_tags_modal_action(manage_task_tag_modal_action),
						deleted_task_tag_id
							.and_then(|deleted_task_tag_id| {
								let action =
									self.content_page.project_page.as_mut().map(|project_page| {
										project_page.update(
											ProjectPageMessage::UnsetFilterTaskTag(
												deleted_task_tag_id,
											),
											self.database.ok(),
											&self.preferences,
										)
									});
								action.map(|action| self.perform_content_page_action(action))
							})
							.unwrap_or(Task::none()),
					])
				}
				None => Task::none(),
			},
			Message::OpenManageTaskTagsModal(project_id) => {
				self.manage_tags_modal = Some(ManageTaskTagsModal::new(project_id));
				Task::none()
			}
			Message::CloseManageTaskTagsModal => {
				self.manage_tags_modal = None;
				Task::none()
			}
			Message::CreateTaskModalMessage(message) => match &mut self.create_task_modal {
				Some(create_task_modal) => match create_task_modal
					.update(message, &self.preferences)
				{
					CreateTaskModalAction::None => Task::none(),
					CreateTaskModalAction::Task(task) => task.map(Message::CreateTaskModalMessage),
					CreateTaskModalAction::CreateTask {
						project_id,
						task_id,
						task_name,
						task_description,
						task_tags,
						due_date,
						needed_time_minutes,
						time_spend,
						create_at_top,
					} => {
						self.create_task_modal = None;
						Task::batch([
							self.update(
								DatabaseMessage::CreateTask {
									project_id,
									task_id,
									task_name,
									task_description,
									task_tags,
									due_date,
									needed_time_minutes,
									time_spend,
									create_at_top,
								}
								.into(),
							),
							self.update(ProjectPageMessage::RefreshCachedTaskList.into()),
							self.update(OverviewPageMessage::RefreshCachedTaskList.into()),
						])
					}
				},
				None => Task::none(),
			},
			Message::OpenCreateTaskModalCurrent => match self
				.content_page
				.project_page
				.as_ref()
				.map(|project_page| project_page.project_id)
			{
				Some(project_id) => self.update(Message::OpenCreateTaskModal(project_id)),
				None => Task::none(),
			},
			Message::OpenCreateTaskModal(project_id) => {
				self.create_task_modal = Some(CreateTaskModal::new(project_id));
				Task::none()
			}
			Message::CloseCreateTaskModal => {
				self.create_task_modal = None;
				Task::none()
			}
			Message::TaskModalMessage(message) => match &mut self.task_modal {
				Some(task_modal) => match task_modal.update(message, self.database.ok()) {
					TaskModalAction::None => Task::none(),
					TaskModalAction::Task(task) => task.map(Message::TaskModalMessage),
					TaskModalAction::DatabaseMessage(message) => {
						if let DatabaseMessage::DeleteTask { .. } = &message {
							self.task_modal = None;
						}
						self.update(message.into())
					}
				},
				None => Task::none(),
			},
			Message::OpenTaskModal {
				project_id,
				task_id,
			} => {
				let (task_modal, task) = TaskModal::new(project_id, task_id);
				self.task_modal = Some(task_modal);
				task
			}
			Message::CloseTaskModal => {
				self.task_modal = None;
				Task::none()
			}
		};

		if matches!(self.wait_closing_modal, WaitClosingModal::Opened { .. })
			&& matches!(self.error_msg_modal, ErrorMsgModal::Closed)
			&& !self.exporting_database
			&& !self.importing_database
		{
			self.wait_closing_modal = WaitClosingModal::Closed;
			task = Task::batch([task, self.update(Message::TryClosing)]);
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
				actions
					.into_iter()
					.map(|action| self.perform_content_page_action(action)),
			),
			ContentPageAction::Task(task) => task,
			ContentPageAction::DatabaseMessage(message) => {
				if let DatabaseMessage::DeleteTask {
					project_id,
					task_id,
				} = &message
				{
					if let Some(task_modal) = &mut self.task_modal {
						if *project_id == task_modal.project_id && *task_id == task_modal.task_id {
							self.task_modal = None;
						}
					}
				}

				self.update(message.into())
			}
			ContentPageAction::OpenManageTaskTagsModal(project_id) => {
				self.update(Message::OpenManageTaskTagsModal(project_id))
			}
			ContentPageAction::ConfirmDeleteProject {
				project_id,
				project_name,
			} => self.update(ConfirmModalMessage::open(
				format!("Delete Project '{project_name}'?"),
				DatabaseMessage::DeleteProject(project_id),
			)),
			ContentPageAction::OpenTaskModal {
				project_id,
				task_id,
			} => self.update(Message::OpenTaskModal {
				project_id,
				task_id,
			}),
			ContentPageAction::CloseTaskModal => {
				self.task_modal = None;
				Task::none()
			}
			ContentPageAction::OpenStopwatch => {
				self.update(ContentPageMessage::OpenStopwatch.into())
			}
		}
	}

	fn perform_sidebar_action(&mut self, action: SidebarPageAction) -> Task<Message> {
		match action {
			SidebarPageAction::None => Task::none(),
			SidebarPageAction::Actions(actions) => Task::batch(
				actions
					.into_iter()
					.map(|action| self.perform_sidebar_action(action)),
			),
			SidebarPageAction::Task(task) => task,
			SidebarPageAction::DatabaseMessage(message) => self.update(message.into()),
			SidebarPageAction::StopwatchPageMessage(message) => self.update(message.into()),
			SidebarPageAction::SelectProject(project_id) => {
				self.update(ContentPageMessage::OpenProjectPage(project_id).into())
			}
		}
	}

	fn perform_preference_action(&mut self, action: PreferenceAction) -> Task<Message> {
		match action {
			PreferenceAction::None => Task::none(),
			PreferenceAction::Task(task) => task,
			PreferenceAction::PreferenceMessage(message) => self.update(message.into()),
			PreferenceAction::FailedToSerializePreferences(e) => {
				self.show_error_msg(format!("Failed to serialize preferences to toml: {e}"))
			}
			PreferenceAction::RefreshCachedTaskList => {
				if let Some(project_page) = &mut self.content_page.project_page {
					if let DatabaseState::Loaded(database) = &self.database {
						project_page.generate_cached_task_list(database, &self.preferences);
					}
				}
				Task::none()
			}
			PreferenceAction::RequestAdminInfos => self.update(Message::RequestAdminInfos),
		}
	}

	fn perform_manage_task_tags_modal_action(
		&mut self,
		action: ManageTaskTagsModalAction,
	) -> Task<Message> {
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
				mouse_area(center(opaque(content)).style(modal_background_container_style))
					.on_press(on_close),
			)
		})
	}

	pub fn view(&self) -> Element<Message> {
		let show_sidebar = match &self.preferences {
			Some(preferences) => preferences.show_sidebar(),
			None => true,
		};

		let sidebar_animation_value = self.sidebar_animation.get_value();
		// 0.0..1.0
		let sidebar_animation_percentage = sidebar_animation_value
			.map(|value| (1.0 - value / SidebarPage::SPLIT_LAYOUT_PERCENTAGE));

		let underlay: Element<Message> = if self.database.is_loaded() || self.loading_database {
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

				let sidebar: Element<Message> = if show_sidebar || sidebar_animation_value.is_some()
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
						container(row![
							if show_sidebar || sidebar_animation_value.is_some() {
								Space::with_width(empty_toggle_sidebar_button_layout_width).into()
							} else {
								toggle_sidebar_button(false)
							},
							arc_self.content_page.view(&arc_self),
						])
						.style(default_background_container_style)
					)
					.width(Fill)
					.padding(Padding::default().left(size.width * sidebar_layout_percentage))
				]
				.into()
			})
			.into()
		} else if self.database.error_loading() {
			stack![
				container(
					column![
						text("Failed loading previous database").size(HEADING_TEXT_SIZE),
						row![
							retry_loading_database_button(),
							create_empty_database_button(),
							import_database_button(self.importing_database),
						]
						.spacing(LARGE_SPACING_AMOUNT)
					]
					.spacing(LARGE_SPACING_AMOUNT * 2)
					.align_x(Horizontal::Center),
				)
				.center(Fill),
				container(settings_button())
					.center(Fill)
					.align_x(Horizontal::Left)
					.align_y(Vertical::Bottom)
					.padding(Padding::new(PADDING_AMOUNT))
			]
			.into()
		} else {
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
				.align_x(Horizontal::Center),
			)
			.center(Fill)
			.into()
		};

		Stack::new()
			.push(underlay)
			.push_maybe(Self::modal(
				self.create_task_modal.as_ref().map(|create_task_modal| {
					create_task_modal.view(self.database.ok(), &self.preferences)
				}),
				Message::CloseCreateTaskModal,
			))
			.push_maybe(Self::modal(
				self.task_modal
					.as_ref()
					.map(|task_modal| task_modal.view(self)),
				Message::CloseTaskModal,
			))
			.push_maybe(Self::modal(
				self.manage_tags_modal
					.as_ref()
					.map(|task_modal| task_modal.view(self)),
				Message::CloseManageTaskTagsModal,
			))
			.push_maybe(Self::modal(
				self.settings_modal.view(self),
				SettingsModalMessage::Close.into(),
			))
			.push_maybe(Self::modal(
				self.confirm_modal.as_ref().map(ConfirmModal::view),
				ConfirmModalMessage::Close.into(),
			))
			.push_maybe(
				Self::modal(
					self.wait_closing_modal.view(),
					WaitClosingModalMessage::Close,
				)
				.map(|element| element.map(Message::WaitClosingModalMessage)),
			)
			.push_maybe(Self::modal(
				self.error_msg_modal.view(),
				ErrorMsgModalMessage::Close.into(),
			))
			.into()
	}
}
