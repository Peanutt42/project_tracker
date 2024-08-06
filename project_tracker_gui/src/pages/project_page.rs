use std::collections::BTreeSet;
use iced::{alignment::{Alignment, Horizontal}, theme, widget::{button, column, container, row, scrollable, scrollable::RelativeOffset, text, text_editor, text_input, Row}, Color, Command, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{
	components::{color_palette, color_palette_item_button, completion_bar, create_new_task_button, delete_project_button, manage_task_tags_button, task_list, task_tag_button, unfocusable, CREATE_NEW_TASK_NAME_INPUT_ID, TASK_LIST_ID},
	core::{generate_task_id, Database, DatabaseMessage, Project, ProjectId, TaskId, TaskTagId},
	project_tracker::{ProjectTrackerApp, UiMessage},
	styles::{scrollable_horizontal_direction, HiddenSecondaryButtonStyle, ScrollableStyle, TextInputStyle, PADDING_AMOUNT, SCROLLBAR_WIDTH, SMALL_PADDING_AMOUNT, SPACING_AMOUNT, TITLE_TEXT_SIZE},
};

static PROJECT_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Clone, Debug)]
pub enum ProjectPageMessage {
	OpenCreateNewTask,
	CloseCreateNewTask,
	ChangeCreateNewTaskName(String),
	ToggleCreateNewTaskTag(TaskTagId),
	CreateNewTask,

	ShowDoneTasks(bool),

	ToggleFilterTaskTag(TaskTagId),

	ShowColorPicker,
	HideColorPicker,

	EditProjectName,
	StopEditingProjectName,
	ChangeEditedProjectName(String),
	ChangeProjectName,

	EditTask(TaskId),
	StopEditingTask,
	TaskNameAction(text_editor::Action),
	ToggleTaskTag(TaskTagId),
	ClearTaskNeededTime(TaskId),
	InvalidNeededTimeInput,

	DragTask(TaskId),
	PressTask(TaskId),
	LeftClickReleased,
}

impl From<ProjectPageMessage> for UiMessage {
	fn from(value: ProjectPageMessage) -> Self {
		UiMessage::ProjectPageMessage(value)
	}
}

#[derive(Debug)]
pub struct ProjectPage {
	pub project_id: ProjectId,
	edited_project_name: Option<String>,
	pub create_new_task: Option<(String, BTreeSet<TaskTagId>)>,
	edited_task: Option<(TaskId, text_editor::Content)>, // task_id, new_name
	show_done_tasks: bool,
	show_color_picker: bool,
	filter_task_tags: BTreeSet<TaskTagId>,
	pressed_task: Option<TaskId>,
	dragged_task: Option<TaskId>,
}

impl ProjectPage {
	pub fn new(project_id: ProjectId) -> Self {
		Self {
			project_id,
			edited_project_name: None,
			create_new_task: None,
			edited_task: None,
			show_done_tasks: false,
			show_color_picker: false,
			filter_task_tags: BTreeSet::new(),
			pressed_task: None,
			dragged_task: None,
		}
	}
}

impl ProjectPage {
	pub fn update(&mut self, message: ProjectPageMessage, database: &mut Option<Database>) -> Command<UiMessage> {
		match message {
			ProjectPageMessage::OpenCreateNewTask => {
				self.create_new_task = Some((String::new(), BTreeSet::new()));
				Command::batch([
					text_input::focus(CREATE_NEW_TASK_NAME_INPUT_ID.clone()),
					scrollable::snap_to(TASK_LIST_ID.clone(), RelativeOffset::END),
					self.update(ProjectPageMessage::StopEditingTask, database),
				])
			},
			ProjectPageMessage::CloseCreateNewTask => { self.create_new_task = None; Command::none() },
			ProjectPageMessage::ChangeCreateNewTaskName(new_task_name) => {
				if let Some((create_new_task_name, _create_new_task_tags)) = &mut self.create_new_task {
					*create_new_task_name = new_task_name;
				}
				Command::none()
			},
			ProjectPageMessage::ToggleCreateNewTaskTag(tag_id) => {
				if let Some((_create_new_task_name, create_new_task_tags)) = &mut self.create_new_task {
					if create_new_task_tags.contains(&tag_id) {
						create_new_task_tags.remove(&tag_id);
					}
					else {
						create_new_task_tags.insert(tag_id);
					}
				}
				Command::none()
			},
			ProjectPageMessage::CreateNewTask => {
				if let Some((create_new_task_name, create_new_task_tags)) = &mut self.create_new_task {
					if let Some(db) = database {
						return Command::batch([
							db.update(DatabaseMessage::CreateTask {
								project_id: self.project_id,
								task_id: generate_task_id(),
								task_name: std::mem::take(create_new_task_name),
								task_tags: std::mem::take(create_new_task_tags),
							}),
							self.update(ProjectPageMessage::CloseCreateNewTask, database)
						]);
					}
				}
				self.update(ProjectPageMessage::CloseCreateNewTask, database)
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
			ProjectPageMessage::ChangeProjectName => {
				if let Some(db) = database {
					if let Some(edited_project_name) = &mut self.edited_project_name {
						return Command::batch([
							db.update(DatabaseMessage::ChangeProjectName {
								project_id: self.project_id,
								new_name: std::mem::take(edited_project_name)
							}),
							self.update(ProjectPageMessage::StopEditingProjectName, database)
						]);
					}
				}
				self.update(ProjectPageMessage::StopEditingProjectName, database)
			},

			ProjectPageMessage::EditTask(task_id) => {
				let task_name = database.as_ref().and_then(|db| {
					db.projects().get(&self.project_id)
						.and_then(|project|
							project.tasks
								.get(&task_id)
								.map(|task| task.name.clone())
						)
				}).unwrap_or_default();
				self.edited_task = Some((task_id, text_editor::Content::with_text(&task_name)));
				self.update(ProjectPageMessage::CloseCreateNewTask, database)
			},
			ProjectPageMessage::StopEditingTask => { self.edited_task = None; Command::none() },
			ProjectPageMessage::TaskNameAction(action) => {
				if let Some((edited_task_id, edited_task_name)) = &mut self.edited_task {
					let is_edit = action.is_edit();
					edited_task_name.perform(action);
					if is_edit {
						if let Some(database) = database {
							return database.update(DatabaseMessage::ChangeTaskName {
								project_id: self.project_id,
								task_id: *edited_task_id,
								new_task_name: edited_task_name.text()
							});
						}
					}
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
			ProjectPageMessage::ClearTaskNeededTime(task_id) => {
				if let Some(database) = database {
					database.update(DatabaseMessage::ChangeTaskNeededTime {
						project_id: self.project_id,
						task_id,
						new_needed_time_minutes: None,
					})
				}
				else {
					Command::none()
				}
			}
			ProjectPageMessage::InvalidNeededTimeInput => Command::none(),

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
							.on_submit(ProjectPageMessage::ChangeProjectName.into())
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
						ProjectPageMessage::HideColorPicker.into()
					}
					else {
						ProjectPageMessage::ShowColorPicker.into()
					}
				);

				let mut task_tags_list: Vec<Element<UiMessage>> = Vec::new();
				for (tag_id, tag) in project.task_tags.iter() {
					task_tags_list.push(
						task_tag_button(tag, self.filter_task_tags.contains(&tag_id), true)
							.on_press(ProjectPageMessage::ToggleFilterTaskTag(tag_id).into())
							.into()
					);
				}

				let delete_project_button_element: Element<UiMessage> = delete_project_button(self.project_id, &project.name).into();

				let tags_bottom_scrollbar_padding = Padding { bottom: SMALL_PADDING_AMOUNT + SCROLLBAR_WIDTH, ..Padding::ZERO };

				column![
					column![
						column![
							row![
								show_color_picker_button,
								project_name,
								if self.edited_project_name.is_some() {
									delete_project_button_element
								}
								else {
									container(
										delete_project_button_element
									)
									.width(Length::Fill)
									.align_x(Horizontal::Right)
									.into()
								}
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

						row![
							text(format!("{tasks_done}/{tasks_len} finished ({}%)", (completion_percentage * 100.0).round()))
								.width(Length::Fill),

							container(create_new_task_button(self.create_new_task.is_none()))
								.width(Length::Fill)
								.align_x(Horizontal::Right),
						]
						.width(Length::Fill)
						.align_items(Alignment::Center),

						row![
							container(text("Tags:"))
								.padding(tags_bottom_scrollbar_padding),

							scrollable(
								Row::with_children(task_tags_list)
									.spacing(SPACING_AMOUNT)
									.padding(Padding { bottom: SMALL_PADDING_AMOUNT + SCROLLBAR_WIDTH, ..Padding::ZERO })
							)
							.width(Length::Fill)
							.direction(scrollable_horizontal_direction())
							.style(theme::Scrollable::custom(ScrollableStyle)),

							container(manage_task_tags_button(self.project_id))
								.padding(tags_bottom_scrollbar_padding),
						]
						.spacing(SPACING_AMOUNT)
						.align_items(Alignment::Center),

						completion_bar(completion_percentage),
					]
					.padding(Padding::new(PADDING_AMOUNT))
					.spacing(SPACING_AMOUNT),

					task_list(project_id, project, &self.edited_task, self.dragged_task, app.sidebar_page.task_being_task_hovered, self.show_done_tasks, &self.filter_task_tags, &self.create_new_task),
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
