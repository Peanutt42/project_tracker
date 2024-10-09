use std::str::FromStr;
use iced::{advanced::graphics::core::font, alignment::{Horizontal, Vertical}, widget::{column, container, markdown, row, stack, text, text_editor, text_input, Row, Space}, Element, Font, Length::{Fill, Fixed}};
use iced_aw::{card, date_picker};
use once_cell::sync::Lazy;
use crate::{components::{add_due_date_button, clear_task_due_date_button, clear_task_needed_time_button, delete_task_button, edit_due_date_button, edit_task_description_button, edit_task_name_button, edit_task_needed_time_button, horizontal_scrollable, start_task_timer_button, stop_editing_task_description_button, task_tag_button, unfocusable}, core::{Database, DatabaseMessage, OptionalPreference, ProjectId, SerializableDate, TaskId}, project_tracker::Message, styles::{card_style, link_color, markdown_background_container_style, text_editor_keybindings, text_editor_style, text_input_style, text_input_style_default, HEADING_TEXT_SIZE, LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE, PADDING_AMOUNT, SPACING_AMOUNT}, ProjectTrackerApp};

static TASK_NAME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static EDIT_NEEDED_TIME_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub enum TaskModalMessage {
	Open{
		project_id: ProjectId,
		task_id: TaskId,
	},
	Close,

	EditName,
	StopEditingName,

	EditDescription,
	StopEditingDescription,
	EditDescriptionAction(text_editor::Action),

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

pub enum TaskModal {
	Opened {
		project_id: ProjectId,
		task_id: TaskId,
		edit_name: bool,
		new_description: Option<text_editor::Content>,
		edit_due_date: bool,
		new_needed_time_minutes: Option<Option<usize>>,
	},
	Closed,
}

impl TaskModal {
	pub fn update<'a>(&'a mut self, message: TaskModalMessage, database: &'a mut Option<Database>) -> iced::Task<TaskModalMessage> {
		match message {
			TaskModalMessage::Open { project_id, task_id } => {
				*self = TaskModal::Opened {
					project_id,
					task_id,
					edit_name: false,
					new_description: None,
					edit_due_date: false,
					new_needed_time_minutes: None
				};
				text_input::focus(TASK_NAME_INPUT_ID.clone())
			},
			TaskModalMessage::Close => {
				*self = TaskModal::Closed;
				iced::Task::none()
			},

			TaskModalMessage::EditName => {
				if let TaskModal::Opened { edit_name, .. } = self {
					*edit_name = true;
				}
				text_input::focus(TASK_NAME_INPUT_ID.clone())
			},
			TaskModalMessage::StopEditingName => {
				if let TaskModal::Opened { edit_name, .. } = self {
					*edit_name = false;
				}
				iced::Task::none()
			},

			TaskModalMessage::EditDescription => {
				if let TaskModal::Opened { project_id, task_id, new_description, .. } = self {
					*new_description = database.as_ref().and_then(|db|
						db.get_task(project_id, task_id)
							.map(|task| text_editor::Content::with_text(task.description()))
					);
				}
				iced::Task::none()
			},
			TaskModalMessage::StopEditingDescription => {
				if let TaskModal::Opened { new_description, .. } = self {
					*new_description = None;
				}
				iced::Task::none()
			},
			TaskModalMessage::EditDescriptionAction(action) => {
				if let TaskModal::Opened { project_id, task_id, new_description: Some(new_description),.. } = self {
					new_description.perform(action);
					if let Some(database) = database {
						database.update(DatabaseMessage::ChangeTaskDescription {
							project_id: *project_id,
							task_id: *task_id,
							new_task_description: new_description.text(),
						});
					}
				}
				iced::Task::none()
			},

			TaskModalMessage::EditDueDate => {
				if let TaskModal::Opened { edit_due_date, .. } = self {
					*edit_due_date = true;
				}
				iced::Task::none()
			},
			TaskModalMessage::StopEditingDueDate => {
				if let TaskModal::Opened { edit_due_date, .. } = self {
					*edit_due_date = false;
				}
				iced::Task::none()
			},
			TaskModalMessage::ChangeDueDate(new_due_date) => {
				if let TaskModal::Opened { project_id, task_id, edit_due_date, .. } = self {
					*edit_due_date = false;

					if let Some(database) = database {
						database.update(DatabaseMessage::ChangeTaskDueDate {
							project_id: *project_id,
							task_id: *task_id,
							new_due_date: Some(new_due_date)
						});
					}
				}

				iced::Task::none()
			},

			TaskModalMessage::EditNeededTime => {
				if let TaskModal::Opened { project_id, task_id, new_needed_time_minutes, .. } = self {
					let previous_task_needed_minutes = database.as_ref().and_then(|db| {
						db.get_task(project_id, task_id)
							.and_then(|task| task.needed_time_minutes)
					});
					*new_needed_time_minutes = Some(previous_task_needed_minutes);
				}
				text_input::focus(EDIT_NEEDED_TIME_INPUT_ID.clone())
			},
			TaskModalMessage::StopEditingNeededTime => {
				if let TaskModal::Opened { new_needed_time_minutes, .. } = self {
					*new_needed_time_minutes = None;
				}
				iced::Task::none()
			},
			TaskModalMessage::ChangeNeededTimeInput(new_edited_needed_time_minutes) => {
				if let TaskModal::Opened { new_needed_time_minutes, .. } = self {
					*new_needed_time_minutes = Some(new_edited_needed_time_minutes);
				}
				iced::Task::none()
			},
			TaskModalMessage::ChangeNeededTime => {
				if let TaskModal::Opened { project_id, task_id, new_needed_time_minutes, .. } = self {
					if let Some(new_needed_time_minutes) = new_needed_time_minutes {
						if let Some(database) = database {
							database.update(DatabaseMessage::ChangeTaskNeededTime {
								project_id: *project_id,
								task_id: *task_id,
								new_needed_time_minutes: *new_needed_time_minutes
							});
						}
					}
					*new_needed_time_minutes = None;
				}
				iced::Task::none()
			},
			TaskModalMessage::ClearTaskNeededTime => {
				if let TaskModal::Opened { project_id, task_id, new_needed_time_minutes, .. } = self {
					*new_needed_time_minutes = None;
					if let Some(database) = database {
						database.update(DatabaseMessage::ChangeTaskNeededTime {
							project_id: *project_id,
							task_id: *task_id,
							new_needed_time_minutes: None
						});
					}
				}
				iced::Task::none()
			},
			TaskModalMessage::InvalidNeededTimeInput => iced::Task::none(),
			TaskModalMessage::DeleteTask => {
				if let TaskModal::Opened { project_id, task_id, .. } = self {
					if let Some(database) = database {
						database.update(DatabaseMessage::DeleteTask {
							project_id: *project_id,
							task_id: *task_id,
						});
					}
				}
				*self = TaskModal::Closed;
				iced::Task::none()
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Option<Element<'a, Message>> {
		match self {
			Self::Closed => None,
			Self::Opened { project_id, task_id, edit_name, new_description, edit_due_date, new_needed_time_minutes } => Some(
				card(
					Space::new(0.0, 0.0),

					if let Some(project) = app.database.as_ref().and_then(|db| db.get_project(project_id)) {
						if let Some(task) = project.get_task(task_id) {
							let task_tags_list: Vec<Element<Message>> = project.task_tags.iter()
								.map(|(task_tag_id, task_tag)| {
									task_tag_button(task_tag, task.tags.contains(&task_tag_id), true, true)
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
									.on_input(move |input| {
										let new_needed_time_minutes = match usize::from_str(&input) {
											Ok(new_needed_time_minutes) => {
												Some(Some(new_needed_time_minutes))
											}
											Err(_) => {
												if input.is_empty() {
													Some(None)
												} else {
													None
												}
											}
										};
										match new_needed_time_minutes {
											Some(new_needed_time_minutes) => {
												TaskModalMessage::ChangeNeededTimeInput(
													new_needed_time_minutes,
												)
												.into()
											}
											None => TaskModalMessage::InvalidNeededTimeInput.into(),
										}
									})
									.on_submit(TaskModalMessage::ChangeNeededTime.into())
									.style(move |t, s| {
										text_input_style(t, s, true, false, false, true)
									}),

									TaskModalMessage::StopEditingNeededTime.into()
								);

								row![
									edit_needed_time_element,
									clear_task_needed_time_button()
								]
								.into()
							}
							else {
								edit_task_needed_time_button(task.needed_time_minutes).into()
							};

							let needed_time_view = Row::new()
								.push_maybe(task.needed_time_minutes.as_ref().map(|_| {
									start_task_timer_button(*project_id, *task_id)
								}))
								.push(edit_needed_time_view);

							let date_formatting = app.preferences.date_formatting();

							let add_due_date_button = add_due_date_button();

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
									edit_due_date_button(due_date, date_formatting),
									clear_task_due_date_button(*project_id, *task_id),
								]
								.into()
							}
							else {
								add_due_date_button.into()
							};

							let description_hover_button: Element<'a, Message> = container(
								if new_description.is_some() {
									stop_editing_task_description_button()
								}
								else {
									edit_task_description_button()
								}
							)
							.width(Fill)
							.height(Fill)
							.align_x(Horizontal::Right)
							.align_y(Vertical::Top)
							//.padding(PADDING_AMOUNT)
							.into();

							let description_text: Element<'a, Message> = if let Some(new_description) = new_description {
								text_editor(new_description)
									.on_action(|action| TaskModalMessage::EditDescriptionAction(action).into())
									.style(move |t, s| {
										text_editor_style(t, s, true, true, true, true)
									})
									.key_binding(text_editor_keybindings)
									.into()
							}
							else {
								container(
									if task.description_markdown_items().is_empty() {
										text("No description")
											.width(Fill)
											.into()
									}
									else {
										markdown(
											task.description_markdown_items(),
											markdown::Settings::default(),
											markdown::Style {
												link_color: link_color(app.is_theme_dark()),
												..markdown::Style::from_palette(app.theme().palette())
											}
										)
										.map(|markdown_url| Message::OpenUrl(markdown_url.to_string()))
									}
								)
								.padding(PADDING_AMOUNT)
								.style(markdown_background_container_style)
								.into()
							};

							let bold_font = Font {
								weight: font::Weight::Bold,
								..Default::default()
							};

							let name_text: Element<'a, Message> = if *edit_name {
								unfocusable(
									text_input("Input task name", task.name())
										.id(TASK_NAME_INPUT_ID.clone())
										.on_input(|new_task_name| DatabaseMessage::ChangeTaskName {
											project_id: *project_id,
											task_id: *task_id,
											new_task_name
										}
										.into())
										.on_submit(TaskModalMessage::StopEditingName.into())
										.style(text_input_style_default)
										.size(HEADING_TEXT_SIZE)
										.font(bold_font),

									TaskModalMessage::StopEditingName.into()
								)
								.into()
							}
							else {
								stack![
									container(
										text(task.name())
											.size(HEADING_TEXT_SIZE)
											.font(bold_font)
									)
									.width(Fill),

									container(edit_task_name_button())
										.width(Fill)
										.height(Fill)
										.align_x(Horizontal::Right)
										.align_y(Vertical::Top)
								]
								.into()
							};

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
									description_text,
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