// only enables the 'windows' subsystem when compiling in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{window::{self, icon}, Application, Font, Settings, Size};
#[cfg(target_os = "linux")]
use iced::window::settings::PlatformSpecific;
use project_tracker_gui::ProjectTrackerApp;

fn main() -> Result<(), iced::Error> {
	ProjectTrackerApp::run(Settings {
		window: window::Settings {
			icon: icon::from_file_data(
				include_bytes!("../assets/icon.png"),
				Some(image::ImageFormat::Png)
			).ok(),
			exit_on_close_request: false,
			size: Size::new(1200.0, 900.0),
			#[cfg(target_os = "linux")]
			platform_specific: PlatformSpecific { application_id: "Project Tracker".to_string() },
			..Default::default()
		},
		antialiasing: true,
		default_font: Font::with_name("Fira Sans"),
		..Default::default()
	})
}
