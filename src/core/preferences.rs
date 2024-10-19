use crate::icons::Bootstrap;
use crate::integrations::ServerConfig;
use crate::{
	components::{
		dangerous_button, date_formatting_button, file_location, horizontal_seperator_padded,
		theme_mode_button, HORIZONTAL_SCROLLABLE_PADDING,
	},
	modals::ErrorMsgModalMessage,
	core::{ProjectId, SerializableDate, TaskId},
	project_tracker::Message,
	styles::SPACING_AMOUNT,
	theme_mode::ThemeMode,
};
use iced::{
	alignment::Horizontal,
	widget::{column, container, row, toggler, Row},
	Alignment, Element,
	Length::Fill,
	Task,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

fn default_show_sidebar() -> bool { true }
fn default_create_new_tasks_at_top() -> bool { true }
fn default_sort_unspecified_tasks_at_bottom() -> bool { true }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preferences {
	theme_mode: ThemeMode,

	date_formatting: DateFormatting,

	#[serde(default = "default_create_new_tasks_at_top")]
	create_new_tasks_at_top: bool,

	#[serde(default = "default_sort_unspecified_tasks_at_bottom")]
	sort_unspecified_tasks_at_bottom: bool,

	#[serde(default = "default_show_sidebar")]
	show_sidebar: bool,

	selected_content_page: SerializedContentPage,

	stopwatch_progress: Option<StopwatchProgress>,

	synchronization_filepath: Option<PathBuf>,

	server_synchronization: Option<ServerConfig>,

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
			selected_content_page: SerializedContentPage::default(),
			stopwatch_progress: None,
			synchronization_filepath: None,
			server_synchronization: None,
			last_changed_time: Instant::now(),
			last_saved_time: Instant::now(),
		}
	}
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct StopwatchProgress {
	pub task: Option<(ProjectId, TaskId)>,
	pub elapsed_time_seconds: u64,
	pub paused: bool,
	pub finished_notification_sent: bool,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum SerializedContentPage {
	#[default]
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
	SetStopwatchProgress(Option<StopwatchProgress>),
	SetSynchronizationFilepath(Option<PathBuf>),
	SetDateFormatting(DateFormatting),
	SetCreateNewTaskAtTop(bool),
	SetSortUnspecifiedTasksAtBottom(bool),
}

impl From<PreferenceMessage> for Message {
	fn from(value: PreferenceMessage) -> Self {
		Message::PreferenceMessage(value)
	}
}

pub enum PreferenceAction {
	None,
	Task(Task<Message>),
	RefreshCachedTaskList,
}

impl From<Task<Message>> for PreferenceAction {
	fn from(value: Task<Message>) -> Self {
		PreferenceAction::Task(value)
	}
}

#[derive(Clone, Debug)]
pub enum LoadPreferencesResult {
	Ok(Preferences),
	FailedToOpenFile(PathBuf),
	FailedToParse(PathBuf),
}

impl Preferences {
	const FILE_NAME: &'static str = "preferences.json";

	pub fn synchronization_filepath(&self) -> &Option<PathBuf> {
		&self.synchronization_filepath
	}
	pub fn server_synchronization(&self) -> &Option<ServerConfig> {
		&self.server_synchronization
	}
	pub fn set_server_synchronization(&mut self, config: Option<ServerConfig>) {
		self.modify(|pref| pref.server_synchronization = config);
	}
	pub fn theme_mode(&self) -> &ThemeMode {
		&self.theme_mode
	}
	pub fn selected_content_page(&self) -> &SerializedContentPage {
		&self.selected_content_page
	}
	pub fn stopwatch_progress(&self) -> &Option<StopwatchProgress> {
		&self.stopwatch_progress
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

	pub fn modify(&mut self, f: impl FnOnce(&mut Preferences)) {
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
			PreferenceMessage::Save => {
				Task::perform(Self::save(self.to_json()), |result| match result {
					Ok(begin_time) => PreferenceMessage::Saved(begin_time).into(),
					Err(error_msg) => ErrorMsgModalMessage::open(error_msg),
				})
				.into()
			}
			PreferenceMessage::Saved(begin_time) => {
				self.last_saved_time = begin_time;
				PreferenceAction::None
			}
			PreferenceMessage::Reset => {
				*self = Preferences::default();
				self.modified();
				PreferenceAction::None
			}
			PreferenceMessage::Export => Task::perform(
				Self::export_file_dialog(self.to_json()),
				|result| match result {
					Ok(_) => PreferenceMessage::Exported.into(),
					Err(error_msg) => ErrorMsgModalMessage::open(error_msg),
				},
			)
			.into(),
			PreferenceMessage::Exported => PreferenceAction::None,
			PreferenceMessage::Import => {
				Task::perform(Preferences::import_file_dialog(), |result| {
					if let Some(load_preference_result) = result {
						Message::LoadedPreferences(load_preference_result)
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
				self.modify(|pref| pref.selected_content_page = content_page);
				PreferenceAction::None
			}

			PreferenceMessage::SetStopwatchProgress(progress) => {
				self.modify(|pref| pref.stopwatch_progress = progress);
				PreferenceAction::None
			}

			PreferenceMessage::SetSynchronizationFilepath(filepath) => {
				self.modify(|pref| pref.synchronization_filepath = filepath);
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

			PreferenceMessage::SetSortUnspecifiedTasksAtBottom(sort_unspecified_tasks_at_bottom) => {
				self.modify(|pref| pref.sort_unspecified_tasks_at_bottom = sort_unspecified_tasks_at_bottom);
				PreferenceAction::RefreshCachedTaskList
			}
		}
	}

	pub fn get_filepath() -> PathBuf {
		let project_dirs = directories::ProjectDirs::from("", "", "ProjectTracker")
			.expect("Failed to get saved state filepath");

		project_dirs
			.config_local_dir()
			.join(Self::FILE_NAME)
			.to_path_buf()
	}

	async fn get_and_ensure_filepath() -> PathBuf {
		let filepath = Self::get_filepath();

		tokio::fs::create_dir_all(filepath.parent().unwrap())
			.await
			.expect("Failed to create Local Config Directories");

		filepath
	}

	async fn load_from(filepath: PathBuf) -> LoadPreferencesResult {
		let file_content = if let Ok(file_content) = tokio::fs::read_to_string(&filepath).await {
			file_content
		} else {
			return LoadPreferencesResult::FailedToOpenFile(filepath);
		};

		match serde_json::from_str(&file_content) {
			Ok(preferences) => LoadPreferencesResult::Ok(preferences),
			Err(_) => LoadPreferencesResult::FailedToParse(filepath),
		}
	}

	pub async fn load() -> LoadPreferencesResult {
		Self::load_from(Self::get_and_ensure_filepath().await).await
	}

	async fn save_to(filepath: PathBuf, json: String) -> Result<(), String> {
		if let Err(e) = tokio::fs::write(&filepath, json.as_bytes()).await {
			Err(format!("Failed to save to {}: {e}", filepath.display()))
		} else {
			Ok(())
		}
	}

	pub fn to_json(&self) -> String {
		serde_json::to_string_pretty(self).unwrap()
	}

	// returns begin time of saving
	pub async fn save(json: String) -> Result<Instant, String> {
		let begin_time = Instant::now();
		Self::save_to(Self::get_and_ensure_filepath().await, json).await?;
		Ok(begin_time)
	}

	pub async fn export_file_dialog(json: String) -> Result<(), String> {
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
					.on_toggle(|create_at_top| PreferenceMessage::SetCreateNewTaskAtTop(
						create_at_top
					)
					.into())
					.size(27.5)
			),
			Self::setting_item(
				"Sort unspecified tasks at the bottom",
				toggler(self.sort_unspecified_tasks_at_bottom)
					.on_toggle(|sort_unspecified_tasks_at_bottom| PreferenceMessage::SetSortUnspecifiedTasksAtBottom(
						sort_unspecified_tasks_at_bottom
					)
					.into())
					.size(27.5)
			),
			horizontal_seperator_padded(),
			Self::setting_item(
				container("Preferences file location:").padding(HORIZONTAL_SCROLLABLE_PADDING),
				file_location(Self::get_filepath())
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
			DateFormatting::DayMonthYear => format!("{}.{}.{}", date.day, date.month, date.year),
			DateFormatting::MonthDayYear => format!("{}.{}.{}", date.month, date.day, date.year),
		}
	}
}


pub trait OptionalPreference {
	fn show_sidebar(&self) -> bool;
	fn date_formatting(&self) -> DateFormatting;
	fn create_new_tasks_at_top(&self) -> bool;
	fn sort_unspecified_tasks_at_bottom(&self) -> bool;
	fn server_synchronization(&self) -> Option<&ServerConfig>;
}

impl OptionalPreference for Option<Preferences> {
	fn show_sidebar(&self) -> bool {
		if let Some(preferences) = self {
			preferences.show_sidebar
		}
		else {
			default_show_sidebar()
		}
	}
	fn date_formatting(&self) -> DateFormatting {
		if let Some(preferences) = self {
			preferences.date_formatting
		}
		else {
			DateFormatting::default()
		}
	}
	fn create_new_tasks_at_top(&self) -> bool {
		if let Some(preferences) = self {
			preferences.create_new_tasks_at_top
		}
		else {
			default_create_new_tasks_at_top()
		}
	}
	fn sort_unspecified_tasks_at_bottom(&self) -> bool {
		if let Some(preferences) = self {
			preferences.sort_unspecified_tasks_at_bottom
		}
		else {
			default_sort_unspecified_tasks_at_bottom()
		}
	}
	fn server_synchronization(&self) -> Option<&ServerConfig> {
		if let Some(preferences) = self {
			preferences.server_synchronization.as_ref()
		}
		else {
			None
		}
	}
}