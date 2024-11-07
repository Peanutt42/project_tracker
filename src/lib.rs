mod components;
pub mod core;
mod pages;
mod modals;
mod project_tracker;
pub use project_tracker::ProjectTrackerApp;
mod already_opened_app;
pub use already_opened_app::run_already_opened_application;
pub mod styles;
pub mod icons;
pub mod integrations;
pub mod theme_mode;