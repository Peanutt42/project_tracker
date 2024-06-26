use std::path::PathBuf;
use std::time::Instant;
use iced::{alignment::{Horizontal, Vertical}, widget::{column, container, row, text}, Alignment, Command, Element, Length};
use serde::{Serialize, Deserialize};
use crate::{components::{dangerous_button, file_location, theme_mode_button, ErrorMsgModalMessage}, project_tracker::UiMessage, styles::SPACING_AMOUNT, theme_mode::ThemeMode};

fn default_sidebar_dividor_position() -> u16 {
	300
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preferences {
	pub theme_mode: ThemeMode,

	#[serde(default = "default_sidebar_dividor_position")]
	pub sidebar_dividor_position: u16,

	#[serde(skip, default = "Instant::now")]
	last_changed_time: Instant,

	#[serde(skip, default = "Instant::now")]
	last_saved_time: Instant,
}

impl Default for Preferences {
	fn default() -> Self {
		Self {
			theme_mode: ThemeMode::default(),
			sidebar_dividor_position: default_sidebar_dividor_position(),
			last_changed_time: Instant::now(),
			last_saved_time: Instant::now(),
		}
	}
}

#[derive(Clone, Debug, Copy)]
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

	fn change_was_made(&mut self) {
		self.last_changed_time = Instant::now();
	}

	pub fn has_unsaved_changes(&self) -> bool {
		self.last_changed_time > self.last_saved_time
	}

	pub fn update(&mut self, message: PreferenceMessage) -> Command<UiMessage> {
		match message {
			PreferenceMessage::Save => Command::perform(
				self.clone().save(),
				|result| {
					match result {
						Ok(begin_time) => PreferenceMessage::Saved(begin_time).into(),
						Err(error_msg) => ErrorMsgModalMessage::open(error_msg),
					}
				}
			),
			PreferenceMessage::Saved(begin_time) => { self.last_saved_time = begin_time; Command::none() },
			PreferenceMessage::Reset => { *self = Preferences::default(); self.change_was_made(); Command::none() },
			PreferenceMessage::Export => Command::perform(
				self.clone().export_file_dialog(),
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
				self.theme_mode = theme_mode;
				self.change_was_made();
				Command::none()
			},
			PreferenceMessage::SetSidebarDividorPosition(dividor_position) => {
				self.sidebar_dividor_position = dividor_position;
				self.change_was_made();
				Command::none()
			},
		}
	}

	fn get_filepath() -> PathBuf {
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
		let file_content = if let Ok(file_content) = tokio::fs::read_to_string(filepath.clone()).await {
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

	async fn save_to(self, filepath: PathBuf) -> Result<(), String> {
		if let Err(e) = tokio::fs::write(filepath.clone(), serde_json::to_string_pretty(&self).unwrap().as_bytes()).await {
			Err(format!("Failed to save to {}: {e}", filepath.display()))
		}
		else {
			Ok(())
		}
	}

	// returns begin time of saving
	pub async fn save(self) -> Result<Instant, String> {
		let begin_time = Instant::now();
		self.save_to(Self::get_and_ensure_filepath().await).await?;
		Ok(begin_time)
	}

	pub async fn export_file_dialog(self) -> Result<(), String> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Export ProjectTracker Preferences")
			.set_file_name(Self::FILE_NAME)
			.add_filter("Preference (.json)", &["json"])
			.save_file()
			.await;

		if let Some(result) = file_dialog_result {
			self.save_to(result.path().to_path_buf()).await?;
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

	pub fn view(&self) -> Element<UiMessage> {
		column![
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
				dangerous_button("Reset Preferences", PreferenceMessage::Reset),

				dangerous_button("Import Preferences", PreferenceMessage::Import),

				dangerous_button("Export Preferences", PreferenceMessage::Export),
			]
			.spacing(SPACING_AMOUNT),

			row![
				text("Preference file location: "),
				file_location(Self::get_filepath())
			]
			.align_items(Alignment::Center)
		]
		.spacing(SPACING_AMOUNT)
		.into()
	}
}
