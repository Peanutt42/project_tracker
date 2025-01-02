use crate::icons::Bootstrap;
use crate::integrations::{CodeEditor, ServerConfig};
use crate::{
	components::{
		dangerous_button, date_formatting_button, file_location, horizontal_seperator_padded,
		theme_mode_button, HORIZONTAL_SCROLLABLE_PADDING,
	},
	modals::ErrorMsgModalMessage,
	project_tracker::Message,
	styles::SPACING_AMOUNT,
	theme_mode::ThemeMode,
};
use crate::{ProjectId, SerializableDate, TaskId};
use iced::widget::text;
use iced::{
	alignment::Horizontal,
	widget::{column, container, row, toggler, Row},
	Alignment, Element,
	Length::Fill,
	Task,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;

fn default_show_sidebar() -> bool {
	true
}
fn default_create_new_tasks_at_top() -> bool {
	true
}
fn default_sort_unspecified_tasks_at_bottom() -> bool {
	true
}
fn default_play_timer_notification_sound() -> bool {
	true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SynchronizationSetting {
	Filepath(Option<PathBuf>),
	Server(ServerConfig),
}

impl SynchronizationSetting {
	pub fn as_str(&self) -> &'static str {
		match self {
			SynchronizationSetting::Filepath(_) => "Filepath",
			SynchronizationSetting::Server(_) => "Server",
		}
	}

	pub fn is_same_type(&self, other: &Self) -> bool {
		match self {
			SynchronizationSetting::Filepath(_) => {
				matches!(other, SynchronizationSetting::Filepath(_))
			}
			SynchronizationSetting::Server(_) => {
				matches!(other, SynchronizationSetting::Server(_))
			}
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preferences {
	theme_mode: ThemeMode,

	date_formatting: DateFormatting,

	#[serde(default = "default_create_new_tasks_at_top")]
	create_new_tasks_at_top: bool,

	#[serde(default = "default_sort_unspecified_tasks_at_bottom")]
	sort_unspecified_tasks_at_bottom: bool,

	#[serde(default = "default_play_timer_notification_sound")]
	play_timer_notification_sound: bool,

	#[serde(default = "default_show_sidebar")]
	show_sidebar: bool,

	selected_content_page: SerializedContentPage,

	stopwatch_progress: Option<StopwatchProgress>,

	synchronization: Option<SynchronizationSetting>,

	code_editor: Option<CodeEditor>,

	#[serde(skip, default = "Instant::now")]
	last_changed_time: Instant,

	#[serde(skip, default = "Instant::now")]
	last_saved_time: Instant,
}

impl Default for Preferences {
	fn default() -> Self {
		Self {
			theme_mode: ThemeMode::default(),
			date_formatting: DateFormatting::default(),
			create_new_tasks_at_top: default_create_new_tasks_at_top(),
			sort_unspecified_tasks_at_bottom: default_sort_unspecified_tasks_at_bottom(),
			show_sidebar: default_show_sidebar(),
			play_timer_notification_sound: default_play_timer_notification_sound(),
			selected_content_page: SerializedContentPage::default(),
			stopwatch_progress: None,
			synchronization: None,
			code_editor: None,
			last_changed_time: Instant::now(),
			last_saved_time: Instant::now(),
		}
	}
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum StopwatchProgress {
	Task {
		project_id: ProjectId,
		task_id: TaskId,
		paused: bool,
		finished_notification_sent: bool,
	},
	TrackTime {
		elapsed_time_seconds: u64,
		paused: bool,
	},
	Break {
		elapsed_time_seconds: u64,
		paused: bool,
		break_duration_minutes: usize,
		break_over_notification_sent: bool,
	},
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum SerializedContentPage {
	#[default]
	Overview,
	Stopwatch,
	Project(ProjectId),
}

#[derive(Clone, Debug)]
pub enum PreferenceMessage {
	Save,
	Saved(Instant), // begin_time of saving
	Reset,
	Export,
	Exported,
	Import,
	ImportFailed,

	SetThemeMode(ThemeMode),
	ToggleShowSidebar,
	SetContentPage(SerializedContentPage),
	SetSynchronization(Option<SynchronizationSetting>),
	SetDateFormatting(DateFormatting),
	SetCreateNewTaskAtTop(bool),
	SetSortUnspecifiedTasksAtBottom(bool),
	SetPlayTimerNotificationSound(bool),
}

impl From<PreferenceMessage> for Message {
	fn from(value: PreferenceMessage) -> Self {
		Message::PreferenceMessage(value)
	}
}

pub enum PreferenceAction {
	None,
	Task(Task<Message>),
	PreferenceMessage(PreferenceMessage),
	RefreshCachedTaskList,
	FailedToSerailizePreferences(serde_json::Error),
}

impl From<PreferenceMessage> for PreferenceAction {
	fn from(value: PreferenceMessage) -> Self {
		PreferenceAction::PreferenceMessage(value)
	}
}

impl From<Task<Message>> for PreferenceAction {
	fn from(value: Task<Message>) -> Self {
		PreferenceAction::Task(value)
	}
}

#[derive(Debug, Error)]
pub enum LoadPreferencesError {
	#[error("failed to find preferences filepath")]
	FailedToFindPreferencesFilepath,
	#[error("failed to open preferences file: {filepath}\n{error}")]
	FailedToOpenFile {
		filepath: PathBuf,
		error: std::io::Error,
	},
	#[error("parsing error:\nfailed to load preferences in '{filepath}'")]
	FailedToParse {
		filepath: PathBuf,
		error: serde_json::Error,
	},
}

pub type LoadPreferencesResult = Result<Preferences, LoadPreferencesError>;

#[derive(Debug, Error)]
pub enum SavePreferencesError {
	#[error("failed to find preferences filepath")]
	FailedToFindPreferencesFilepath,
	#[error("failed to save preferences file: {filepath}, error: {error}")]
	FailedToSaveFile {
		filepath: PathBuf,
		error: std::io::Error,
	},
}
pub type SavePreferencesResult<T> = Result<T, SavePreferencesError>;

impl Preferences {
	const FILE_NAME: &'static str = "preferences.json";

	pub fn synchronization(&self) -> &Option<SynchronizationSetting> {
		&self.synchronization
	}
	pub fn theme_mode(&self) -> &ThemeMode {
		&self.theme_mode
	}
	pub fn selected_content_page(&self) -> &SerializedContentPage {
		&self.selected_content_page
	}
	pub fn set_selected_content_page(&mut self, content_page: SerializedContentPage) {
		self.modify(|pref| pref.selected_content_page = content_page);
	}
	pub fn stopwatch_progress(&self) -> &Option<StopwatchProgress> {
		&self.stopwatch_progress
	}
	pub fn set_stopwatch_progress(&mut self, progress: Option<StopwatchProgress>) {
		self.modify(|pref| pref.stopwatch_progress = progress);
	}
	pub fn show_sidebar(&self) -> bool {
		self.show_sidebar
	}
	pub fn date_formatting(&self) -> DateFormatting {
		self.date_formatting
	}
	pub fn create_new_tasks_at_top(&self) -> bool {
		self.create_new_tasks_at_top
	}
	pub fn sort_unspecified_tasks_at_bottom(&self) -> bool {
		self.sort_unspecified_tasks_at_bottom
	}
	pub fn code_editor(&self) -> &Option<CodeEditor> {
		&self.code_editor
	}
	pub fn set_code_editor(&mut self, code_editor: Option<CodeEditor>) {
		self.code_editor = code_editor;
	}

	fn modify(&mut self, f: impl FnOnce(&mut Preferences)) {
		f(self);
		self.modified();
	}

	fn modified(&mut self) {
		self.last_changed_time = Instant::now();
	}

	pub fn has_unsaved_changes(&self) -> bool {
		self.last_changed_time > self.last_saved_time
	}

	pub fn update(&mut self, message: PreferenceMessage) -> PreferenceAction {
		match message {
			PreferenceMessage::Save => match self.to_json() {
				Ok(json) => {
					PreferenceAction::Task(Task::perform(Self::save(json), |result| match result {
						Ok(begin_time) => PreferenceMessage::Saved(begin_time).into(),
						Err(error) => ErrorMsgModalMessage::open_error(error),
					}))
				}
				Err(e) => PreferenceAction::FailedToSerailizePreferences(e),
			},
			PreferenceMessage::Saved(begin_time) => {
				self.last_saved_time = begin_time;
				PreferenceAction::None
			}
			PreferenceMessage::Reset => {
				*self = Preferences::default();
				self.modified();
				PreferenceAction::None
			}
			PreferenceMessage::Export => match self.to_json() {
				Ok(json) => PreferenceAction::Task(Task::perform(
					Self::export_file_dialog(json),
					|result| match result {
						Ok(_) => PreferenceMessage::Exported.into(),
						Err(error) => ErrorMsgModalMessage::open_error(error),
					},
				)),
				Err(e) => PreferenceAction::FailedToSerailizePreferences(e),
			},
			PreferenceMessage::Exported => PreferenceAction::None,
			PreferenceMessage::Import => {
				Task::perform(Preferences::import_file_dialog(), |result| {
					if let Some(load_preference_result) = result {
						Message::LoadedPreferences(load_preference_result.map_err(Arc::new))
					} else {
						PreferenceMessage::ImportFailed.into()
					}
				})
				.into()
			}
			PreferenceMessage::ImportFailed => PreferenceAction::None,

			PreferenceMessage::SetThemeMode(theme_mode) => {
				self.modify(|pref| pref.theme_mode = theme_mode);
				PreferenceAction::None
			}

			PreferenceMessage::ToggleShowSidebar => {
				self.modify(|pref| pref.show_sidebar = !pref.show_sidebar);
				PreferenceAction::None
			}

			PreferenceMessage::SetContentPage(content_page) => {
				self.set_selected_content_page(content_page);
				PreferenceAction::None
			}

			PreferenceMessage::SetSynchronization(setting) => {
				self.modify(|pref| pref.synchronization = setting);
				PreferenceAction::None
			}

			PreferenceMessage::SetDateFormatting(date_formatting) => {
				self.modify(|pref| pref.date_formatting = date_formatting);
				PreferenceAction::None
			}

			PreferenceMessage::SetCreateNewTaskAtTop(create_at_top) => {
				self.modify(|pref| pref.create_new_tasks_at_top = create_at_top);
				PreferenceAction::None
			}

			PreferenceMessage::SetSortUnspecifiedTasksAtBottom(
				sort_unspecified_tasks_at_bottom,
			) => {
				self.modify(|pref| {
					pref.sort_unspecified_tasks_at_bottom = sort_unspecified_tasks_at_bottom
				});
				PreferenceAction::RefreshCachedTaskList
			}

			PreferenceMessage::SetPlayTimerNotificationSound(play_sound) => {
				self.modify(|pref| pref.play_timer_notification_sound = play_sound);
				PreferenceAction::None
			}
		}
	}

	pub fn get_filepath() -> Option<PathBuf> {
		let project_dirs = directories::ProjectDirs::from("", "", "ProjectTracker")?;

		Some(
			project_dirs
				.config_local_dir()
				.join(Self::FILE_NAME)
				.to_path_buf(),
		)
	}

	async fn get_and_ensure_filepath() -> Option<PathBuf> {
		let filepath = Self::get_filepath()?;
		tokio::fs::create_dir_all(filepath.parent()?).await.ok()?;

		Some(filepath)
	}

	async fn load_from(filepath: PathBuf) -> LoadPreferencesResult {
		let file_content = tokio::fs::read_to_string(&filepath)
			.await
			.map_err(|error| LoadPreferencesError::FailedToOpenFile {
				filepath: filepath.clone(),
				error,
			})?;

		serde_json::from_str(&file_content)
			.map_err(|error| LoadPreferencesError::FailedToParse { filepath, error })
	}

	pub async fn load() -> LoadPreferencesResult {
		Self::load_from(
			Self::get_and_ensure_filepath()
				.await
				.ok_or(LoadPreferencesError::FailedToFindPreferencesFilepath)?,
		)
		.await
	}

	async fn save_to(filepath: PathBuf, json: String) -> SavePreferencesResult<()> {
		tokio::fs::write(&filepath, json.as_bytes())
			.await
			.map_err(|error| SavePreferencesError::FailedToSaveFile { filepath, error })
	}

	pub fn to_json(&self) -> serde_json::Result<String> {
		serde_json::to_string_pretty(self)
	}

	// returns begin time of saving
	pub async fn save(json: String) -> SavePreferencesResult<Instant> {
		let begin_time = Instant::now();
		Self::save_to(
			Self::get_and_ensure_filepath()
				.await
				.ok_or(SavePreferencesError::FailedToFindPreferencesFilepath)?,
			json,
		)
		.await?;
		Ok(begin_time)
	}

	pub async fn export_file_dialog(json: String) -> SavePreferencesResult<()> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Export ProjectTracker Preferences")
			.set_file_name(Self::FILE_NAME)
			.add_filter("Preference (.json)", &["json"])
			.save_file()
			.await;

		if let Some(result) = file_dialog_result {
			Self::save_to(result.path().to_path_buf(), json).await?;
		}
		Ok(())
	}

	pub async fn import_file_dialog() -> Option<LoadPreferencesResult> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Import ProjectTracker Preferences")
			.add_filter("Preference (.json)", &["json"])
			.pick_file()
			.await;

		if let Some(result) = file_dialog_result {
			Some(Self::load_from(result.path().to_path_buf()).await)
		} else {
			None
		}
	}

	fn setting_item<'a>(
		label: impl Into<Element<'a, Message>>,
		content: impl Into<Element<'a, Message>>,
	) -> Row<'a, Message> {
		row![
			label.into(),
			container(content).width(Fill).align_x(Horizontal::Right),
		]
		.align_y(Alignment::Center)
	}

	pub fn view(&self) -> Element<Message> {
		column![
			Self::setting_item(
				"Theme Mode:",
				row![
					theme_mode_button(ThemeMode::System, self.theme_mode, true, false),
					theme_mode_button(ThemeMode::Dark, self.theme_mode, false, false),
					theme_mode_button(ThemeMode::Light, self.theme_mode, false, true),
				]
			),
			Self::setting_item(
				"Date Formatting:",
				row![
					date_formatting_button(
						&DateFormatting::DayMonthYear,
						&self.date_formatting,
						true
					),
					date_formatting_button(
						&DateFormatting::MonthDayYear,
						&self.date_formatting,
						false
					),
				]
			),
			Self::setting_item(
				"Create new tasks at top:",
				toggler(self.create_new_tasks_at_top)
					.on_toggle(|create_at_top| {
						PreferenceMessage::SetCreateNewTaskAtTop(create_at_top).into()
					})
					.size(27.5)
			),
			Self::setting_item(
				"Sort unspecified tasks at the bottom:",
				toggler(self.sort_unspecified_tasks_at_bottom)
					.on_toggle(|sort_unspecified_tasks_at_bottom| {
						PreferenceMessage::SetSortUnspecifiedTasksAtBottom(
							sort_unspecified_tasks_at_bottom,
						)
						.into()
					})
					.size(27.5)
			),
			Self::setting_item(
				"Play timer notification sound:",
				toggler(self.play_timer_notification_sound)
					.on_toggle(|play_timer_notification_sound| {
						PreferenceMessage::SetPlayTimerNotificationSound(
							play_timer_notification_sound,
						)
						.into()
					})
					.size(27.5)
			),
			horizontal_seperator_padded(),
			Self::setting_item(
				container("Preferences file location:").padding(HORIZONTAL_SCROLLABLE_PADDING),
				match Self::get_filepath() {
					Some(filepath) => file_location(filepath),
					None => text("couldnt get filepath").into(),
				}
			),
			container(
				row![
					dangerous_button(
						Bootstrap::Trash,
						"Reset",
						Some("Reset Preferences".to_string()),
						PreferenceMessage::Reset
					),
					dangerous_button(
						Bootstrap::Download,
						"Import",
						None,
						PreferenceMessage::Import
					),
					dangerous_button(Bootstrap::Upload, "Export", None, PreferenceMessage::Export),
				]
				.spacing(SPACING_AMOUNT)
			)
			.width(Fill)
			.align_x(Horizontal::Right),
		]
		.spacing(SPACING_AMOUNT)
		.into()
	}
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum DateFormatting {
	#[default]
	DayMonthYear,
	MonthDayYear,
}

impl DateFormatting {
	pub fn as_str(&self) -> &'static str {
		match self {
			DateFormatting::DayMonthYear => "DD.MM.YY",
			DateFormatting::MonthDayYear => "MM.DD.YY",
		}
	}

	pub fn format(&self, date: &SerializableDate) -> String {
		match self {
			DateFormatting::DayMonthYear => {
				format!("{}.{}.{}", date.day, date.month, date.year)
			}
			DateFormatting::MonthDayYear => {
				format!("{}.{}.{}", date.month, date.day, date.year)
			}
		}
	}
}

pub trait OptionalPreference {
	fn show_sidebar(&self) -> bool;
	fn date_formatting(&self) -> DateFormatting;
	fn create_new_tasks_at_top(&self) -> bool;
	fn sort_unspecified_tasks_at_bottom(&self) -> bool;
	fn synchronization(&self) -> Option<&SynchronizationSetting>;
	fn play_timer_notification_sound(&self) -> bool;
	fn code_editor(&self) -> Option<&CodeEditor>;
}

impl OptionalPreference for Option<Preferences> {
	fn show_sidebar(&self) -> bool {
		if let Some(preferences) = self {
			preferences.show_sidebar
		} else {
			default_show_sidebar()
		}
	}
	fn date_formatting(&self) -> DateFormatting {
		if let Some(preferences) = self {
			preferences.date_formatting
		} else {
			DateFormatting::default()
		}
	}
	fn create_new_tasks_at_top(&self) -> bool {
		if let Some(preferences) = self {
			preferences.create_new_tasks_at_top
		} else {
			default_create_new_tasks_at_top()
		}
	}
	fn sort_unspecified_tasks_at_bottom(&self) -> bool {
		if let Some(preferences) = self {
			preferences.sort_unspecified_tasks_at_bottom
		} else {
			default_sort_unspecified_tasks_at_bottom()
		}
	}
	fn synchronization(&self) -> Option<&SynchronizationSetting> {
		self.as_ref()
			.and_then(|preferences| preferences.synchronization.as_ref())
	}
	fn play_timer_notification_sound(&self) -> bool {
		if let Some(preferences) = self {
			preferences.play_timer_notification_sound
		} else {
			default_play_timer_notification_sound()
		}
	}
	fn code_editor(&self) -> Option<&CodeEditor> {
		self.as_ref().and_then(|prefs| prefs.code_editor.as_ref())
	}
}
