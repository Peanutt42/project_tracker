use iced::{widget::{column, text, text_input, row, button}, Command, Element};
use crate::{project_tracker::{ProjectTrackerApp, UiMessage}, saved_state::SavedState};
use crate::components::create_new_task_button;
use crate::styles::SPACING_AMOUNT;

#[derive(Debug, Clone)]
pub enum ProjectPageMessage {
	OpenCreateNewTask,
	CloseCreateNewTask,
	ChangeCreateNewTaskName(String),
}

impl From<ProjectPageMessage> for UiMessage {
	fn from(value: ProjectPageMessage) -> Self {
		UiMessage::ProjectPageMessage(value)
	}
}

#[derive(Debug, Clone)]
pub struct ProjectPage {
	pub project_name: String,
	pub create_new_task_name: Option<String>,
}

impl ProjectPage {
	pub fn new(project_name: String) -> Self {
		Self {
			project_name,
			create_new_task_name: None,
		}
	}
}

impl ProjectPage {
	pub fn update<'a>(&'a mut self, message: ProjectPageMessage, mut saved_state: &'a mut Option<SavedState>) -> Command<UiMessage> {
		match message {
			ProjectPageMessage::OpenCreateNewTask => {  self.create_new_task_name = Some(String::new()); Command::none() },
			ProjectPageMessage::CloseCreateNewTask => { self.create_new_task_name = None; Command::none() },
			ProjectPageMessage::ChangeCreateNewTaskName(new_task_name) => {
				if let Some(create_new_task_name) = &mut self.create_new_task_name {
					*create_new_task_name = new_task_name;
				}
				Command::none()
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
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

			let create_new_task_element: Element<UiMessage> = if let Some(create_new_task_name) = &self.create_new_task_name {
				row![
					text_input("New task name", create_new_task_name)
					.on_input(|input| ProjectPageMessage::ChangeCreateNewTaskName(input).into())
					.on_submit(UiMessage::CreateTask {
						project_name: self.project_name.clone(),
						task_name: self.create_new_task_name.clone().unwrap_or(String::from("<invalid task name input>")),
					}),

					button(text("X"))
						.on_press(ProjectPageMessage::CloseCreateNewTask.into())					
				].into()
			}
			else {
				create_new_task_button().into()
			};

			column![
				project_element,
				create_new_task_element,
			]
			.spacing(SPACING_AMOUNT)
			.into()
		}
		else {
			column![].into()
		}
	}
}