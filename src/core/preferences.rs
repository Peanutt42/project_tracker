use std::path::PathBuf;
use iced::{alignment::{Horizontal, Vertical}, widget::{column, container, row, text}, Alignment, Element, Length};
use serde::{Serialize, Deserialize};
use crate::{project_tracker::UiMessage, styles::SPACING_AMOUNT, components::{dangerous_button, theme_mode_button}, theme_mode::ThemeMode};


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Preferences {
	pub theme_mode: ThemeMode,
}

#[derive(Debug, Clone)]
pub enum LoadPreferencesResult {
	Ok(Preferences),
	FailedToReadFile(PathBuf),
	FailedToParse(PathBuf),
}

impl Preferences {
	async fn filepath() -> PathBuf {
		let project_dirs = directories::ProjectDirs::from("", "", "ProjectTracker")
		.expect("Failed to get saved state filepath");

		let config_dir = project_dirs.config_local_dir();

		tokio::fs::create_dir_all(config_dir).await.expect("Failed to create Local Config Directories");

		config_dir
			.join("preferences.json")
			.to_path_buf()
	}

	async fn load_from(filepath: PathBuf) -> LoadPreferencesResult {
		let file_content = if let Ok(file_content) = tokio::fs::read_to_string(filepath.clone()).await {
			file_content
		}
		else {
			return LoadPreferencesResult::FailedToReadFile(filepath);
		};

		match serde_json::from_str(&file_content) {
			Ok(preferences) => LoadPreferencesResult::Ok(preferences),
			Err(_) => LoadPreferencesResult::FailedToParse(filepath),
		}
	}

	pub async fn load() -> LoadPreferencesResult {
		Self::load_from(Self::filepath().await).await
	}

	async fn save_to(self, filepath: PathBuf) {
		if let Err(e) = tokio::fs::write(filepath.clone(), serde_json::to_string_pretty(&self).unwrap().as_bytes()).await {
			eprintln!("Failed to save to {}: {e}", filepath.display());
		}
	}

	pub async fn save(self) {
		self.save_to(Self::filepath().await).await;
	}

	pub async fn export_file_dialog(self) {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.add_filter("Preference", &["json"])
			.save_file()
			.await;

		if let Some(result) = file_dialog_result {
			self.save_to(result.path().to_path_buf()).await;
		}
	}
	
	pub async fn import_file_dialog() -> Option<LoadPreferencesResult> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.add_filter("Preference", &["json"])
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
				dangerous_button("Reset Preferences")
					.on_press(UiMessage::ResetPreferences),

				dangerous_button("Import Preferences")
					.on_press(UiMessage::ImportPreferences),
				
				dangerous_button("Export Preferences")
					.on_press(UiMessage::ExportPreferences),
			]
			.spacing(SPACING_AMOUNT)
		]
		.into()
	}
}