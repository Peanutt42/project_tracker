use std::collections::HashSet;
use crate::{
	components::{close_create_new_task_modal_button, create_new_task_modal_button, horizontal_scrollable, task_tag_button}, core::{Database, ProjectId, TaskId, TaskTagId}, project_tracker::Message, styles::{card_style, text_editor_keybindings, text_editor_style, LARGE_TEXT_SIZE, SPACING_AMOUNT}
};
use iced::{
	keyboard, widget::{column, row, text, text_editor, text_editor::Action, Row, Space}, Element, Length::Fill, Subscription
};
use iced_aw::card;

#[derive(Debug, Clone)]
pub enum CreateTaskModalMessage {
	Open(ProjectId),
	Close,
	CreateTask,
	TaskNameAction(Action),
	ToggleTaskTag(TaskTagId),
}

impl From<CreateTaskModalMessage> for Message {
	fn from(value: CreateTaskModalMessage) -> Self {
		Message::CreateTaskModalMessage(value)
	}
}

#[derive(Debug, Clone)]
pub enum CreateTaskModalAction {
	None,
	CreateTask {
		project_id: ProjectId,
		task_id: TaskId,
		task_name: String,
		task_tags: HashSet<TaskTagId>,
	},
}

pub enum CreateTaskModal {
	Opened {
		project_id: ProjectId,
		new_task_name: text_editor::Content,
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
					new_task_name: text_editor::Content::new(),
					task_tags: HashSet::new(),
				};
				CreateTaskModalAction::None
			},
			CreateTaskModalMessage::Close => { *self = Self::Closed; CreateTaskModalAction::None },
			CreateTaskModalMessage::CreateTask => {
				let action = match self {
					CreateTaskModal::Opened { project_id, new_task_name, task_tags } => CreateTaskModalAction::CreateTask{
						project_id: *project_id,
						task_id: TaskId::generate(),
						task_name: new_task_name.text(),
						task_tags: task_tags.clone(),
					},
					CreateTaskModal::Closed => CreateTaskModalAction::None
				};
				*self = CreateTaskModal::Closed;
				action
			},
			CreateTaskModalMessage::TaskNameAction(action) => {
				if let CreateTaskModal::Opened { new_task_name, .. } = self {
					new_task_name.perform(action);
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
			Self::Opened { new_task_name, task_tags, project_id } => Some(
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

							horizontal_scrollable(
								Row::with_children(task_tags_list)
									.spacing(SPACING_AMOUNT)
							)
							.width(Fill)
							.into()
						}
						else {
							Element::new(text("<invalid project id>"))
						},

						text_editor(new_task_name)
							.on_action(CreateTaskModalMessage::TaskNameAction)
							.style(move |t, s| {
								text_editor_style(t, s, true, true, true, true)
							})
							.key_binding(text_editor_keybindings),

						Space::new(0.0, SPACING_AMOUNT),

						row![
							create_new_task_modal_button(),
							close_create_new_task_modal_button()
						]
						.spacing(SPACING_AMOUNT)
					],
				)
				.max_width(600.0)
				.style(card_style)
				.into(),
			),
		}
	}
}