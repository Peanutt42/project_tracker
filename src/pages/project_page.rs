use iced::{theme, widget::{button, column, row, text, text_input}, Alignment, Command, Element, Length};
use once_cell::sync::Lazy;
use crate::{components::{cancel_button, completion_bar, partial_horizontal_seperator, task_list}, project::{Project, TaskFilter}, project_tracker::{ProjectTrackerApp, UiMessage}, styles::{TextInputStyle, SPACING_AMOUNT, TITLE_TEXT_SIZE}};
use crate::components::create_new_task_button;
use crate::styles::{HORIZONTAL_PADDING, LARGE_SPACING_AMOUNT};

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
	pub project_name: String,
	pub create_new_task_name: Option<String>,
	task_filter: TaskFilter,
}

impl ProjectPage {
	pub fn new(project_name: String) -> Self {
		Self {
			project_name,
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
		if let Some(saved_state) = &app.saved_state {
			let mut current_project = None;
			for project in saved_state.projects.iter() {
				if project.name == self.project_name {
					current_project = Some(project);
					break;
				}
			}
			
			let filter_button = |label, filter, current_filter| {
				button(text(label))
					//.padding(8)
					.style(if filter == current_filter {
						theme::Button::Primary
					} else {
						theme::Button::Text
					})
					.on_press(ProjectPageMessage::ChangeTaskFilter(filter).into())
			};

			let project_element: Element<UiMessage> = if let Some(project) = current_project {
				let tasks_done = project.get_tasks_done();
				let tasks_len = project.tasks.len();
				let completion_percentage = Project::calculate_completion_percentage(tasks_done, tasks_len);

				column![
					text(&project.name).size(TITLE_TEXT_SIZE),
					completion_bar(completion_percentage),
					row![
						text(format!("{tasks_done}/{tasks_len} finished ({}%)", (completion_percentage * 100.0).round()))
							.width(Length::Fill),

						row![
							filter_button("All", TaskFilter::All, self.task_filter),
							filter_button("Todo", TaskFilter::Todo, self.task_filter),
							filter_button("Done", TaskFilter::Done, self.task_filter)
						]
						.width(Length::Shrink)
						.spacing(SPACING_AMOUNT),
					]
					.width(Length::Fill)
					.align_items(Alignment::Center),

					partial_horizontal_seperator(1.0),

					task_list(&project.tasks, self.task_filter, &project.name)
				]
				.spacing(SPACING_AMOUNT)
				.into()
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
						})
						.style(theme::TextInput::Custom(Box::new(TextInputStyle))),

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
				partial_horizontal_seperator(1.0),
				create_new_task_element,
			]
			.spacing(LARGE_SPACING_AMOUNT)
			.padding(HORIZONTAL_PADDING)
			.into()
		}
		else {
			column![].into()
		}
	}
}