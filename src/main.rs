use iced::{Settings, Application, window};

mod project;
mod task;
mod project_tracker;
use project_tracker::ProjectTrackerApp;
mod page;
mod saved_state;
mod window_icon;
mod components;

fn main() -> Result<(), iced::Error> {
	let icon = include_icon!("../assets/icon-handdrawn.png");

	ProjectTrackerApp::run(Settings {
		window: window::Settings {
			icon,
			..Default::default()
		},
		..Default::default()
	})
}
