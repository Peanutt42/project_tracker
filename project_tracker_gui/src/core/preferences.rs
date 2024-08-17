use std::path::PathBuf;
use std::time::Instant;
use iced::{alignment::{Horizontal, Vertical}, theme, widget::{button, column, container, row, text, Column}, Alignment, Command, Element, Length};
use iced_aw::{drop_down, Bootstrap, DropDown};
use serde::{Serialize, Deserialize};
use crate::{components::{dangerous_button, file_location, theme_mode_button, ErrorMsgModalMessage, SettingsModalMessage}, core::{ProjectId, SerializableDate}, project_tracker::UiMessage, styles::{DropDownContainerStyle, HiddenSecondaryButtonStyle, SecondaryButtonStyle, SPACING_AMOUNT}, theme_mode::ThemeMode};

fn default_sidebar_dividor_position() -> u16 { 300 }
fn default_show_sidebar() -> bool { true }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preferences {
	theme_mode: ThemeMode,

	date_formatting: DateFormatting,

	#[serde(default = "default_sidebar_dividor_position")]
	sidebar_dividor_position: u16,

	#[serde(default = "default_show_sidebar")]
	show_sidebar: bool,

	selected_content_page: SerializedContentPage,

	synchronization_filepath: Option<PathBuf>,

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
			sidebar_dividor_position: default_sidebar_dividor_position(),
			show_sidebar: default_show_sidebar(),
			selected_content_page: SerializedContentPage::default(),
			synchronization_filepath: None,
			last_changed_time: Instant::now(),
			last_saved_time: Instant::now(),
		}
	}
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum SerializedContentPage {
	#[default]
	Overview,
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
	SetSidebarDividorPosition(u16),
	ToggleShowSidebar,
	SetContentPage(SerializedContentPage),
	SetSynchronizationFilepath(Option<PathBuf>),
	SetDateFormatting(DateFormatting),
}

impl From<PreferenceMessage> for UiMessage {
	fn from(value: PreferenceMessage) -> Self {
		UiMessage::PreferenceMessage(value)
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

	pub fn synchronization_filepath(&self) -> &Option<PathBuf> { &self.synchronization_filepath }
	pub fn theme_mode(&self) -> &ThemeMode { &self.theme_mode }
	pub fn selected_content_page(&self) -> &SerializedContentPage { &self.selected_content_page }
	pub fn show_sidebar(&self) -> bool { self.show_sidebar }
	pub fn sidebar_dividor_position(&self) -> u16 { self.sidebar_dividor_position }
	pub fn date_formatting(&self) -> DateFormatting { self.date_formatting }

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

	pub fn update(&mut self, message: PreferenceMessage) -> Command<UiMessage> {
		match message {
			PreferenceMessage::Save => Command::perform(
				Self::save(self.to_json()),
				|result| {
					match result {
						Ok(begin_time) => PreferenceMessage::Saved(begin_time).into(),
						Err(error_msg) => ErrorMsgModalMessage::open(error_msg),
					}
				}
			),
			PreferenceMessage::Saved(begin_time) => { self.last_saved_time = begin_time; Command::none() },
			PreferenceMessage::Reset => { *self = Preferences::default(); self.modified(); Command::none() },
			PreferenceMessage::Export => Command::perform(
				Self::export_file_dialog(self.to_json()),
				|result| {
					match result {
						Ok(_) => PreferenceMessage::Exported.into(),
						Err(error_msg) => ErrorMsgModalMessage::open(error_msg),
					}
				}
			),
			PreferenceMessage::Exported => Command::none(),
			PreferenceMessage::Import => Command::perform(
				Preferences::import_file_dialog(),
				|result| {
					if let Some(load_preference_result) = result {
						UiMessage::LoadedPreferences(load_preference_result)
					}
					else {
						PreferenceMessage::ImportFailed.into()
					}
				}
			),
			PreferenceMessage::ImportFailed => Command::none(),

			PreferenceMessage::SetThemeMode(theme_mode) => {
				self.modify(|pref| pref.theme_mode = theme_mode);
				Command::none()
			},
			PreferenceMessage::SetSidebarDividorPosition(dividor_position) => {
				self.modify(|pref| pref.sidebar_dividor_position = dividor_position);
				Command::none()
			},

			PreferenceMessage::ToggleShowSidebar => {
				self.modify(|pref| pref.show_sidebar = !pref.show_sidebar);
				Command::none()
			},

			PreferenceMessage::SetContentPage(content_page) => {
				self.modify(|pref| pref.selected_content_page = content_page);
				Command::none()
			},

			PreferenceMessage::SetSynchronizationFilepath(filepath) => {
				self.modify(|pref| pref.synchronization_filepath = filepath);
				Command::none()
			},

			PreferenceMessage::SetDateFormatting(date_formatting) => {
				self.modify(|pref| pref.date_formatting = date_formatting);
				Command::none()
			},
		}
	}

	pub fn get_filepath() -> PathBuf {
		let project_dirs = directories::ProjectDirs::from("", "", "ProjectTracker")
			.expect("Failed to get saved state filepath");

		project_dirs.config_local_dir().join(Self::FILE_NAME)
			.to_path_buf()
	}

	async fn get_and_ensure_filepath() -> PathBuf {
		let filepath = Self::get_filepath();

		tokio::fs::create_dir_all(filepath.parent().unwrap()).await.expect("Failed to create Local Config Directories");

		filepath
	}

	async fn load_from(filepath: PathBuf) -> LoadPreferencesResult {
		let file_content = if let Ok(file_content) = tokio::fs::read_to_string(&filepath).await {
			file_content
		}
		else {
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
		}
		else {
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
		}
		else {
			None
		}
	}

	pub fn view(&self, date_formatting_expanded: bool) -> Element<UiMessage> {
		column![
			row![
				text("File location: "),
				container(file_location(&Self::get_filepath()))
					.width(Length::Fill)
					.align_x(Horizontal::Right)
			]
			.align_items(Alignment::Center),

			row![
				text("Theme Mode:").horizontal_alignment(Horizontal::Center).vertical_alignment(Vertical::Center),
				container(
					row![
						theme_mode_button(ThemeMode::System, self.theme_mode),
						theme_mode_button(ThemeMode::Dark, self.theme_mode),
						theme_mode_button(ThemeMode::Light, self.theme_mode),
					]
					.spacing(SPACING_AMOUNT)
					.align_items(Alignment::Center)
				)
				.width(Length::Fill)
				.align_x(Horizontal::Right),
			]
			.align_items(Alignment::Center),

			row![
				text("Date Formatting:"),
				container(
					DropDown::new(
						button(text(self.date_formatting.as_str()))
							.on_press(SettingsModalMessage::ToggleExpandDateFormatting.into())
							.style(theme::Button::custom(SecondaryButtonStyle::default())),

						container(
							Column::with_children(DateFormatting::FORMATS.iter()
								.map(|format| {
									button(
										text(format.as_str()).horizontal_alignment(Horizontal::Center)
									)
									.width(Length::Fill)
									.on_press(SettingsModalMessage::SetDateFormatting(*format).into())
									.style(
										if self.date_formatting == *format {
											theme::Button::custom(SecondaryButtonStyle::default())
										}
										else {
											theme::Button::custom(HiddenSecondaryButtonStyle)
										})
									.into()
								}))
						)
						.style(theme::Container::Custom(Box::new(DropDownContainerStyle))),

						date_formatting_expanded
					)
					.on_dismiss(SettingsModalMessage::DismissDateFormatting.into())
					.alignment(drop_down::Alignment::Bottom)
				)
				.width(Length::Fill)
				.align_x(Horizontal::Right)
			]
			.align_items(Alignment::Center),

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

				dangerous_button(
					Bootstrap::Upload,
					"Export",
					None,
					PreferenceMessage::Export
				),
			]
			.spacing(SPACING_AMOUNT),
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
	pub const FORMATS: [DateFormatting; 2] = [
		DateFormatting::DayMonthYear,
		DateFormatting::MonthDayYear,
	];

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