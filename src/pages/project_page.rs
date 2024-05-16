use iced::{widget::{column, row, text, container}, alignment::Horizontal, Element, Length};
use crate::project_tracker::UiMessage;
use crate::components::{home_button, create_new_task_button, CreateNewTaskModal, CreateNewTaskModalMessage};
use crate::pages::Page;

#[derive(Debug, Clone)]
pub struct ProjectPage {
	project_name: String,
	create_new_task_modal: CreateNewTaskModal,
}

impl ProjectPage {
	pub fn new(project_name: String) -> Self {
		Self {
			project_name,
			create_new_task_modal: CreateNewTaskModal::new(),
		}
	}
}

impl Page for ProjectPage {
	fn update_create_new_project_modal(&mut self, _message: crate::components::CreateNewProjectModalMessage) {}

	fn update_create_new_task_modal(&mut self, message: CreateNewTaskModalMessage) {
		self.create_new_task_modal.update(message);
	}

	fn view<'a>(&'a self, app: &'a crate::project_tracker::ProjectTrackerApp) -> Element<UiMessage> {
		let dark_mode = app.is_dark_mode();

		if let Some(saved_state) = &app.saved_state {
			let mut current_project = None;
			for project in saved_state.projects.iter() {
				if project.name == self.project_name {
					current_project = Some(project);
					break;
				}
			}
			let project_element = if let Some(project) = current_project {
				project.view(&self.create_new_task_modal, dark_mode)
			}
			else {
				text("Invalid Project").into()
			};
			column![
				row![
					home_button(),
					container(create_new_task_button())
						.align_x(Horizontal::Right)
						.width(Length::Fill),
				],
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