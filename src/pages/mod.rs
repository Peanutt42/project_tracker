use iced::Element;
use crate::{components::{CreateNewProjectModalMessage, CreateNewTaskModalMessage}, project_tracker::{ProjectTrackerApp, UiMessage}};

mod start_page;
pub use start_page::StartPage;
mod project_page;
pub use project_page::ProjectPage;

pub trait Page: std::fmt::Debug {
	fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage>;

	fn update_create_new_project_modal(&mut self, message: CreateNewProjectModalMessage);
	
	fn update_create_new_task_modal(&mut self, message: CreateNewTaskModalMessage);
}