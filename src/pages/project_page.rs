use iced::{widget::{column, text, container}, alignment::Horizontal, Element, Length};
use crate::project_tracker::UiMessage;
use crate::components::create_new_task_button;

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
				container(create_new_task_button())
						.align_x(Horizontal::Right)
						.width(Length::Fill),
				project_element,
			]
			.spacing(10)
			.into()
		}
		else {
			column![].into()
		}
	}
}