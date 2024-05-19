use iced::{widget::{column, text}, Element};
use crate::project_tracker::UiMessage;
use crate::components::create_new_task_button;
use crate::styles::SPACING_AMOUNT;

#[derive(Debug, Clone)]
pub struct ProjectPage {
	pub project_name: String,
}

impl ProjectPage {
	pub fn new(project_name: String) -> Self {
		Self {
			project_name,
		}
	}
}

impl ProjectPage {
	pub fn view<'a>(&'a self, app: &'a crate::project_tracker::ProjectTrackerApp) -> Element<UiMessage> {
		if let Some(saved_state) = &app.saved_state {
			let mut current_project = None;
			for project in saved_state.projects.iter() {
				if project.name == self.project_name {
					current_project = Some(project);
					break;
				}
			}
			let project_element = if let Some(project) = current_project {
				project.view()
			}
			else {
				text("Invalid Project").into()
			};
			column![
				project_element,
				create_new_task_button(),
			]
			.spacing(SPACING_AMOUNT)
			.into()
		}
		else {
			column![].into()
		}
	}
}