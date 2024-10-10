use std::collections::HashSet;
use crate::{
	components::{close_create_new_task_modal_button, create_new_task_modal_button, horizontal_scrollable, task_tag_button}, core::{Database, ProjectId, TaskId, TaskTagId}, project_tracker::Message, styles::{card_style, description_text_editor_style, text_editor_keybindings, text_input_style_borderless, HEADING_TEXT_SIZE, LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE, SPACING_AMOUNT}
};
use iced::{
	alignment::Horizontal, font, keyboard, widget::{column, container, row, text, text_editor, text_input, Row, Space}, Element, Font, Length::Fill, Subscription
};
use iced_aw::card;
use once_cell::sync::Lazy;

static TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub enum CreateTaskModalMessage {
	Open(ProjectId),
	Close,
	CreateTask,
	ChangeTaskName(String),
	TaskDescriptionAction(text_editor::Action),
	ToggleTaskTag(TaskTagId),
}

impl From<CreateTaskModalMessage> for Message {
	fn from(value: CreateTaskModalMessage) -> Self {
		Message::CreateTaskModalMessage(value)
	}
}

pub enum CreateTaskModalAction {
	None,
	Task(iced::Task<CreateTaskModalMessage>),
	CreateTask {
		project_id: ProjectId,
		task_id: TaskId,
		task_name: String,
		task_description: String,
		task_tags: HashSet<TaskTagId>,
	},
}

pub enum CreateTaskModal {
	Opened {
		project_id: ProjectId,
		task_name: String,
		task_description: text_editor::Content,
		task_tags: HashSet<TaskTagId>,
	},
	Closed,
}

impl CreateTaskModal {
	pub fn subscription(&self) -> Subscription<Message> {
		keyboard::on_key_press(move |key, modifiers| match key.as_ref() {
			keyboard::Key::Character("n") if modifiers.command() && !modifiers.shift() => {
				Some(Message::OpenCreateTaskModal)
			},
			_ => None,
		})
	}

	#[must_use]
	pub fn update(&mut self, message: CreateTaskModalMessage) -> CreateTaskModalAction {
		match message {
			CreateTaskModalMessage::Open(project_id) => {
				*self = Self::Opened {
					project_id,
					task_name: String::new(),
					task_description: text_editor::Content::new(),
					task_tags: HashSet::new(),
				};
				CreateTaskModalAction::Task(text_input::focus(TASK_NAME_INPUT_ID.clone()))
			},
			CreateTaskModalMessage::Close => { *self = Self::Closed; CreateTaskModalAction::None },
			CreateTaskModalMessage::CreateTask => {
				let action = match self {
					CreateTaskModal::Opened { project_id, task_name, task_description, task_tags } => CreateTaskModalAction::CreateTask{
						project_id: *project_id,
						task_id: TaskId::generate(),
						task_name: task_name.clone(),
						task_description: task_description.text(),
						task_tags: task_tags.clone(),
					},
					CreateTaskModal::Closed => CreateTaskModalAction::None
				};
				*self = CreateTaskModal::Closed;
				action
			},
			CreateTaskModalMessage::ChangeTaskName(new_task_name) => {
				if let CreateTaskModal::Opened { task_name, .. } = self {
					*task_name = new_task_name;
				}
				CreateTaskModalAction::None
			}
			CreateTaskModalMessage::TaskDescriptionAction(action) => {
				if let CreateTaskModal::Opened { task_description, .. } = self {
					task_description.perform(action);
				}
				CreateTaskModalAction::None
			},
			CreateTaskModalMessage::ToggleTaskTag(task_tag_id) => {
				if let CreateTaskModal::Opened { task_tags, .. } = self {
					if task_tags.contains(&task_tag_id) {
						task_tags.remove(&task_tag_id);
					}
					else {
						task_tags.insert(task_tag_id);
					}
				}
				CreateTaskModalAction::None
			},
		}
	}

	pub fn view<'a>(&'a self, database: &'a Option<Database>) -> Option<Element<'a, CreateTaskModalMessage>> {
		match self {
			Self::Closed => None,
			Self::Opened { task_name, task_description, task_tags, project_id } => Some(
				card(
					if let Some(project) = database.as_ref().and_then(|db| db.get_project(project_id)) {
						text(format!("Create Task in {}", project.name))
					}
					else {
						text("Create Task")
					}
					.size(LARGE_TEXT_SIZE),

					column![
						if let Some(project) = database.as_ref().and_then(|db| db.get_project(project_id)) {
							let task_tags_list: Vec<Element<CreateTaskModalMessage>> = project.task_tags.iter()
								.map(|(tag_id, tag)| {
									task_tag_button(tag, task_tags.contains(&tag_id), true, true)
										.on_press(CreateTaskModalMessage::ToggleTaskTag(tag_id))
										.into()
								})
								.collect();

							if task_tags_list.is_empty() {
								Element::new(Space::new(0.0, 0.0))
							}
							else {
								horizontal_scrollable(
									Row::with_children(task_tags_list)
										.spacing(SPACING_AMOUNT)
								)
								.width(Fill)
								.into()
							}
						}
						else {
							Element::new(text("<invalid project id>"))
						},

						text_input("task name", task_name)
							.id(TASK_NAME_INPUT_ID.clone())
							.on_input(CreateTaskModalMessage::ChangeTaskName)
							.on_submit(CreateTaskModalMessage::CreateTask)
							.style(text_input_style_borderless)
							.size(HEADING_TEXT_SIZE)
							.font(Font {
								weight: font::Weight::Bold,
								..Default::default()
							}),

						Space::new(0.0, SPACING_AMOUNT),

						text("Description:"),

						text_editor(task_description)
							.on_action(CreateTaskModalMessage::TaskDescriptionAction)
							.style(description_text_editor_style)
							.key_binding(text_editor_keybindings),

						Space::new(0.0, LARGE_SPACING_AMOUNT),

						container(
							row![
								create_new_task_modal_button(),
								close_create_new_task_modal_button()
							]
							.spacing(SPACING_AMOUNT)
						)
						.width(Fill)
						.align_x(Horizontal::Right)
					],
				)
				.max_width(600.0)
				.close_size(LARGE_TEXT_SIZE)
				.on_close(CreateTaskModalMessage::Close)
				.style(card_style)
				.into(),
			),
		}
	}
}