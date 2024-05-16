use iced::{Settings, Application, window::{self, icon}};

mod project;
mod task;
mod project_tracker;
use project_tracker::ProjectTrackerApp;
mod saved_state;
mod theme_mode;
mod components;
mod pages;
mod styles;

fn main() -> Result<(), iced::Error> {
	ProjectTrackerApp::run(Settings {
		window: window::Settings {
			icon: icon::from_file_data(
				include_bytes!("../assets/icon-handdrawn.png"),
				Some(image::ImageFormat::Png)
			).ok(),
			..Default::default()
		},
		antialiasing: true,
		..Default::default()
	})
}
