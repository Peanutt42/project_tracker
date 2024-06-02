use iced::{theme, widget::{column, container, row, text, text_input}, alignment::{Alignment, Horizontal}, Command, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{components::{cancel_button, completion_bar, partial_horizontal_seperator, create_new_task_button, task_list}, core::{Project, ProjectId, TaskFilter}, project_tracker::{ProjectTrackerApp, UiMessage}, styles::{TextInputStyle, SPACING_AMOUNT, PADDING_AMOUNT, TITLE_TEXT_SIZE}};

static TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub enum ProjectPageMessage {
	OpenCreateNewTask,
	CloseCreateNewTask,
	ChangeCreateNewTaskName(String),
	ChangeTaskFilter(TaskFilter),
}

impl From<ProjectPageMessage> for UiMessage {
	fn from(value: ProjectPageMessage) -> Self {
		UiMessage::ProjectPageMessage(value)
	}
}

#[derive(Debug, Clone)]
pub struct ProjectPage {
	pub project_id: ProjectId,
	pub create_new_task_name: Option<String>,
	task_filter: TaskFilter,
}

impl ProjectPage {
	pub fn new(project_id: ProjectId) -> Self {
		Self {
			project_id,
			create_new_task_name: None,
			task_filter: TaskFilter::All,
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
			ProjectPageMessage::ChangeTaskFilter(new_task_filter) => { self.task_filter = new_task_filter; Command::none() },
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		if let Some(database) = &app.database {
			let project_element: Element<UiMessage> = if let Some(project) = database.projects.get(&self.project_id) {
				let tasks_done = project.get_tasks_done();
				let tasks_len = project.tasks.len();
				let completion_percentage = Project::calculate_completion_percentage(tasks_done, tasks_len);

				column![
					text(&project.name).size(TITLE_TEXT_SIZE),
					completion_bar(completion_percentage),
					row![
						text(format!("{tasks_done}/{tasks_len} finished ({}%)", (completion_percentage * 100.0).round()))
							.width(Length::Fill),

						self.task_filter.view(),
					]
					.width(Length::Fill)
					.align_items(Alignment::Center),

					partial_horizontal_seperator(),

					task_list(&project.tasks, self.task_filter, self.project_id)
				]
				.spacing(SPACING_AMOUNT)
				.into()
			}
			else {
				text("<Invalid ProjectId>").into()
			};

			let create_new_task_element: Element<UiMessage> = if let Some(create_new_task_name) = &self.create_new_task_name {
				container(
					row![
						text_input("New task name", create_new_task_name)
							.id(TEXT_INPUT_ID.clone())
							.on_input(|input| ProjectPageMessage::ChangeCreateNewTaskName(input).into())
							.on_submit(UiMessage::CreateTask {
								project_id: self.project_id,
								task_name: self.create_new_task_name.clone().unwrap_or(String::from("<invalid task name input>")),
							})
							.style(theme::TextInput::Custom(Box::new(TextInputStyle))),
	
						cancel_button()
							.on_press(ProjectPageMessage::CloseCreateNewTask.into())					
					]
					.align_items(Alignment::Center)
				)
				.max_width(600.0)
				.align_x(Horizontal::Center)
				.into()
			}
			else {
				create_new_task_button().into()
			};

			column![
				project_element,
				partial_horizontal_seperator(),
				create_new_task_element,
			]
			.spacing(SPACING_AMOUNT)
			.padding(Padding::new(PADDING_AMOUNT))
			.height(Length::Fill)
			.align_items(Alignment::Center)
			.into()
		}
		else {
			column![].into()
		}
	}
}