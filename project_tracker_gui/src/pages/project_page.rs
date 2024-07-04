use iced::{alignment::{Alignment, Horizontal}, theme, widget::{button, column, container, row, text, text_input}, Command, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{
	components::{completion_bar, create_new_task_button, delete_project_button, move_project_down_button, move_project_up_button, partial_horizontal_seperator, task_list, CREATE_NEW_TASK_NAME_INPUT_ID, EDIT_TASK_NAME_INPUT_ID},
	core::{DatabaseMessage, Project, ProjectId, TaskId},
	project_tracker::{ProjectTrackerApp, UiMessage},
	styles::{HiddenSecondaryButtonStyle, TextInputStyle, PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, TITLE_TEXT_SIZE}
};

static PROJECT_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Clone, Debug)]
pub enum ProjectPageMessage {
	OpenCreateNewTask,
	CloseCreateNewTask,
	ChangeCreateNewTaskName(String),

	ShowDoneTasks(bool),

	EditProjectName,
	StopEditingProjectName,

	EditTask(TaskId),
	StopEditingTask,
}

impl From<ProjectPageMessage> for UiMessage {
	fn from(value: ProjectPageMessage) -> Self {
		UiMessage::ProjectPageMessage(value)
	}
}

#[derive(Clone, Debug)]
pub struct ProjectPage {
	pub project_id: ProjectId,
	edit_project_name: bool,
	pub create_new_task_name: Option<String>,
	task_being_edited_id: Option<TaskId>,
	show_done_tasks: bool,
}

impl ProjectPage {
	pub fn new(project_id: ProjectId) -> Self {
		Self {
			project_id,
			edit_project_name: false,
			create_new_task_name: None,
			task_being_edited_id: None,
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
					self.update(ProjectPageMessage::StopEditingTask),
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

			ProjectPageMessage::EditProjectName => { self.edit_project_name = true; text_input::focus(PROJECT_NAME_TEXT_INPUT_ID.clone()) },
			ProjectPageMessage::StopEditingProjectName => { self.edit_project_name = false; Command::none() },

			ProjectPageMessage::EditTask(task_id) => {
				self.task_being_edited_id = Some(task_id);
				Command::batch([
					text_input::focus(EDIT_TASK_NAME_INPUT_ID.clone()),
					self.update(ProjectPageMessage::CloseCreateNewTask),
				])
			},
			ProjectPageMessage::StopEditingTask => { self.task_being_edited_id = None; Command::none() },
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		if let Some(database) = &app.database {
			if let Some(project) = database.projects.get(&self.project_id) {
				let tasks_done = project.get_tasks_done();
				let tasks_len = project.tasks.len();
				let completion_percentage = Project::calculate_completion_percentage(tasks_done, tasks_len);

				let project_name : Element<UiMessage> = if self.edit_project_name {
					text_input("New project name", &project.name)
						.id(PROJECT_NAME_TEXT_INPUT_ID.clone())
						.size(TITLE_TEXT_SIZE)
						.on_input(|new_name| DatabaseMessage::ChangeProjectName{ project_id: self.project_id, new_name }.into())
						.on_submit(ProjectPageMessage::StopEditingProjectName.into())
						.style(theme::TextInput::Custom(Box::new(TextInputStyle)))
						.into()
				}
				else {
					button(
						text(&project.name).size(TITLE_TEXT_SIZE)
					)
					.on_press(ProjectPageMessage::EditProjectName.into())
					.style(theme::Button::custom(HiddenSecondaryButtonStyle))
					.into()
				};

				let order = database.projects.get_order(&self.project_id);
				let can_move_up = if let Some(order) = order { order != 0 } else { false };
				let can_move_down = if let Some(order) = order { order != database.projects.len() - 1 } else { false };

				column![
					column![
						row![
							project_name,
							container(
								row![
									move_project_up_button(self.project_id, can_move_up),
									move_project_down_button(self.project_id, can_move_down),
									delete_project_button(self.project_id),
								]
								.spacing(SMALL_SPACING_AMOUNT)
							)
							.align_x(Horizontal::Right)
							.width(Length::Fill)
						],

						completion_bar(completion_percentage),

						row![
							text(format!("{tasks_done}/{tasks_len} finished ({}%)", (completion_percentage * 100.0).round()))
								.width(Length::Fill),

							container(create_new_task_button(self.create_new_task_name.is_none()))
								.width(Length::Fill)
								.align_x(Horizontal::Right),
						]
						.width(Length::Fill)
						.align_items(Alignment::Center),

						partial_horizontal_seperator(),
					]
					.padding(Padding::new(PADDING_AMOUNT))
					.spacing(SPACING_AMOUNT),

					task_list(&project.tasks, self.project_id, self.task_being_edited_id, self.show_done_tasks, &self.create_new_task_name),
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
