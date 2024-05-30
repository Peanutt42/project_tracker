use iced::{Settings, Application, window::{self, icon}};

mod core;
mod components;
mod pages;
mod styles;
mod project_tracker;
use project_tracker::ProjectTrackerApp;
mod theme_mode;

fn main() -> Result<(), iced::Error> {
	ProjectTrackerApp::run(Settings {
		window: window::Settings {
			icon: icon::from_file_data(
				include_bytes!("../assets/icon-handdrawn.png"),
				Some(image::ImageFormat::Png)
			).ok(),
			exit_on_close_request: false,
			..Default::default()
		},
		antialiasing: true,
		..Default::default()
	})
}
