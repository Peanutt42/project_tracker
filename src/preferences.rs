use crate::components::first_weekday_button;
use crate::icons::Bootstrap;
use crate::integrations::CodeEditor;
use crate::pages::overview_page::CalendarView;
use crate::pages::sidebar_page;
use crate::project_tracker::AppFlags;
use crate::synchronization::Synchronization;
use crate::{
	components::{
		dangerous_button, date_formatting_button, file_location, horizontal_seperator_padded,
		theme_mode_button, HORIZONTAL_SCROLLABLE_PADDING,
	},
	modals::error_msg_modal,
	project_tracker::Message,
	styles::SPACING_AMOUNT,
	theme_mode::ThemeMode,
};
use crate::{ProjectId, SerializableDate, TaskId};
use chrono::Weekday;
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
fn default_sidebar_ratio() -> f32 {
	sidebar_page::Page::DEFAULT_SPLIT_RATIO
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preferences {
	#[serde(default)]
	theme_mode: ThemeMode,

	#[serde(default)]
	date_formatting: DateFormatting,

	#[serde(default)]
	first_day_of_week: FirstWeekday,

	#[serde(default = "default_create_new_tasks_at_top")]
	create_new_tasks_at_top: bool,

	#[serde(default = "default_sort_unspecified_tasks_at_bottom")]
	sort_unspecified_tasks_at_bottom: bool,

	#[serde(default = "default_play_timer_notification_sound")]
	play_timer_notification_sound: bool,

	#[serde(default = "default_show_sidebar")]
	show_sidebar: bool,

	#[serde(default = "default_sidebar_ratio")]
	sidebar_ratio: f32,

	#[serde(default)]
	selected_content_page: SerializedContentPage,

	#[serde(default)]
	serialized_overview_page: SerializedOverviewPage,

	#[serde(default)]
	stopwatch_progress: Option<StopwatchProgress>,

	#[serde(default)]
	code_editor: Option<CodeEditor>,

	#[serde(default)]
	synchronization: Option<Synchronization>,

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
			first_day_of_week: FirstWeekday::default(),
			create_new_tasks_at_top: default_create_new_tasks_at_top(),
			sort_unspecified_tasks_at_bottom: default_sort_unspecified_tasks_at_bottom(),
			show_sidebar: default_show_sidebar(),
			sidebar_ratio: default_sidebar_ratio(),
			play_timer_notification_sound: default_play_timer_notification_sound(),
			selected_content_page: SerializedContentPage::default(),
			serialized_overview_page: SerializedOverviewPage::default(),
			stopwatch_progress: None,
			code_editor: None,
			synchronization: None,
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SerializedOverviewPage {
	List,
	Calendar { view: CalendarView },
}
impl SerializedOverviewPage {
	pub const DEFAULT: Self = Self::List;

	pub fn get_calendar_view(&self) -> CalendarView {
		match self {
			Self::Calendar { view } => *view,
			_ => CalendarView::default(),
		}
	}
}
impl Default for SerializedOverviewPage {
	fn default() -> Self {
		Self::DEFAULT
	}
}

#[derive(Clone, Debug)]
pub enum PreferenceMessage {
	Save(PathBuf),
	Saved(Instant), // begin_time of saving
	Reset,
	Export,
	Exported,
	Import,
	ImportFailed,

	SetThemeMode(ThemeMode),
	ToggleShowSidebar,
	SetContentPage(SerializedContentPage),
	SetDateFormatting(DateFormatting),
	SetFirstWeekday(FirstWeekday),
	SetOverviewPage(SerializedOverviewPage),
	SetCreateNewTaskAtTop(bool),
	SetSortUnspecifiedTasksAtBottom(bool),
	SetPlayTimerNotificationSound(bool),

	SetSynchronization(Option<Synchronization>),
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
	RequestAdminInfos,
	FailedToSerializePreferences(toml::ser::Error),
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
	#[error("failed to open preferences file: {filepath}\n{error}")]
	FailedToOpenFile {
		filepath: PathBuf,
		error: std::io::Error,
	},
	#[error("parsing error:\nfailed to load preferences in '{filepath}'")]
	FailedToParse {
		filepath: PathBuf,
		error: toml::de::Error,
	},
}

pub type LoadPreferencesResult = Result<Preferences, LoadPreferencesError>;

#[derive(Debug, Error)]
pub enum SavePreferencesError {
	#[error("failed to save preferences file: {filepath}, error: {error}")]
	FailedToSaveFile {
		filepath: PathBuf,
		error: std::io::Error,
	},
}
pub type SavePreferencesResult<T> = Result<T, SavePreferencesError>;

impl Preferences {
	pub const FILE_NAME: &'static str = "preferences.toml";

	pub fn synchronization(&self) -> &Option<Synchronization> {
		&self.synchronization
	}
	pub fn set_synchronization(&mut self, synchronization: Option<Synchronization>) {
		self.modify(|pref| pref.synchronization = synchronization);
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
	pub fn serialized_overview_page(&self) -> &SerializedOverviewPage {
		&self.serialized_overview_page
	}
	pub fn set_serialized_overview_page(
		&mut self,
		serialized_overview_page: SerializedOverviewPage,
	) {
		self.modify(|pref| pref.serialized_overview_page = serialized_overview_page);
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
	pub fn sidebar_ratio(&self) -> f32 {
		self.sidebar_ratio
	}
	pub fn set_sidebar_ratio(&mut self, ratio: f32) {
		self.modify(|pref| pref.sidebar_ratio = ratio);
	}
	pub fn date_formatting(&self) -> DateFormatting {
		self.date_formatting
	}
	pub fn first_day_of_week(&self) -> FirstWeekday {
		self.first_day_of_week
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
			PreferenceMessage::Save(filepath) => match self.serialized() {
				Ok(serialized_str) => PreferenceAction::Task(Task::perform(
					Self::save(filepath, serialized_str),
					|result| match result {
						Ok(begin_time) => PreferenceMessage::Saved(begin_time).into(),
						Err(error) => error_msg_modal::Message::open_error(error),
					},
				)),
				Err(e) => PreferenceAction::FailedToSerializePreferences(e),
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
			PreferenceMessage::Export => match self.serialized() {
				Ok(serialized_str) => PreferenceAction::Task(Task::perform(
					Self::export_file_dialog(serialized_str),
					|result| match result {
						Ok(_) => PreferenceMessage::Exported.into(),
						Err(error) => error_msg_modal::Message::open_error(error),
					},
				)),
				Err(e) => PreferenceAction::FailedToSerializePreferences(e),
			},
			PreferenceMessage::Exported => PreferenceAction::None,
			PreferenceMessage::Import => {
				Task::perform(Preferences::import_file_dialog(), |result| match result {
					Some(load_preference_result) => {
						Message::LoadedPreferences(load_preference_result.map_err(Arc::new))
					}
					None => PreferenceMessage::ImportFailed.into(),
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

			PreferenceMessage::SetDateFormatting(date_formatting) => {
				self.modify(|pref| pref.date_formatting = date_formatting);
				PreferenceAction::None
			}

			PreferenceMessage::SetFirstWeekday(first_weekday) => {
				self.modify(|pref| pref.first_day_of_week = first_weekday);
				PreferenceAction::None
			}

			PreferenceMessage::SetOverviewPage(serialized_overview_page) => {
				self.set_serialized_overview_page(serialized_overview_page);
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

			PreferenceMessage::SetSynchronization(synchronization) => {
				self.modify(|pref| pref.synchronization = synchronization);
				PreferenceAction::None
			}
		}
	}

	fn get_default_filepath() -> Option<PathBuf> {
		let project_dirs = directories::ProjectDirs::from("", "", "ProjectTracker")?;

		Some(
			project_dirs
				.config_local_dir()
				.join(Self::FILE_NAME)
				.to_path_buf(),
		)
	}

	// either returns custom filepath or the default filepath based on the system
	pub fn get_filepath(custom_filepath: Option<PathBuf>) -> Option<PathBuf> {
		custom_filepath.or(Self::get_default_filepath())
	}

	pub async fn load(filepath: PathBuf) -> LoadPreferencesResult {
		let file_content = tokio::fs::read_to_string(&filepath)
			.await
			.map_err(|error| LoadPreferencesError::FailedToOpenFile {
				filepath: filepath.clone(),
				error,
			})?;

		toml::from_str(&file_content)
			.map_err(|error| LoadPreferencesError::FailedToParse { filepath, error })
	}

	// returns begin time of saving
	pub async fn save(filepath: PathBuf, serialized_str: String) -> SavePreferencesResult<Instant> {
		let begin_time = Instant::now();
		if let Some(parent) = filepath.parent() {
			// if this fails, 'tokio::fs::write' will also fail --> correct io error
			let _ = tokio::fs::create_dir_all(parent).await;
		}
		tokio::fs::write(&filepath, serialized_str.as_bytes())
			.await
			.map_err(|error| SavePreferencesError::FailedToSaveFile { filepath, error })?;
		Ok(begin_time)
	}

	pub fn serialized(&self) -> Result<String, toml::ser::Error> {
		toml::to_string_pretty(self)
	}

	pub async fn export_file_dialog(serialized_str: String) -> SavePreferencesResult<()> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Export ProjectTracker Preferences")
			.set_file_name(Self::FILE_NAME)
			.add_filter("Preference (.toml)", &["toml"])
			.save_file()
			.await;

		if let Some(result) = file_dialog_result {
			Self::save(result.path().to_path_buf(), serialized_str).await?;
		}
		Ok(())
	}

	pub async fn import_file_dialog() -> Option<LoadPreferencesResult> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Import ProjectTracker Preferences")
			.add_filter("Preference (.toml)", &["toml"])
			.pick_file()
			.await;

		match file_dialog_result {
			Some(result) => Some(Self::load(result.path().to_path_buf()).await),
			None => None,
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

	pub fn view<'a>(&'a self, app_flags: &'a AppFlags) -> Element<'a, Message> {
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
				"First Weekday:",
				row![
					first_weekday_button(&FirstWeekday::Monday, &self.first_day_of_week, true),
					first_weekday_button(&FirstWeekday::Sunday, &self.first_day_of_week, false),
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
				match app_flags.get_preferences_filepath() {
					Some(filepath) => file_location(filepath),
					None => text("could not get filepath").into(),
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
			DateFormatting::DayMonthYear => "DD.MM.YYYY",
			DateFormatting::MonthDayYear => "MM.DD.YYYY",
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum FirstWeekday {
	#[default]
	Monday,
	Sunday,
}

impl FirstWeekday {
	pub fn as_week_day(&self) -> Weekday {
		match self {
			Self::Monday => Weekday::Mon,
			Self::Sunday => Weekday::Sun,
		}
	}
}

pub trait OptionalPreference {
	fn show_sidebar(&self) -> bool;
	fn date_formatting(&self) -> DateFormatting;
	fn first_day_of_week(&self) -> FirstWeekday;
	fn create_new_tasks_at_top(&self) -> bool;
	fn serialized_overview_page(&self) -> &SerializedOverviewPage;
	fn sort_unspecified_tasks_at_bottom(&self) -> bool;
	fn synchronization(&self) -> Option<&Synchronization>;
	fn play_timer_notification_sound(&self) -> bool;
	fn code_editor(&self) -> Option<&CodeEditor>;
}

impl OptionalPreference for Option<Preferences> {
	fn show_sidebar(&self) -> bool {
		match self {
			Some(preferences) => preferences.show_sidebar,
			None => default_show_sidebar(),
		}
	}
	fn date_formatting(&self) -> DateFormatting {
		match self {
			Some(preferences) => preferences.date_formatting,
			None => DateFormatting::default(),
		}
	}
	fn first_day_of_week(&self) -> FirstWeekday {
		match self {
			Some(preferences) => preferences.first_day_of_week,
			None => FirstWeekday::default(),
		}
	}
	fn create_new_tasks_at_top(&self) -> bool {
		match self {
			Some(preferences) => preferences.create_new_tasks_at_top,
			None => default_create_new_tasks_at_top(),
		}
	}
	fn sort_unspecified_tasks_at_bottom(&self) -> bool {
		match self {
			Some(preferences) => preferences.sort_unspecified_tasks_at_bottom,
			None => default_sort_unspecified_tasks_at_bottom(),
		}
	}
	fn synchronization(&self) -> Option<&Synchronization> {
		self.as_ref()
			.and_then(|preferences| preferences.synchronization.as_ref())
	}
	fn serialized_overview_page(&self) -> &SerializedOverviewPage {
		match self {
			Some(preferences) => &preferences.serialized_overview_page,
			None => &SerializedOverviewPage::DEFAULT,
		}
	}
	fn play_timer_notification_sound(&self) -> bool {
		match self {
			Some(preferences) => preferences.play_timer_notification_sound,
			None => default_play_timer_notification_sound(),
		}
	}
	fn code_editor(&self) -> Option<&CodeEditor> {
		self.as_ref().and_then(|prefs| prefs.code_editor.as_ref())
	}
}
