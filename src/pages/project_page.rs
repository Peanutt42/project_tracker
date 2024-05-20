use iced::{widget::{column, text, text_input, row}, Alignment, Command, Element};
use once_cell::sync::Lazy;
use crate::{components::cancel_button, project_tracker::{ProjectTrackerApp, UiMessage}};
use crate::components::create_new_task_button;
use crate::styles::{HORIZONTAL_PADDING, SPACING_AMOUNT};

static TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

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
	pub fn update(&mut self, message: ProjectPageMessage) -> Command<UiMessage> {
		match message {
			ProjectPageMessage::OpenCreateNewTask => {  self.create_new_task_name = Some(String::new()); text_input::focus(TEXT_INPUT_ID.clone()) },
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
						.id(TEXT_INPUT_ID.clone())
						.on_input(|input| ProjectPageMessage::ChangeCreateNewTaskName(input).into())
						.on_submit(UiMessage::CreateTask {
							project_name: self.project_name.clone(),
							task_name: self.create_new_task_name.clone().unwrap_or(String::from("<invalid task name input>")),
						}),

					cancel_button()
						.on_press(ProjectPageMessage::CloseCreateNewTask.into())					
				]
				.align_items(Alignment::Center)
				.into()
			}
			else {
				create_new_task_button().into()
			};

			column![
				project_element,
				create_new_task_element,
			]
			.spacing(SPACING_AMOUNT)
			.padding(HORIZONTAL_PADDING)
			.into()
		}
		else {
			column![].into()
		}
	}
}