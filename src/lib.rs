#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

pub use project_tracker_core::*;

mod components;
pub mod core;
mod modals;
mod pages;
mod preferences;
mod project_tracker;
pub use preferences::{
	DateFormatting, LoadPreferencesError, LoadPreferencesResult, OptionalPreference,
	PreferenceAction, PreferenceMessage, Preferences, SerializedContentPage, StopwatchProgress,
};
pub use project_tracker::{AppFlags, DatabaseState, ProjectTrackerApp};
mod already_opened_app;
pub use already_opened_app::run_already_opened_application;
pub mod icons;
pub mod integrations;
pub mod styles;
pub mod synchronization;
pub mod theme_mode;

pub fn run_project_tracker_app(flags: AppFlags) -> iced::Result {
	use crate::icons::{APP_ICON_BYTES, BOOTSTRAP_FONT_BYTES};
	use crate::styles::{INTER_FONT, INTER_FONT_BYTES, JET_BRAINS_MONO_FONT_BYTES};
	#[cfg(target_os = "linux")]
	use iced::window::settings::PlatformSpecific;
	use iced::{
		window::{self, icon},
		Size,
	};
	use iced_fonts::REQUIRED_FONT_BYTES;

	iced::application(
		ProjectTrackerApp::title,
		ProjectTrackerApp::update,
		ProjectTrackerApp::view,
	)
	.theme(ProjectTrackerApp::theme)
	.subscription(ProjectTrackerApp::subscription)
	.font(BOOTSTRAP_FONT_BYTES)
	.font(REQUIRED_FONT_BYTES)
	.font(JET_BRAINS_MONO_FONT_BYTES)
	.font(INTER_FONT_BYTES)
	.default_font(INTER_FONT)
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
	.run_with(|| ProjectTrackerApp::new(flags))
}
