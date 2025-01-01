#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

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
	SynchronizationSetting,
};
pub use project_tracker::{DatabaseState, ProjectTrackerApp};
mod already_opened_app;
pub use already_opened_app::run_already_opened_application;
pub mod icons;
pub mod integrations;
pub mod styles;
pub mod theme_mode;
