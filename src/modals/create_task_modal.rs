use std::collections::HashSet;
use crate::{
	components::{add_due_date_button, clear_task_due_date_button, clear_task_needed_time_button, close_create_new_task_modal_button, create_new_task_modal_button, edit_due_date_button, edit_task_needed_time_button, horizontal_scrollable, task_tag_button, unfocusable, vertical_scrollable, SCROLLBAR_WIDTH}, core::{Database, DatabaseMessage, OptionalPreference, Preferences, ProjectId, SerializableDate, TaskId, TaskTagId}, project_tracker::Message, styles::{card_style, description_text_editor_style, on_number_input, text_editor_keybindings, text_input_style, text_input_style_borderless, unindent_text, HEADING_TEXT_SIZE, LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE, SMALL_PADDING_AMOUNT, SPACING_AMOUNT}
};
use iced::{
	font, keyboard, widget::{column, container, row, text, text_editor, text_input, Row, Space}, Element, Font, Length::{Fill, Fixed}, Padding, Subscription
};
use iced_aw::{card, date_picker};
use once_cell::sync::Lazy;

static TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static EDIT_NEEDED_TIME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub enum CreateTaskModalMessage {
	Open(ProjectId),
	Close,
	CreateTask,
	ChangeTaskName(String),
	TaskDescriptionAction(text_editor::Action),
	UnindentDescription,
	ToggleTaskTag(TaskTagId),
	ChangeNeededTimeInput(Option<usize>),
	EditNeededTime,
	StopEditingNeededTime,
	InvalidNeededTimeInput,
	EditDueDate,
	StopEditingDueDate,
	ChangeDueDate(SerializableDate),
	ClearDueDate,
}

impl From<CreateTaskModalMessage> for Message {
	fn from(value: CreateTaskModalMessage) -> Self {
		Message::CreateTaskModalMessage(value)
	}
}

pub enum CreateTaskModalAction {
	None,
	Task(iced::Task<CreateTaskModalMessage>),
	DatabaseMessage(DatabaseMessage),
}

impl From<iced::Task<CreateTaskModalMessage>> for CreateTaskModalAction {
	fn from(value: iced::Task<CreateTaskModalMessage>) -> Self {
		Self::Task(value)
	}
}

impl From<DatabaseMessage> for CreateTaskModalAction {
	fn from(value: DatabaseMessage) -> Self {
		Self::DatabaseMessage(value)
	}
}

pub enum CreateTaskModal {
	Opened {
		project_id: ProjectId,
		task_name: String,
		task_description: text_editor::Content,
		task_tags: HashSet<TaskTagId>,
		due_date: Option<SerializableDate>,
		edit_due_date: bool,
		needed_time_minutes: Option<Option<usize>>, // first option -> where to open ui, second -> valid time input
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
	pub fn update(&mut self, message: CreateTaskModalMessage, preferences: &Option<Preferences>) -> CreateTaskModalAction {
		match message {
			CreateTaskModalMessage::Open(project_id) => {
				*self = Self::Opened {
					project_id,
					task_name: String::new(),
					task_description: text_editor::Content::new(),
					task_tags: HashSet::new(),
					due_date: None,
					edit_due_date: false,
					needed_time_minutes: None,
				};
				text_input::focus(TASK_NAME_INPUT_ID.clone()).into()
			},
			CreateTaskModalMessage::Close => { *self = Self::Closed; CreateTaskModalAction::None },
			CreateTaskModalMessage::CreateTask => {
				let action = match self {
					CreateTaskModal::Opened {
						project_id,
						task_name,
						task_description,
						task_tags,
						due_date,
						needed_time_minutes,
						..
					} => DatabaseMessage::CreateTask{
						project_id: *project_id,
						task_id: TaskId::generate(),
						task_name: task_name.clone(),
						task_description: task_description.text(),
						task_tags: task_tags.clone(),
						due_date: *due_date,
						needed_time_minutes: needed_time_minutes.and_then(|needed_time| needed_time),
						time_spend: None,
						create_at_top: preferences.create_new_tasks_at_top(),
					}.into(),
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
			CreateTaskModalMessage::UnindentDescription => {
				if let CreateTaskModal::Opened { task_description, .. } = self {
					unindent_text(task_description);
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
			CreateTaskModalMessage::ChangeNeededTimeInput(new_input) => {
				if let CreateTaskModal::Opened { needed_time_minutes, .. } = self {
					*needed_time_minutes = Some(new_input);
				}
				CreateTaskModalAction::None
			},
			CreateTaskModalMessage::EditNeededTime => {
				if let CreateTaskModal::Opened { needed_time_minutes, .. } = self {
					*needed_time_minutes = Some(None);
				}
				CreateTaskModalAction::None
			},
			CreateTaskModalMessage::StopEditingNeededTime => {
				if let CreateTaskModal::Opened { needed_time_minutes, .. } = self {
					*needed_time_minutes = None;
				}
				CreateTaskModalAction::None
			},
			CreateTaskModalMessage::InvalidNeededTimeInput => CreateTaskModalAction::None,
			CreateTaskModalMessage::EditDueDate => {
				if let CreateTaskModal::Opened { edit_due_date, .. } = self {
					*edit_due_date = true;
				}
				CreateTaskModalAction::None
			},
			CreateTaskModalMessage::StopEditingDueDate => {
				if let CreateTaskModal::Opened { edit_due_date, .. } = self {
					*edit_due_date = false;
				}
				CreateTaskModalAction::None
			},
			CreateTaskModalMessage::ChangeDueDate(new_due_date) => {
				if let CreateTaskModal::Opened { due_date, edit_due_date, .. } = self {
					*due_date = Some(new_due_date);
					*edit_due_date = false;
				}
				CreateTaskModalAction::None
			},
			CreateTaskModalMessage::ClearDueDate => {
				if let CreateTaskModal::Opened { due_date, .. } = self {
					*due_date = None;
				}
				CreateTaskModalAction::None
			},
		}
	}

	pub fn view<'a>(&'a self, database: &'a Option<Database>, preferences: &'a Option<Preferences>) -> Option<Element<'a, CreateTaskModalMessage>> {
		match self {
			Self::Closed => None,
			Self::Opened { task_name, task_description, task_tags, project_id, due_date, edit_due_date, needed_time_minutes } => {
				let edit_needed_time_view: Element<'a, CreateTaskModalMessage> = if let Some(new_needed_time_minutes) = needed_time_minutes {
					let edit_needed_time_element = unfocusable(
						text_input(
							"mins",
							&match new_needed_time_minutes {
								Some(new_needed_time_minutes) => {
									format!("{new_needed_time_minutes}")
								}
								None => String::new(),
							},
						)
						.id(EDIT_NEEDED_TIME_INPUT_ID.clone())
						.width(Fixed(50.0))
						.on_input(move |input| on_number_input(
							input,
							CreateTaskModalMessage::ChangeNeededTimeInput,
							CreateTaskModalMessage::InvalidNeededTimeInput,
						))
						.style(move |t, s| {
							text_input_style(t, s, true, false, false, true)
						}),

						CreateTaskModalMessage::StopEditingNeededTime
					);

					row![
						edit_needed_time_element,
						clear_task_needed_time_button(CreateTaskModalMessage::StopEditingNeededTime)
					]
					.into()
				}
				else {
					edit_task_needed_time_button(
						None,
						CreateTaskModalMessage::EditNeededTime
					).into()
				};

				let add_due_date_button = add_due_date_button(CreateTaskModalMessage::EditDueDate);

				let due_date_view: Element<'a, CreateTaskModalMessage> = if *edit_due_date {
					date_picker(
						true,
						due_date.unwrap_or(date_picker::Date::today().into()),
						add_due_date_button,
						CreateTaskModalMessage::StopEditingDueDate,
						|date| CreateTaskModalMessage::ChangeDueDate(date.into())
					)
					.into()
				}
				else if let Some(due_date) = &due_date {
					let date_formatting = preferences.date_formatting();

					row![
						edit_due_date_button(due_date, date_formatting, CreateTaskModalMessage::EditDueDate),
						clear_task_due_date_button(CreateTaskModalMessage::ClearDueDate),
					]
					.into()
				}
				else {
					add_due_date_button.into()
				};

				Some(
					card(
						if let Some(project) = database.as_ref().and_then(|db| db.get_project(project_id)) {
							text(format!("Create Task in {}", project.name))
						}
						else {
							text("Create Task")
						}
						.size(LARGE_TEXT_SIZE),

						container(
							vertical_scrollable(
								column![
									if let Some(project) = database.as_ref().and_then(|db| db.get_project(project_id)) {
										let task_tags_list: Vec<Element<CreateTaskModalMessage>> = project.task_tags.iter()
											.map(|(tag_id, tag)| {
												task_tag_button(tag, task_tags.contains(&tag_id))
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
										.style(|t, s| text_input_style_borderless(t, s, true))
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
										.key_binding(|key_press| text_editor_keybindings(key_press, CreateTaskModalMessage::UnindentDescription)),

									Space::new(0.0, LARGE_SPACING_AMOUNT),

									row![
										edit_needed_time_view,
										due_date_view,
										Space::new(Fill, 0.0),
										create_new_task_modal_button(),
										close_create_new_task_modal_button()
									]
									.spacing(SPACING_AMOUNT)
									.width(Fill)
								]
							)
						)
						.padding(
							Padding::default()
								.bottom(SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT)
						)
					)
					.max_width(600.0)
					.close_size(LARGE_TEXT_SIZE)
					.on_close(CreateTaskModalMessage::Close)
					.style(card_style)
					.into(),
				)
			},
		}
	}
}