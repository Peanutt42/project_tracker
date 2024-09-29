use std::{path::PathBuf, time::{Duration, Instant}};
use iced::{clipboard, event::Status, keyboard, mouse, time, widget::{center, container, focus_next, focus_previous, mouse_area, opaque, row, Stack}, window, Color, Element, Event, Length::FillPortion, Padding, Subscription, Task, Theme};
use crate::{
	components::{invisible_toggle_sidebar_button, toggle_sidebar_button, vertical_seperator, ConfirmModal, ConfirmModalMessage, ErrorMsgModal, ErrorMsgModalMessage, ManageTaskTagsModal, ManageTaskTagsModalMessage, SettingsModal, SettingsModalMessage},
	core::{Database, DatabaseMessage, LoadDatabaseResult, LoadPreferencesResult, PreferenceMessage, Preferences, ProjectId, SerializedContentPage, SyncDatabaseResult},
	pages::{ProjectPage, ProjectPageMessage, SidebarPage, SidebarPageMessage, StopwatchPage, StopwatchPageMessage},
	styles::PADDING_AMOUNT,
	theme_mode::{get_theme, is_system_theme_dark, system_theme_subscription, ThemeMode},
};

pub struct ProjectTrackerApp {
	pub sidebar_page: SidebarPage,
	pub stopwatch_page: StopwatchPage,
	pub project_page: Option<ProjectPage>,
	pub database: Option<Database>,
	pub importing_database: bool,
	pub exporting_database: bool,
	pub syncing_database: bool,
	pub preferences: Option<Preferences>,
	pub confirm_modal: ConfirmModal,
	pub error_msg_modal: ErrorMsgModal,
	pub settings_modal: SettingsModal,
	pub manage_tags_modal: ManageTaskTagsModal,
	pub is_system_theme_dark: bool,
}

#[derive(Clone, Debug)]
pub enum UiMessage {
	CloseWindowRequested(window::Id),
	EscapePressed,
	EnterPressed,
	LeftClickReleased,
	CopyToClipboard(String),
	OpenUrl(String),
	FocusNext,
	FocusPrevious,
	SaveChangedFiles,
	OpenFolderLocation(PathBuf),
	SystemTheme { is_dark: bool },
	ConfirmModalMessage(ConfirmModalMessage),
	ConfirmModalConfirmed(Box<UiMessage>),
	ErrorMsgModalMessage(ErrorMsgModalMessage),
	SaveDatabase,
	DatabaseSaved(Instant), // begin_time since saving
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
	DatabaseMessage(DatabaseMessage),
	PreferenceMessage(PreferenceMessage),
	SelectProject(Option<ProjectId>),
	SwitchToUpperProject, // switches to upper project when using shortcuts
	SwitchToLowerProject, // switches to lower project when using shortcuts
	SwitchToProject{ order: usize }, // switches to project when using shortcuts
	DeleteSelectedProject,
	OpenStopwatch,
	StopwatchPageMessage(StopwatchPageMessage),
	ProjectPageMessage(ProjectPageMessage),
	SidebarPageMessage(SidebarPageMessage),
	SettingsModalMessage(SettingsModalMessage),
	ManageTaskTagsModalMessage(ManageTaskTagsModalMessage),
}

impl ProjectTrackerApp {
	fn show_error_msg(&mut self, error_msg: String) -> Task<UiMessage> {
		self.update(ErrorMsgModalMessage::open(error_msg))
	}

	pub fn is_theme_dark(&self) -> bool {
		if let Some(preferences) = &self.preferences {
			match preferences.theme_mode() {
				ThemeMode::System => self.is_system_theme_dark,
				ThemeMode::Dark => true,
				ThemeMode::Light => false,
			}
		}
		else {
			self.is_system_theme_dark
		}
	}

	pub fn get_theme(&self) -> &'static Theme {
		get_theme(self.is_theme_dark())
	}
}

impl ProjectTrackerApp {
	pub fn new() -> (Self, Task<UiMessage>) {
		(
			Self {
				sidebar_page: SidebarPage::new(),
				stopwatch_page: StopwatchPage::default(),
				project_page: None,
				database: None,
				importing_database: false,
				exporting_database: false,
				syncing_database: false,
				preferences: None,
				confirm_modal: ConfirmModal::Closed,
				error_msg_modal: ErrorMsgModal::Closed,
				settings_modal: SettingsModal::Closed,
				manage_tags_modal: ManageTaskTagsModal::Closed,
				is_system_theme_dark: is_system_theme_dark(),
			},
			Task::batch([
				Task::perform(Preferences::load(), UiMessage::LoadedPreferences),
				Task::perform(Database::load(), UiMessage::LoadedDatabase),
			])
		)
	}

	pub fn theme(&self) -> Theme {
		self.get_theme().clone()
	}

	pub fn subscription(&self) -> Subscription<UiMessage> {
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
				keyboard::Key::Character("h") if modifiers.command() => Some(UiMessage::OpenStopwatch),
				keyboard::Key::Character("r") if modifiers.command() => Some(ProjectPageMessage::EditProjectName.into()),
				keyboard::Key::Character("f") if modifiers.command() => Some(ProjectPageMessage::OpenSearchTasks.into()),
				keyboard::Key::Character(",") if modifiers.command() => Some(SettingsModalMessage::Open.into()),
				keyboard::Key::Named(keyboard::key::Named::Escape) => Some(UiMessage::EscapePressed),
				keyboard::Key::Named(keyboard::key::Named::Enter) => Some(UiMessage::EnterPressed),
				keyboard::Key::Named(keyboard::key::Named::Delete) if modifiers.command() => Some(UiMessage::DeleteSelectedProject),
				keyboard::Key::Named(keyboard::key::Named::Space) => Some(StopwatchPageMessage::Toggle.into()),
				keyboard::Key::Named(keyboard::key::Named::Tab) => Some(
					if modifiers.command() {
						if modifiers.shift() {
							UiMessage::SwitchToUpperProject
						}
						else {
							UiMessage::SwitchToLowerProject
						}
					}
					else if modifiers.shift() {
						UiMessage::FocusPrevious
					}
					else {
						UiMessage::FocusNext
					}
				),
				_ => None,
			}),

			iced::event::listen_with(move |event, status, id| {
				match event {
					Event::Window(window::Event::CloseRequested) if matches!(status, Status::Ignored) => Some(UiMessage::CloseWindowRequested(id)),
					Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => Some(UiMessage::LeftClickReleased),
					_ => None,
				}
			}),

			self.stopwatch_page.subscription(self.project_page.is_none()),

			if let Some(project_page) = &self.project_page {
				project_page.subscription()
			}
			else {
				Subscription::none()
			},

			time::every(Duration::from_secs(1))
				.map(|_| UiMessage::SaveChangedFiles),

			system_theme_subscription(),
		])
	}

	pub fn update(&mut self, message: UiMessage) -> Task<UiMessage> {
		match message {
			UiMessage::CloseWindowRequested(id) => {
				Task::batch([
					self.update(UiMessage::SaveDatabase),
					self.update(PreferenceMessage::Save.into()),
					window::close(id),
				])
			},
			UiMessage::EscapePressed => {
				if matches!(self.error_msg_modal, ErrorMsgModal::Open { .. }) {
					return self.update(ErrorMsgModalMessage::Close.into());
				}
				if matches!(self.confirm_modal, ConfirmModal::Opened { .. }) {
					return self.update(ConfirmModalMessage::Close.into());
				}
				if matches!(self.settings_modal, SettingsModal::Opened{ .. }) {
					return self.update(SettingsModalMessage::Close.into());
				}
				if matches!(self.manage_tags_modal, ManageTaskTagsModal::Opened { .. }) {
					return self.update(ManageTaskTagsModalMessage::Close.into());
				}
				if self.project_page.is_some() {
					self.update(ProjectPageMessage::HideColorPicker.into())
				}
				else {
					self.update(StopwatchPageMessage::Stop.into())
				}
			},
			UiMessage::EnterPressed => {
				self.error_msg_modal = ErrorMsgModal::Closed;

				match &self.confirm_modal {
					ConfirmModal::Opened{ on_confirmed, .. } => {
						self.update(UiMessage::ConfirmModalConfirmed(Box::new(on_confirmed.clone())))
					},
					ConfirmModal::Closed => Task::none()
				}
			},
			UiMessage::LeftClickReleased => {
				let select_project_command = self.sidebar_page
					.should_select_project()
        			.map(|project_id| self.update(UiMessage::SelectProject(Some(project_id))))
               		.unwrap_or(Task::none());

				Task::batch([
					self.update(ProjectPageMessage::LeftClickReleased.into()),
					select_project_command,
				])
			},
			UiMessage::CopyToClipboard(copied_text) => clipboard::write(copied_text),
			UiMessage::OpenUrl(url) => { let _ = webbrowser::open(url.as_str()); Task::none() },
			UiMessage::FocusNext => focus_next(),
			UiMessage::FocusPrevious => focus_previous(),
			UiMessage::SaveChangedFiles => {
				let mut commands = Vec::new();
				if let Some(database) = &mut self.database {
					if database.has_unsaved_changes() {
						commands.push(self.update(UiMessage::SaveDatabase));
					}
				}
				if let Some(preferences) = &mut self.preferences {
					if preferences.has_unsaved_changes() {
						commands.push(preferences.update(PreferenceMessage::Save));
					}
				}
				Task::batch(commands)
			},
			UiMessage::OpenFolderLocation(filepath) => {
				let _ = open::that(filepath);
				Task::none()
			},
			UiMessage::SystemTheme{ is_dark } => { self.is_system_theme_dark = is_dark; Task::none() },
			UiMessage::ConfirmModalConfirmed(message) => {
				Task::batch([
					self.update(*message),
					self.update(ConfirmModalMessage::Close.into()),
				])
			},
			UiMessage::ConfirmModalMessage(message) => {
				self.confirm_modal.update(message);
				Task::none()
			},
			UiMessage::ErrorMsgModalMessage(message) => {
				self.error_msg_modal.update(message);
				Task::none()
			},
			UiMessage::SaveDatabase => if let Some(database) = &self.database {
				Task::perform(
					Database::save(database.to_json()),
					|result| {
						match result {
							Ok(begin_time) => UiMessage::DatabaseSaved(begin_time),
							Err(error_msg) => ErrorMsgModalMessage::open(error_msg),
						}
					}
				)
			}
			else {
				Task::none()
			},
			UiMessage::DatabaseSaved(saved_time) => {
				if let Some(database) = &mut self.database {
					database.last_saved_time = saved_time;
				}
				Task::none()
			},
			UiMessage::ExportDatabaseDialog => Task::perform(
				Database::export_file_dialog(),
				|filepath| {
					match filepath {
						Some(filepath) => UiMessage::ExportDatabase(filepath),
						None => UiMessage::ExportDatabaseDialogCanceled,
					}
				}
			),
			UiMessage::ExportDatabaseDialogCanceled => { self.exporting_database = false; Task::none() },
			UiMessage::ExportDatabase(filepath) => {
				if let Some(database) = &self.database {
					self.exporting_database = true;
					Task::perform(
						Database::save_to(filepath, database.to_json()),
						|result| match result {
							Ok(_) => UiMessage::DatabaseExported,
							Err(e) => UiMessage::ExportDatabaseFailed(e),
						}
					)
				}
				else {
					Task::none()
				}
			},
			UiMessage::ExportDatabaseFailed(error_msg) => { self.exporting_database = false; self.show_error_msg(error_msg) }
			UiMessage::DatabaseExported => { self.exporting_database = false; Task::none() },
			UiMessage::ImportDatabaseDialog => Task::perform(
				Database::import_file_dialog(),
				|filepath| {
					if let Some(filepath) = filepath {
						UiMessage::ImportDatabase(filepath)
					}
					else {
						UiMessage::ImportDatabaseDialogCanceled
					}
				}
			),
			UiMessage::ImportDatabaseDialogCanceled => { self.importing_database = false; Task::none() },
			UiMessage::ImportDatabase(filepath) => {
				self.importing_database = true;
				Task::perform(Database::load_from(filepath), UiMessage::LoadedDatabase)
			},
			UiMessage::SyncDatabase(filepath) => {
				self.syncing_database = true;
				Task::perform(Database::sync(filepath.clone()), move |result| {
					match result {
						SyncDatabaseResult::InvalidSynchronizationFilepath => UiMessage::SyncDatabaseFailed(format!("Failed to open synchronization file in\n\"{}\"", filepath.display())),
						SyncDatabaseResult::Upload => UiMessage::SyncDatabaseUpload(filepath.clone()),
						SyncDatabaseResult::Download => UiMessage::ImportDatabase(filepath.clone()),
					}
				})
			},
			UiMessage::SyncDatabaseUpload(filepath) => {
				if let Some(database) = &self.database {
					Task::perform(
						Database::save_to(filepath, database.to_json()),
						|_| UiMessage::SyncDatabaseUploaded
					)
				}
				else {
					Task::none()
				}
			},
			UiMessage::SyncDatabaseUploaded => { self.syncing_database = false; Task::none() },
			UiMessage::SyncDatabaseFailed(error_msg) => { self.syncing_database = false; self.show_error_msg(error_msg) },
			UiMessage::LoadedDatabase(load_database_result) => {
				match load_database_result {
					LoadDatabaseResult::Ok(database) => {
						self.database = Some(database);
						self.importing_database = false;
						self.syncing_database = false;
						let task = if let Some(preferences) = &self.preferences {
							let stopwatch_progress_message: Option<UiMessage> = preferences
								.stopwatch_progress()
								.as_ref()
								.map(|progress|
									StopwatchPageMessage::StartupAgain {
										task: progress.task,
										elapsed_time: Duration::from_secs(progress.elapsed_time_seconds),
										paused: progress.paused,
										finished_notification_sent: progress.finished_notification_sent,
									}
									.into()
								);

							let selected_content_page = *preferences.selected_content_page();

							Task::batch([
								if let Some(stopwatch_progress_message) = stopwatch_progress_message {
									self.update(stopwatch_progress_message)
								}
								else {
									Task::none()
								},

								match selected_content_page {
									SerializedContentPage::Stopwatch => self.update(UiMessage::OpenStopwatch),
									SerializedContentPage::Project(project_id) => {
										match &self.project_page {
											Some(project_page) => self.update(UiMessage::SelectProject(Some(project_page.project_id))),
											None => self.update(UiMessage::SelectProject(Some(project_id))),
										}
									},
								},
							])
						}
						else {
							Task::none()
						};

						Task::batch([
							task,
							self.update(UiMessage::SaveDatabase),
						])
					},
					LoadDatabaseResult::FailedToOpenFile(filepath) => {
						self.importing_database = false;
						self.syncing_database = false;
						let command = if self.database.is_none() {
							self.database = Some(Database::default());
							self.update(UiMessage::SaveDatabase)
						}
						else {
							Task::none()
						};

						Task::batch([
							command,
							self.show_error_msg(format!("Failed to open database file:\n{}", filepath.display()))
						])
					},
					LoadDatabaseResult::FailedToParse(filepath) => {
						// saves the corrupted database, just so we don't lose the progress and can correct it afterwards
						let saved_corrupted_filepath = Database::get_filepath().parent().unwrap().join("corrupted - database.json");
						let _ = std::fs::copy(filepath.clone(), saved_corrupted_filepath.clone());
						if self.database.is_none() {
							self.database = Some(Database::default());
						}
						Task::batch([
							self.update(UiMessage::SaveDatabase),
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
							Task::none()
						}
					},
					LoadPreferencesResult::FailedToParse(filepath) => {
						// saves the corrupted preferences, just so we don't lose the progress and can correct it afterwards
						let saved_corrupted_filepath = Preferences::get_filepath().parent().unwrap().join("corrupted - preferences.json");
						let _ = std::fs::copy(filepath.clone(), saved_corrupted_filepath.clone());
						if self.preferences.is_none() {
							self.preferences = Some(Preferences::default());
						}
						Task::batch([
							self.update(PreferenceMessage::Save.into()),
							self.show_error_msg(format!("Parsing Error:\nFailed to load preferences in\n\"{}\"\n\nOld corrupted preferences saved into\n\"{}\"", filepath.display(), saved_corrupted_filepath.display()))
						])
					},
				}
			},
			UiMessage::DatabaseMessage(database_message) => {
				if let Some(database) = &mut self.database {
					let previous_project_progress = self.project_page.as_ref().and_then(|project_page| {
						database
							.get_project(&project_page.project_id)
							.map(|project| project.get_completion_percentage())
					});
					database.update(database_message.clone());
					match database_message {
						DatabaseMessage::DeleteProject(project_id) => {
							match &self.project_page {
								Some(project_page) if project_page.project_id == project_id => {
									self.update(UiMessage::OpenStopwatch)
								},
								_ => Task::none(),
							}
						},
						DatabaseMessage::ChangeProjectColor { .. } => {
							self.update(ProjectPageMessage::HideColorPicker.into())
						},
						DatabaseMessage::Clear |
						DatabaseMessage::CreateTask { .. } |
					 	DatabaseMessage::DeleteTask { .. } |
						DatabaseMessage::SetTaskDone { .. } |
						DatabaseMessage::SetTaskTodo { .. } |
						DatabaseMessage::DeleteDoneTasks(_) |
						DatabaseMessage::DeleteTaskTag { .. } |
						DatabaseMessage::MoveTask { .. } => {
							let new_project_progress = self.project_page.as_ref().and_then(|project_page| {
								database
									.get_project(&project_page.project_id)
									.map(|project| project.get_completion_percentage())
							});
							if let Some(previous_project_progress) = previous_project_progress {
								if let Some(new_project_progress) = new_project_progress {
									return self.update(ProjectPageMessage::StartProgressbarAnimation {
										start_percentage: previous_project_progress,
										target_percentage: new_project_progress
									}.into());
								}
							}
							Task::none()
						},
						_ => Task::none(),
					}
				}
				else {
					Task::none()
				}
			},
			UiMessage::PreferenceMessage(preference_message) => {
				if let Some(preferences) = &mut self.preferences {
					preferences.update(preference_message)
				}
				else {
					Task::none()
				}
			},
			UiMessage::OpenStopwatch => self.update(UiMessage::SelectProject(None)),
			UiMessage::StopwatchPageMessage(message) => {
				let messages = self.stopwatch_page.update(message, &self.database, self.project_page.is_none());
				if let Some(messages) = messages {
					let tasks: Vec<Task<UiMessage>> = messages.into_iter().map(|msg| self.update(msg)).collect();
					Task::batch(tasks)
				}
				else {
					Task::none()
				}
			},
			UiMessage::SelectProject(project_id) => {
				let open_project_info = if let Some(database) = &self.database {
					project_id.and_then(|project_id| {
						database.get_project(&project_id)
							.map(|project| (project_id, project))
					})
				}
				else {
					None
				};
				if let Some((project_id, project)) = open_project_info {
					self.project_page = Some(ProjectPage::new(project_id, project));
					self.update(PreferenceMessage::SetContentPage(SerializedContentPage::Project(project_id)).into())
				}
				else {
					self.project_page = None;
					self.update(PreferenceMessage::SetContentPage(SerializedContentPage::Stopwatch).into())
				}
			},
			UiMessage::SwitchToLowerProject => {
				if let Some(database) = &self.database {
					if let Some(project_page) = &self.project_page {
						if let Some(order) = database.projects().get_order(&project_page.project_id) {
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
				}
				self.update(UiMessage::SwitchToProject { order: 0 })
			},
			UiMessage::SwitchToUpperProject => {
				if let Some(database) = &self.database {
					if let Some(project_page) = &self.project_page {
						if let Some(order) = database.projects().get_order(&project_page.project_id) {
							let order_to_switch_to = if order > 0 {
								order - 1
							}
							else {
								database.projects().len() - 1 // switches to the last project
							};
							return self.update(UiMessage::SwitchToProject { order: order_to_switch_to });
						}
					}
					return self.update(UiMessage::SwitchToProject { order: database.projects().len() - 1 });
				}
				self.update(UiMessage::SwitchToProject { order: 0 })
			},
			UiMessage::SwitchToProject { order } => {
				if let Some(database) = &self.database {
					let switched_project_id = database.projects().get_key_at_order(order);
					let sidebar_snap_command = self.sidebar_page.snap_to_project(order, database);
					return Task::batch([
						if let Some(project_id) = switched_project_id {
							self.update(UiMessage::SelectProject(Some(*project_id)))
						}
						else {
							Task::none()
						},
						sidebar_snap_command,
					]);
				}
				Task::none()
			},
			UiMessage::DeleteSelectedProject => {
				if let Some(project_page) = &self.project_page {
					if let Some(database) = &self.database {
						if let Some(project) = database.get_project(&project_page.project_id) {
							return self.update(ConfirmModalMessage::open(format!("Delete Project '{}'?", project.name), DatabaseMessage::DeleteProject(project_page.project_id)));
						}
					}
				}
				Task::none()
			}
			UiMessage::ProjectPageMessage(message) => match &mut self.project_page {
				Some(project_page) => project_page.update(message, &mut self.database, &self.preferences),
				None => Task::none()
			},
			UiMessage::SidebarPageMessage(message) => {
				let is_theme_dark = self.is_theme_dark();
				let sidebar_command = self.sidebar_page.update(message.clone(), &mut self.database, &mut self.stopwatch_page, is_theme_dark);
				let command = match message {
					SidebarPageMessage::CreateNewProject(project_id) => self.update(UiMessage::SelectProject(Some(project_id))),
					SidebarPageMessage::DragTask { task_id, point, .. } => {
						match &mut self.project_page {
							Some(project_page) => project_page.update(ProjectPageMessage::DragTask{ task_id, point }, &mut self.database, &self.preferences),
							None => Task::none()
						}
					},
					SidebarPageMessage::CancelDragTask => {
						match &mut self.project_page {
							Some(project_page) => project_page.update(ProjectPageMessage::CancelDragTask, &mut self.database, &self.preferences),
							None => Task::none()
						}
					},
					_ => Task::none(),
				};
				Task::batch([
					sidebar_command,
					command
				])
			},
			UiMessage::SettingsModalMessage(message) => self.settings_modal.update(message, &mut self.preferences),
			UiMessage::ManageTaskTagsModalMessage(message) => {
				let deleted_task_tag_id = if let ManageTaskTagsModalMessage::DeleteTaskTag(task_tag_id) = &message {
					Some(*task_tag_id)
				}
				else {
					None
				};

				Task::batch([
					self.manage_tags_modal.update(message, &mut self.database),
					deleted_task_tag_id.and_then(|deleted_task_tag_id| {
						self.project_page
        					.as_mut()
							.map(|project_page| project_page.update(ProjectPageMessage::UnsetFilterTaskTag(deleted_task_tag_id), &mut self.database, &self.preferences))
					})
					.unwrap_or(Task::none())
				])
			},
		}
	}

	fn modal(content: Option<Element<UiMessage>>, on_close: UiMessage) -> Option<Element<UiMessage>> {
		content.map(|content| {
			opaque(
				mouse_area(center(opaque(content)).style(|_theme| {
					container::Style {
						background: Some(
							Color {
								a: 0.75,
								..Color::BLACK
							}.into()
						),
						..Default::default()
					}
				}))
				.on_press(on_close)
			)
		})
	}

	pub fn view(&self) -> Element<UiMessage> {
		let show_sidebar =
			if let Some(preferences) = &self.preferences {
				preferences.show_sidebar()
			}
			else {
				true
			};

		let content_view = self.project_page
			.as_ref()
			.map(|project_page| project_page.view(self))
			.unwrap_or(self.stopwatch_page.view(&self.database));

		let underlay: Element<UiMessage> = if show_sidebar {
			let sidebar_page_view = self.sidebar_page.view(self);

			row![
				container(sidebar_page_view).width(FillPortion(1)),
				vertical_seperator(),
				container(content_view).width(FillPortion(2))
			]
			.into()
			/*
			Split::new(
				self.sidebar_page.view(self),
				content_view,
				Some(sidebar_dividor_position),
				Axis::Vertical,
				|pos| PreferenceMessage::SetSidebarDividorPosition(pos).into()
			)
			.style(SplitStyles::custom(SplitStyle))
			.into()*/
		}
		else {
			row![
				container(toggle_sidebar_button())
					.padding(Padding { left: PADDING_AMOUNT, top: PADDING_AMOUNT, ..Padding::ZERO }),

				content_view,

				container(invisible_toggle_sidebar_button())
					.padding(Padding { right: PADDING_AMOUNT, top: PADDING_AMOUNT, ..Padding::ZERO }),
			]
			.into()
		};

		Stack::new()
			.push(underlay)
			.push_maybe(Self::modal(self.manage_tags_modal.view(self), ManageTaskTagsModalMessage::Close.into()))
			.push_maybe(Self::modal(self.settings_modal.view(self), SettingsModalMessage::Close.into()))
			.push_maybe(Self::modal(self.confirm_modal.view(), ConfirmModalMessage::Close.into()))
			.push_maybe(Self::modal(self.error_msg_modal.view(), ErrorMsgModalMessage::Close.into()))
			.into()
	}
}