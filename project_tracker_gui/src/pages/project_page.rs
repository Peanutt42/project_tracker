use std::collections::HashSet;

use iced::{alignment::{Alignment, Horizontal}, theme, widget::{button, column, container, row, scrollable, scrollable::RelativeOffset, text, text_input, Row}, Color, Command, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{
	components::{cancel_create_new_task_tag_button, color_palette, color_palette_item_button, completion_bar, create_new_label_button, create_new_task_button, delete_project_button, task_list, task_tag_button, unfocusable, CREATE_NEW_TASK_NAME_INPUT_ID, EDIT_TASK_NAME_INPUT_ID, TASK_LIST_ID},
	core::{Database, DatabaseMessage, Project, ProjectId, TaskId, TaskTag, TaskTagId},
	project_tracker::{ProjectTrackerApp, UiMessage},
	styles::{HiddenSecondaryButtonStyle, TextInputStyle, PADDING_AMOUNT, SPACING_AMOUNT, TITLE_TEXT_SIZE},
};

static PROJECT_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static TASK_TAG_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Clone, Debug)]
pub enum ProjectPageMessage {
	OpenCreateNewTask,
	CloseCreateNewTask,
	ChangeCreateNewTaskName(String),

	ShowDoneTasks(bool),

	ToggleFilterTaskTag(TaskTagId),
	OpenCreateNewTaskTag,
	CloseCreateNewTaskTag,
	ChangeCreateNewTaskTagName(String),
	ChangeCreateNewTaskTagColor(Color),
	CreateNewTaskTag,

	ShowColorPicker,
	HideColorPicker,

	EditProjectName,
	StopEditingProjectName,
	ChangeEditedProjectName(String),

	EditTask(TaskId),
	StopEditingTask,
	ChangeEditedTaskName(String),
	ToggleTaskTag(TaskTagId),

	DragTask(TaskId),
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
	filter_task_tags: HashSet<TaskTagId>,
	create_new_task_tag: Option<TaskTag>,
	pressed_task: Option<TaskId>,
	dragged_task: Option<TaskId>,
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
			filter_task_tags: HashSet::new(),
			create_new_task_tag: None,
			pressed_task: None,
			dragged_task: None,
		}
	}
}

impl ProjectPage {
	pub fn update(&mut self, message: ProjectPageMessage, database: &mut Option<Database>) -> Command<UiMessage> {
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

			ProjectPageMessage::ToggleFilterTaskTag(task_tag_id) => {
				if self.filter_task_tags.contains(&task_tag_id) {
					self.filter_task_tags.remove(&task_tag_id);
				}
				else {
					self.filter_task_tags.insert(task_tag_id);
				}
				Command::none()
			},
			ProjectPageMessage::OpenCreateNewTaskTag => {
				self.create_new_task_tag = Some(TaskTag::new(String::new(), Color::WHITE.into()));
				text_input::focus(TASK_TAG_NAME_TEXT_INPUT_ID.clone())
			},
			ProjectPageMessage::CloseCreateNewTaskTag => {
				self.create_new_task_tag = None;
				Command::none()
			},
			ProjectPageMessage::ChangeCreateNewTaskTagName(new_name) => {
				if let Some(tag) = &mut self.create_new_task_tag {
					tag.name = new_name;
				}
				Command::none()
			},
			ProjectPageMessage::ChangeCreateNewTaskTagColor(new_color) => {
				if let Some(tag) = &mut self.create_new_task_tag {
					tag.color = new_color.into();
				}
				Command::none()
			},
			ProjectPageMessage::CreateNewTaskTag => {
				if let Some(create_new_task_tag) = &self.create_new_task_tag {
					if let Some(database) = database {
						database.modify(|projects| {
							if let Some(project) = projects.get_mut(&self.project_id) {
								project.task_tags.insert(TaskTagId::generate(), create_new_task_tag.clone());
							}
						});
					}
				}
				self.update(ProjectPageMessage::CloseCreateNewTaskTag, database)
			},

			ProjectPageMessage::ShowColorPicker => { self.show_color_picker = true; Command::none() },
			ProjectPageMessage::HideColorPicker => { self.show_color_picker = false; Command::none() },

			ProjectPageMessage::EditProjectName => {
				let project_name = database.as_ref()
					.and_then(|db|
						db.projects()
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
					db.projects().get(&self.project_id)
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
			ProjectPageMessage::ToggleTaskTag(task_tag_id) => {
				if let Some((edited_task_id, _edited_task_name)) = &mut self.edited_task {
					if let Some(database) = database {
						return database.update(DatabaseMessage::ToggleTaskTag {
							project_id: self.project_id,
							task_id: *edited_task_id,
							task_tag_id
						});
					}
				}
				Command::none()
			},

			ProjectPageMessage::DragTask(task_id) => {
				self.dragged_task = Some(task_id);
				Command::none()
			},
			ProjectPageMessage::PressTask(task_id) => {
				self.pressed_task = Some(task_id);
				Command::none()
			},
			ProjectPageMessage::LeftClickReleased => {
				let command = if let Some(pressed_task) = &self.pressed_task {
					if self.dragged_task.is_none() {
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
				self.dragged_task = None;
				command
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, UiMessage> {
		if let Some(database) = &app.database {
			if let Some(project) = database.projects().get(&self.project_id) {
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
							.style(theme::TextInput::Custom(Box::new(TextInputStyle { round_left: true, round_right: false }))),

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

				let mut task_tags_list: Vec<Element<UiMessage>> = Vec::new();
				for (tag_id, tag) in project.task_tags.iter() {
					task_tags_list.push(
						task_tag_button(tag, self.filter_task_tags.contains(tag_id))
							.on_press(ProjectPageMessage::ToggleFilterTaskTag(*tag_id).into())
							.into()
					);
				}
				task_tags_list.push(
					if let Some(tag) = &self.create_new_task_tag {
						row![
							text_input("Tag name", &tag.name)
								.id(TASK_TAG_NAME_TEXT_INPUT_ID.clone())
								.on_input(|new_name| ProjectPageMessage::ChangeCreateNewTaskTagName(new_name).into())
								.on_submit(ProjectPageMessage::CreateNewTaskTag.into())
								.style(theme::TextInput::Custom(Box::new(TextInputStyle { round_left: true, round_right: false }))),

							cancel_create_new_task_tag_button()
						]
        				.into()
					}
					else {
						create_new_label_button().into()
					}
				);

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
								delete_project_button(self.project_id, &project.name)
							)
							.align_x(Horizontal::Right)
						]
						.align_items(Alignment::Center)
						.spacing(SPACING_AMOUNT),

						row![
							text(format!("{tasks_done}/{tasks_len} finished ({}%)", (completion_percentage * 100.0).round()))
								.width(Length::Fill),

							container(create_new_task_button(self.create_new_task_name.is_none()))
								.width(Length::Fill)
								.align_x(Horizontal::Right),
						]
						.width(Length::Fill)
						.align_items(Alignment::Center),

						row![
							text("Tags:"),

							Row::with_children(task_tags_list)
								.spacing(SPACING_AMOUNT),
						]
						.spacing(SPACING_AMOUNT)
						.align_items(Alignment::Center),

						completion_bar(completion_percentage),
					]
					.padding(Padding::new(PADDING_AMOUNT))
					.spacing(SPACING_AMOUNT),

					task_list(project_id, project, &self.edited_task, self.dragged_task, app.sidebar_page.task_being_task_hovered, self.show_done_tasks, &self.filter_task_tags, &self.create_new_task_name),
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
