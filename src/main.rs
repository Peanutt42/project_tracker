// only enables the 'windows' subsystem when compiling in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use project_tracker::{run_already_opened_application, run_project_tracker_app, AppFlags};
use single_instance::SingleInstance;
use std::process::exit;

fn main() -> iced::Result {
	let instance = SingleInstance::new("ProjectTrackerInstance").unwrap();
	if !instance.is_single() {
		eprintln!("another instance is already running. closing...");
		run_already_opened_application()?;
		exit(1);
	}

	tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new()).unwrap();

	run_project_tracker_app(AppFlags::default())
}
