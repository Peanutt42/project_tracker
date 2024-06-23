use iced::{alignment::Alignment, widget::{column, row, text, text_input}, Command, Element, Length, Padding};
use iced_aw::{floating_element, floating_element::Anchor};
use crate::{components::{completion_bar, create_new_task_button, partial_horizontal_seperator, task_list, EDIT_TASK_NAME_INPUT_ID, CREATE_NEW_TASK_NAME_INPUT_ID}, core::{Project, ProjectId, TaskId}, project_tracker::{ProjectTrackerApp, UiMessage}, styles::{LARGE_PADDING_AMOUNT, PADDING_AMOUNT, SPACING_AMOUNT, TITLE_TEXT_SIZE}};

#[derive(Clone, Debug)]
pub enum ProjectPageMessage {
	OpenCreateNewTask,
	CloseCreateNewTask,
	ChangeCreateNewTaskName(String),

	ShowDoneTasks(bool),

	EditTask(TaskId),
	StopEditing,

	HoveringTask(TaskId),
	StoppedHoveringTask,
}

impl From<ProjectPageMessage> for UiMessage {
	fn from(value: ProjectPageMessage) -> Self {
		UiMessage::ProjectPageMessage(value)
	}
}

#[derive(Clone, Debug)]
pub struct ProjectPage {
	pub project_id: ProjectId,
	pub create_new_task_name: Option<String>,
	task_being_edited_id: Option<TaskId>,
	hovered_task: Option<TaskId>,
	show_done_tasks: bool,
}

impl ProjectPage {
	pub fn new(project_id: ProjectId) -> Self {
		Self {
			project_id,
			create_new_task_name: None,
			task_being_edited_id: None,
			hovered_task: None,
			show_done_tasks: false,
		}
	}
}

impl ProjectPage {
	pub fn update(&mut self, message: ProjectPageMessage) -> Command<UiMessage> {
		match message {
			ProjectPageMessage::OpenCreateNewTask => {
				self.create_new_task_name = Some(String::new());
				Command::batch([
					text_input::focus(CREATE_NEW_TASK_NAME_INPUT_ID.clone()),
					self.update(ProjectPageMessage::StopEditing),
				])
			},
			ProjectPageMessage::CloseCreateNewTask => { self.create_new_task_name = None; Command::none() },
			ProjectPageMessage::ChangeCreateNewTaskName(new_task_name) => {
				if let Some(create_new_task_name) = &mut self.create_new_task_name {
					*create_new_task_name = new_task_name;
				}
				Command::none()
			},
			ProjectPageMessage::ShowDoneTasks(show) => { self.show_done_tasks = show; Command::none() },
			ProjectPageMessage::EditTask(task_id) => {
				self.task_being_edited_id = Some(task_id);
				Command::batch([
					text_input::focus(EDIT_TASK_NAME_INPUT_ID.clone()),
					self.update(ProjectPageMessage::CloseCreateNewTask),
				])
			},
			ProjectPageMessage::StopEditing => { self.task_being_edited_id = None; Command::none() },
			ProjectPageMessage::HoveringTask(task_id) => { self.hovered_task = Some(task_id); Command::none() },
			ProjectPageMessage::StoppedHoveringTask => { self.hovered_task = None; Command::none() },
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		if let Some(database) = &app.database {
			if let Some(project) = database.projects.get(&self.project_id) {
				let tasks_done = project.get_tasks_done();
				let tasks_len = project.tasks.len();
				let completion_percentage = Project::calculate_completion_percentage(tasks_done, tasks_len);

				column![
					column![
						text(&project.name).size(TITLE_TEXT_SIZE),
						completion_bar(completion_percentage),
						row![
							text(format!("{tasks_done}/{tasks_len} finished ({}%)", (completion_percentage * 100.0).round()))
								.width(Length::Fill),
						]
						.width(Length::Fill)
						.align_items(Alignment::Center),

						partial_horizontal_seperator(),
					]
					.padding(Padding::new(PADDING_AMOUNT))
					.spacing(SPACING_AMOUNT),

					floating_element(
						task_list(&project.tasks, self.project_id, self.hovered_task, self.task_being_edited_id, self.show_done_tasks, &self.create_new_task_name),
						create_new_task_button(self.create_new_task_name.is_none())
					)
					.anchor(Anchor::SouthEast)
					.offset(LARGE_PADDING_AMOUNT),
				]
				.spacing(SPACING_AMOUNT)
				.width(Length::Fill)
				.height(Length::Fill)
				.into()
			}
			else {
				text("<Invalid ProjectId>").into()
			}
		}
		else {
			column![].into()
		}
	}
}
