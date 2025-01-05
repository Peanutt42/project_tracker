// only enables the 'windows' subsystem when compiling in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::exit;

#[cfg(target_os = "linux")]
use iced::window::settings::PlatformSpecific;
use iced::{
	window::{self, icon},
	Size,
};
use iced_fonts::REQUIRED_FONT_BYTES;
use project_tracker::{
	icons::{APP_ICON_BYTES, BOOTSTRAP_FONT_BYTES},
	run_already_opened_application,
	styles::{FIRA_SANS_FONT, FIRA_SANS_FONT_BYTES},
	AppFlags, ProjectTrackerApp,
};
use single_instance::SingleInstance;

fn main() -> Result<(), iced::Error> {
	let instance = SingleInstance::new("ProjectTrackerInstance").unwrap();
	if !instance.is_single() {
		eprintln!("another instance is already running. closing...");
		run_already_opened_application()?;
		exit(1);
	}

	tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new()).unwrap();

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
	.run_with(|| ProjectTrackerApp::new(AppFlags::default()))
}
