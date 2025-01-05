use iced::{
	window::{self, icon, settings::PlatformSpecific},
	Size,
};
use iced_fonts::REQUIRED_FONT_BYTES;
use project_tracker::{
	icons::{APP_ICON_BYTES, BOOTSTRAP_FONT_BYTES},
	styles::{FIRA_SANS_FONT, FIRA_SANS_FONT_BYTES},
	AppFlags, Database, Preferences, ProjectTrackerApp,
};

fn main() -> Result<(), iced::Error> {
	tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new()).unwrap();

	let temp_dir = std::env::temp_dir();
	let custom_database_filepath = temp_dir.join(Database::FILE_NAME);
	let custom_preferences_filepath = temp_dir.join(Preferences::FILE_NAME);

	// clean up previous temp files from us
	let _ = std::fs::remove_file(&custom_database_filepath);
	let _ = std::fs::remove_file(&custom_preferences_filepath);

	iced::application(
		ProjectTrackerApp::title,
		ProjectTrackerApp::update,
		ProjectTrackerApp::view,
	)
	.theme(ProjectTrackerApp::theme)
	.subscription(ProjectTrackerApp::subscription)
	.font(BOOTSTRAP_FONT_BYTES)
	.font(REQUIRED_FONT_BYTES)
	.font(FIRA_SANS_FONT_BYTES)
	.default_font(FIRA_SANS_FONT)
	.antialiasing(true)
	.window(window::Settings {
		icon: icon::from_file_data(APP_ICON_BYTES, Some(image::ImageFormat::Png)).ok(),
		exit_on_close_request: false,
		size: Size::new(1200.0, 900.0),
		#[cfg(target_os = "linux")]
		platform_specific: PlatformSpecific {
			application_id: "project_tracker".to_string(),
			..Default::default()
		},
		..Default::default()
	})
	.run_with(move || {
		ProjectTrackerApp::new(AppFlags::custom(
			custom_database_filepath,
			custom_preferences_filepath,
		))
	})
}
