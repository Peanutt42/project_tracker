// only enables the 'windows' subsystem when compiling in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{Settings, Application, Font, window::{self, icon}};

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
		default_font: Font::with_name("Fira Sans"),
		..Default::default()
	})
}
