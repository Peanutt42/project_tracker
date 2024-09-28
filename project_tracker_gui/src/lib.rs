pub mod core;
mod components;
mod pages;
mod styles;
mod project_tracker;
pub use project_tracker::ProjectTrackerApp;
mod theme_mode;
pub mod integrations;
pub mod icons;
#[cfg(feature = "hot_reload")]
mod hot_reload;