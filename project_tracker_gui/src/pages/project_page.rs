use iced::{alignment::{Alignment, Horizontal}, theme, widget::{button, column, container, row, scrollable::{self, RelativeOffset}, text, text_input}, Color, Command, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{
	components::{color_palette, color_palette_item_button, completion_bar, create_new_task_button, delete_project_button, move_project_down_button, move_project_up_button, partial_horizontal_seperator, task_list, unfocusable, CREATE_NEW_TASK_NAME_INPUT_ID, EDIT_TASK_NAME_INPUT_ID, TASK_LIST_ID},
	core::{Database, DatabaseMessage, Project, ProjectId, TaskId},
	project_tracker::{ProjectTrackerApp, UiMessage},
	styles::{HiddenSecondaryButtonStyle, TextInputStyle, PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, TITLE_TEXT_SIZE},
};

static PROJECT_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Clone, Debug)]
pub enum ProjectPageMessage {
	OpenCreateNewTask,
	CloseCreateNewTask,
	ChangeCreateNewTaskName(String),

	ShowDoneTasks(bool),

	ShowColorPicker,
	HideColorPicker,

	EditProjectName,
	StopEditingProjectName,
	ChangeEditedProjectName(String),

	EditTask(TaskId),
	StopEditingTask,
	ChangeEditedTaskName(String),

	DragTask,
	PressTask(TaskId),
	LeftClickReleased,
}

impl From<ProjectPageMessage> for UiMessage {
	fn from(value: ProjectPageMessage) -> Self {
		UiMessage::ProjectPageMessage(value)
	}
}

#[derive(Clone, Debug)]
pub struct ProjectPage {
	pub project_id: ProjectId,
	edited_project_name: Option<String>,
	pub create_new_task_name: Option<String>,
	edited_task: Option<(TaskId, String)>, // task_id, new_name
	show_done_tasks: bool,
	show_color_picker: bool,
	pressed_task: Option<TaskId>,
	task_was_dragged: bool,
}

impl ProjectPage {
	pub fn new(project_id: ProjectId) -> Self {
		Self {
			project_id,
			edited_project_name: None,
			create_new_task_name: None,
			edited_task: None,
			show_done_tasks: false,
			show_color_picker: false,
			pressed_task: None,
			task_was_dragged: false,
		}
	}
}

impl ProjectPage {
	pub fn update(&mut self, message: ProjectPageMessage, database: &Option<Database>) -> Command<UiMessage> {
		match message {
			ProjectPageMessage::OpenCreateNewTask => {
				self.create_new_task_name = Some(String::new());
				Command::batch([
					text_input::focus(CREATE_NEW_TASK_NAME_INPUT_ID.clone()),
					scrollable::snap_to(TASK_LIST_ID.clone(), RelativeOffset::END),
					self.update(ProjectPageMessage::StopEditingTask, database),
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

			ProjectPageMessage::ShowColorPicker => { self.show_color_picker = true; Command::none() },
			ProjectPageMessage::HideColorPicker => { self.show_color_picker = false; Command::none() },

			ProjectPageMessage::EditProjectName => {
				let project_name = database.as_ref()
					.and_then(|db|
						db.projects
							.get(&self.project_id)
							.map(|project| project.name.clone())
					)
					.unwrap_or_default();
				self.edited_project_name = Some(project_name);
				text_input::focus(PROJECT_NAME_TEXT_INPUT_ID.clone())
			},
			ProjectPageMessage::ChangeEditedProjectName(edited_name) => { self.edited_project_name = Some(edited_name); Command::none() },
			ProjectPageMessage::StopEditingProjectName => { self.edited_project_name = None; Command::none() },

			ProjectPageMessage::EditTask(task_id) => {
				let task_name = database.as_ref().and_then(|db| {
					db.projects.get(&self.project_id)
						.and_then(|project|
							project.tasks
								.get(&task_id)
								.map(|task| task.name.clone())
						)
				}).unwrap_or_default();
				self.edited_task = Some((task_id, task_name));
				Command::batch([
					text_input::focus(EDIT_TASK_NAME_INPUT_ID.clone()),
					self.update(ProjectPageMessage::CloseCreateNewTask, database),
				])
			},
			ProjectPageMessage::StopEditingTask => { self.edited_task = None; Command::none() },
			ProjectPageMessage::ChangeEditedTaskName(edited_name) => {
				if let Some((_edited_task_id, edited_task_name)) = &mut self.edited_task {
					*edited_task_name = edited_name;
				}
				Command::none()
			},

			ProjectPageMessage::DragTask => {
				self.task_was_dragged = true;
				Command::none()
			},
			ProjectPageMessage::PressTask(task_id) => {
				self.pressed_task = Some(task_id);
				Command::none()
			},
			ProjectPageMessage::LeftClickReleased => {
				let command = if let Some(pressed_task) = &self.pressed_task {
					if !self.task_was_dragged {
						self.update(ProjectPageMessage::EditTask(*pressed_task), database)
					}
					else {
						Command::none()
					}
				}
				else {
					Command::none()
				};
				self.pressed_task = None;
				self.task_was_dragged = false;
				command
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, UiMessage> {
		if let Some(database) = &app.database {
			if let Some(project) = database.projects.get(&self.project_id) {
				let tasks_done = project.get_tasks_done();
				let tasks_len = project.tasks.len();
				let completion_percentage = Project::calculate_completion_percentage(tasks_done, tasks_len);

				let project_name : Element<UiMessage> = if let Some(edited_project_name) = &self.edited_project_name {
					unfocusable(
						text_input("New project name", edited_project_name)
							.id(PROJECT_NAME_TEXT_INPUT_ID.clone())
							.size(TITLE_TEXT_SIZE)
							.on_input(|edited_name| ProjectPageMessage::ChangeEditedProjectName(edited_name).into())
							.on_submit(DatabaseMessage::ChangeProjectName { project_id: self.project_id, new_name: edited_project_name.clone() }.into())
							.style(theme::TextInput::Custom(Box::new(TextInputStyle))),

						ProjectPageMessage::StopEditingProjectName.into()
					)
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

				let project_id = self.project_id;

				let show_color_picker_button = color_palette_item_button(
					project.color.into(),
					false,
					if self.show_color_picker {
						None
					}
					else {
						Some(ProjectPageMessage::ShowColorPicker.into())
					});

				let order = database.projects.get_order(&self.project_id);
				let can_move_up = if let Some(order) = order { order != 0 } else { false };
				let can_move_down = if let Some(order) = order { order != database.projects.len() - 1 } else { false };

				column![
					column![
						row![
							column![
								row![
									show_color_picker_button,
									project_name
								]
								.align_items(Alignment::Center)
							]
							.push_maybe(if self.show_color_picker {
								Some(color_palette(project.color.into(), move |c: Color| DatabaseMessage::ChangeProjectColor{ project_id, new_color: c.into() }.into()))
							}
							else {
								None
							})
							.width(Length::Fill),

							container(
								row![
									move_project_up_button(self.project_id, can_move_up),
									move_project_down_button(self.project_id, can_move_down),
									delete_project_button(self.project_id, &project.name),
								]
								.spacing(SMALL_SPACING_AMOUNT)
							)
							.align_x(Horizontal::Right)
						]
						.spacing(SPACING_AMOUNT),

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

					task_list(&project.tasks, self.project_id, &project.name, &self.edited_task, self.show_done_tasks, &self.create_new_task_name),
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
