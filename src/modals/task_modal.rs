use iced::{alignment::{Horizontal, Vertical}, widget::{column, container, row, stack, text, text_editor, text_input, Row, Space}, Element, Length::{Fill, Fixed}, Padding};
use iced_aw::{card, date_picker};
use once_cell::sync::Lazy;
use crate::{components::{add_due_date_button, clear_task_due_date_button, clear_task_needed_time_button, delete_task_button, edit_due_date_button, edit_task_description_button, edit_task_needed_time_button, horizontal_scrollable, start_task_timer_button, task_description, task_tag_button, unfocusable, vertical_scrollable, view_task_description_button, ICON_BUTTON_WIDTH, SCROLLBAR_WIDTH}, core::{Database, DatabaseMessage, OptionalPreference, ProjectId, SerializableDate, TaskId}, project_tracker::Message, styles::{card_style, description_text_editor_style, markdown_background_container_style, on_number_input, text_editor_keybindings, text_input_style, text_input_style_borderless, tooltip_container_style, unindent_text, BOLD_FONT, HEADING_TEXT_SIZE, LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE, PADDING_AMOUNT, SMALL_PADDING_AMOUNT, SPACING_AMOUNT}, ProjectTrackerApp};

static TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static EDIT_NEEDED_TIME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub enum TaskModalMessage {
	Open{
		project_id: ProjectId,
		task_id: TaskId,
	},
	Close,

	EditDescription,
	ViewDescription,
	EditDescriptionAction(text_editor::Action),
	UnindentDescription,

	EditDueDate,
	StopEditingDueDate,
	ChangeDueDate(SerializableDate),

	EditNeededTime,
	StopEditingNeededTime,
	ClearTaskNeededTime,
	ChangeNeededTimeInput(Option<usize>),
	ChangeNeededTime,
	InvalidNeededTimeInput,

	DeleteTask,
}

impl From<TaskModalMessage> for Message {
	fn from(value: TaskModalMessage) -> Self {
		Message::TaskModalMessage(value)
	}
}

pub enum TaskModalAction {
	None,
	Task(iced::Task<TaskModalMessage>),
	DatabaseMessage(DatabaseMessage),
}

impl From<iced::Task<TaskModalMessage>> for TaskModalAction {
	fn from(value: iced::Task<TaskModalMessage>) -> Self {
		TaskModalAction::Task(value)
	}
}
impl From<DatabaseMessage> for TaskModalAction {
	fn from(value: DatabaseMessage) -> Self {
		TaskModalAction::DatabaseMessage(value)
	}
}

pub enum TaskModal {
	Opened {
		project_id: ProjectId,
		task_id: TaskId,
		new_description: Option<text_editor::Content>,
		edit_due_date: bool,
		new_needed_time_minutes: Option<Option<usize>>,
	},
	Closed,
}

impl TaskModal {
	pub fn update<'a>(&'a mut self, message: TaskModalMessage, database: &'a Option<Database>) -> TaskModalAction {
		match message {
			TaskModalMessage::Open { project_id, task_id } => {
				*self = TaskModal::Opened {
					project_id,
					task_id,
					new_description: None,
					edit_due_date: false,
					new_needed_time_minutes: None
				};
				text_input::focus(TASK_NAME_INPUT_ID.clone()).into()
			},
			TaskModalMessage::Close => {
				*self = TaskModal::Closed;
				TaskModalAction::None
			},

			TaskModalMessage::EditDescription => {
				if let TaskModal::Opened { project_id, task_id, new_description, .. } = self {
					*new_description = database.as_ref().and_then(|db|
						db.get_task(project_id, task_id)
							.map(|task| text_editor::Content::with_text(task.description()))
					);
				}
				TaskModalAction::None
			},
			TaskModalMessage::ViewDescription => {
				if let TaskModal::Opened { new_description, .. } = self {
					*new_description = None;
				}
				TaskModalAction::None
			},
			TaskModalMessage::EditDescriptionAction(action) => {
				if let TaskModal::Opened { project_id, task_id, new_description: Some(new_description),.. } = self {
					new_description.perform(action);
					DatabaseMessage::ChangeTaskDescription {
						project_id: *project_id,
						task_id: *task_id,
						new_task_description: new_description.text(),
					}
					.into()
				}
				else {
					TaskModalAction::None
				}
			},
			TaskModalMessage::UnindentDescription => {
				if let TaskModal::Opened { new_description: Some(new_description), project_id, task_id, .. } = self {
					unindent_text(new_description);
					DatabaseMessage::ChangeTaskDescription {
						project_id: *project_id,
						task_id: *task_id,
						new_task_description: new_description.text(),
					}
					.into()
				}
				else {
					TaskModalAction::None
				}
			}

			TaskModalMessage::EditDueDate => {
				if let TaskModal::Opened { edit_due_date, .. } = self {
					*edit_due_date = true;
				}
				TaskModalAction::None
			},
			TaskModalMessage::StopEditingDueDate => {
				if let TaskModal::Opened { edit_due_date, .. } = self {
					*edit_due_date = false;
				}
				TaskModalAction::None
			},
			TaskModalMessage::ChangeDueDate(new_due_date) => {
				if let TaskModal::Opened { project_id, task_id, edit_due_date, .. } = self {
					*edit_due_date = false;
					DatabaseMessage::ChangeTaskDueDate {
						project_id: *project_id,
						task_id: *task_id,
						new_due_date: Some(new_due_date)
					}
					.into()
				}
				else {
					TaskModalAction::None
				}
			},

			TaskModalMessage::EditNeededTime => {
				if let TaskModal::Opened { project_id, task_id, new_needed_time_minutes, .. } = self {
					let previous_task_needed_minutes = database.as_ref().and_then(|db| {
						db.get_task(project_id, task_id)
							.and_then(|task| task.needed_time_minutes)
					});
					*new_needed_time_minutes = Some(previous_task_needed_minutes);
				}
				text_input::focus(EDIT_NEEDED_TIME_INPUT_ID.clone()).into()
			},
			TaskModalMessage::StopEditingNeededTime => {
				if let TaskModal::Opened { new_needed_time_minutes, .. } = self {
					*new_needed_time_minutes = None;
				}
				TaskModalAction::None
			},
			TaskModalMessage::ChangeNeededTimeInput(new_edited_needed_time_minutes) => {
				if let TaskModal::Opened { new_needed_time_minutes, .. } = self {
					*new_needed_time_minutes = Some(new_edited_needed_time_minutes);
				}
				TaskModalAction::None
			},
			TaskModalMessage::ChangeNeededTime => {
				if let TaskModal::Opened { project_id, task_id, new_needed_time_minutes, .. } = self {
					if let Some(new_needed_time_minutes_clone) = *new_needed_time_minutes {
						*new_needed_time_minutes = None;
						return DatabaseMessage::ChangeTaskNeededTime {
							project_id: *project_id,
							task_id: *task_id,
							new_needed_time_minutes: new_needed_time_minutes_clone
						}
						.into();
					}
				}
				TaskModalAction::None
			},
			TaskModalMessage::ClearTaskNeededTime => {
				if let TaskModal::Opened { project_id, task_id, new_needed_time_minutes, .. } = self {
					*new_needed_time_minutes = None;
					DatabaseMessage::ChangeTaskNeededTime {
						project_id: *project_id,
						task_id: *task_id,
						new_needed_time_minutes: None
					}
					.into()
				}
				else {
					TaskModalAction::None
				}
			},
			TaskModalMessage::InvalidNeededTimeInput => TaskModalAction::None,
			TaskModalMessage::DeleteTask => {
				let action = if let TaskModal::Opened { project_id, task_id, .. } = self {
					DatabaseMessage::DeleteTask {
						project_id: *project_id,
						task_id: *task_id,
					}
					.into()
				}
				else {
					TaskModalAction::None
				};
				*self = TaskModal::Closed;
				action
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Option<Element<'a, Message>> {
		match self {
			Self::Closed => None,
			Self::Opened { project_id, task_id, new_description, edit_due_date, new_needed_time_minutes } => Some(
				card(
					Space::new(0.0, 0.0),

					if let Some(project) = app.database.as_ref().and_then(|db| db.get_project(project_id)) {
						if let Some(task) = project.get_task(task_id) {
							let task_tags_list: Vec<Element<Message>> = project.task_tags.iter()
								.map(|(task_tag_id, task_tag)| {
									task_tag_button(task_tag, task.tags.contains(&task_tag_id))
										.on_press(DatabaseMessage::ToggleTaskTag{
											project_id: *project_id,
											task_id: *task_id,
											task_tag_id
										}
										.into())
										.into()
								})
								.collect();

							let edit_needed_time_view: Element<'a, Message> = if let Some(new_needed_time_minutes) = new_needed_time_minutes {
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
										|new_time| TaskModalMessage::ChangeNeededTimeInput(new_time).into(),
										TaskModalMessage::InvalidNeededTimeInput.into(),
									))
									.on_submit(TaskModalMessage::ChangeNeededTime.into())
									.style(move |t, s| {
										text_input_style(t, s, true, false, false, true)
									}),

									TaskModalMessage::StopEditingNeededTime.into()
								);

								row![
									edit_needed_time_element,
									clear_task_needed_time_button(TaskModalMessage::ClearTaskNeededTime.into())
								]
								.into()
							}
							else {
								edit_task_needed_time_button(
									task.needed_time_minutes,
									TaskModalMessage::EditNeededTime.into()
								).into()
							};

							let needed_time_view = Row::new()
								.push_maybe(task.needed_time_minutes.as_ref().map(|_| {
									start_task_timer_button(*project_id, *task_id)
								}))
								.push(edit_needed_time_view);

							let date_formatting = app.preferences.date_formatting();

							let add_due_date_button = add_due_date_button(TaskModalMessage::EditDueDate.into());

							let due_date_view: Element<'a, Message> = if *edit_due_date {
								date_picker(
									true,
									task.due_date.unwrap_or(date_picker::Date::today().into()),
									add_due_date_button,
									TaskModalMessage::StopEditingDueDate.into(),
									move |date| TaskModalMessage::ChangeDueDate(date.into()).into()
								)
								.into()
							}
							else if let Some(due_date) = &task.due_date {
								row![
									edit_due_date_button(due_date, date_formatting, TaskModalMessage::EditDueDate.into()),
									clear_task_due_date_button(
										DatabaseMessage::ChangeTaskDueDate {
											project_id: *project_id,
											task_id: *task_id,
											new_due_date: None
										}.into()
									),
								]
								.into()
							}
							else {
								add_due_date_button.into()
							};

							let editing_description = new_description.is_some();
							let viewing_description = !editing_description;
							let description_hover_button: Element<'a, Message> = container(
								container(
									row![
										view_task_description_button(viewing_description),
										edit_task_description_button(editing_description),
									]
								)
								.style(tooltip_container_style)
							)
							.width(Fill)
							.height(Fill)
							.align_x(Horizontal::Right)
							.align_y(Vertical::Top)
							.into();

							let description_text: Element<'a, Message> = if let Some(new_description) = new_description {
								text_editor(new_description)
									.on_action(|action| TaskModalMessage::EditDescriptionAction(action).into())
									.style(description_text_editor_style)
									.key_binding(|key_press| text_editor_keybindings(key_press, TaskModalMessage::UnindentDescription.into()))
									.padding(PADDING_AMOUNT)
									.into()
							}
							else {
								task_description(task, app)
							};

							let name_text: Element<'a, Message> = text_input("Input task name", task.name())
								.id(TASK_NAME_INPUT_ID.clone())
								.on_input(|new_task_name| DatabaseMessage::ChangeTaskName {
									project_id: *project_id,
									task_id: *task_id,
									new_task_name
								}
								.into())
								.style(|t, s| text_input_style_borderless(t, s, true))
								.size(HEADING_TEXT_SIZE)
								.font(BOLD_FONT)
								.into();

							container(
								vertical_scrollable(
									column![
										Space::new(0.0, SPACING_AMOUNT),

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
										},

										name_text,

										Space::new(0.0, LARGE_SPACING_AMOUNT),

										stack![
											container(description_text)
												.padding(Padding::ZERO.right(ICON_BUTTON_WIDTH * 2.0))
												.style(markdown_background_container_style),
											description_hover_button
										],

										Space::new(0.0, LARGE_SPACING_AMOUNT),

										row![
											needed_time_view,
											due_date_view,
											Space::new(Fill, 0.0),
											delete_task_button(),
										]
										.spacing(SPACING_AMOUNT)
									]
								)
							)
							.padding(
								Padding::default()
									.bottom(SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT)
							)
							.into()
						}
						else {
							text("<invalid task id>").into()
						}
					}
					else {
						Element::new(text("<invalid project id>"))
					}
				)
				.max_width(600.0)
				.close_size(LARGE_TEXT_SIZE)
				.on_close(TaskModalMessage::Close.into())
				.style(card_style)
				.into(),
			),
		}
	}
}