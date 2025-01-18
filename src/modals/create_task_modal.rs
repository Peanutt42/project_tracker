use crate::{
	components::{
		close_create_new_task_modal_button, create_new_task_modal_button, due_date_button,
		edit_needed_time_button, horizontal_scrollable, task_description_editor, task_tag_button,
		vertical_scrollable, SCROLLBAR_WIDTH,
	},
	core::SerializableDateConversion,
	project_tracker,
	styles::{
		card_style, text_input_style_borderless, unindent_text, HEADING_TEXT_SIZE,
		LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE, SMALL_PADDING_AMOUNT, SPACING_AMOUNT,
	},
	OptionalPreference, Preferences,
};
use iced::{
	font,
	widget::{column, container, row, text, text_editor, text_input, Row, Space},
	Element, Font,
	Length::Fill,
	Padding,
};
use iced_aw::card;
use project_tracker_core::{
	duration_to_minutes, parse_duration_from_str, Database, ProjectId, SerializableDate, TaskId,
	TaskTagId, TimeSpend,
};
use std::collections::BTreeSet;
use std::sync::LazyLock;
use tracing::error;

static TASK_NAME_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);
static EDIT_NEEDED_TIME_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub enum Message {
	CreateTask,
	ChangeTaskName(String),
	TaskDescriptionAction(text_editor::Action),
	UnindentDescription,
	ToggleTaskTag(TaskTagId),
	ChangeNeededTimeInput(String),
	EditNeededTime,
	StopEditingNeededTime,
	InvalidNeededTimeInput,
	EditDueDate,
	StopEditingDueDate,
	ChangeDueDate(SerializableDate),
	ClearDueDate,
}

impl From<Message> for project_tracker::Message {
	fn from(value: Message) -> Self {
		project_tracker::Message::CreateTaskModalMessage(value)
	}
}

pub enum Action {
	None,
	Task(iced::Task<Message>),
	CreateTask {
		project_id: ProjectId,
		task_id: TaskId,
		task_name: String,
		task_description: String,
		task_tags: BTreeSet<TaskTagId>,
		due_date: Option<SerializableDate>,
		needed_time_minutes: Option<usize>,
		time_spend: Option<TimeSpend>,
		create_at_top: bool,
	},
}

impl From<iced::Task<Message>> for Action {
	fn from(value: iced::Task<Message>) -> Self {
		Self::Task(value)
	}
}

pub struct Modal {
	project_id: ProjectId,
	task_name: String,
	task_description: text_editor::Content,
	task_tags: BTreeSet<TaskTagId>,
	due_date: Option<SerializableDate>,
	edit_due_date: bool,
	needed_time_minutes: Option<String>,
}

impl Modal {
	pub fn new(project_id: ProjectId) -> Self {
		Self {
			project_id,
			task_name: String::new(),
			task_description: text_editor::Content::new(),
			task_tags: BTreeSet::new(),
			due_date: None,
			edit_due_date: false,
			needed_time_minutes: None,
		}
	}

	#[must_use]
	pub fn update(&mut self, message: Message, preferences: &Option<Preferences>) -> Action {
		match message {
			Message::CreateTask => Action::CreateTask {
				project_id: self.project_id,
				task_id: TaskId::generate(),
				task_name: self.task_name.clone(),
				task_description: self.task_description.text(),
				task_tags: self.task_tags.clone(),
				due_date: self.due_date,
				needed_time_minutes: self.needed_time_minutes.as_ref().and_then(|needed_time| {
					parse_duration_from_str(needed_time).map(duration_to_minutes)
				}),
				time_spend: None,
				create_at_top: preferences.create_new_tasks_at_top(),
			},
			Message::ChangeTaskName(new_task_name) => {
				self.task_name = new_task_name;
				Action::None
			}
			Message::TaskDescriptionAction(action) => {
				self.task_description.perform(action);
				Action::None
			}
			Message::UnindentDescription => {
				unindent_text(&mut self.task_description);
				Action::None
			}
			Message::ToggleTaskTag(task_tag_id) => {
				if self.task_tags.contains(&task_tag_id) {
					self.task_tags.remove(&task_tag_id);
				} else {
					self.task_tags.insert(task_tag_id);
				}
				Action::None
			}
			Message::ChangeNeededTimeInput(new_input) => {
				self.needed_time_minutes = Some(new_input);
				Action::None
			}
			Message::EditNeededTime => {
				self.needed_time_minutes = Some(String::new());
				Action::None
			}
			Message::StopEditingNeededTime => {
				self.needed_time_minutes = None;
				Action::None
			}
			Message::InvalidNeededTimeInput => Action::None,
			Message::EditDueDate => {
				self.edit_due_date = true;
				Action::None
			}
			Message::StopEditingDueDate => {
				self.edit_due_date = false;
				Action::None
			}
			Message::ChangeDueDate(new_due_date) => {
				self.due_date = Some(new_due_date);
				self.edit_due_date = false;
				Action::None
			}
			Message::ClearDueDate => {
				self.due_date = None;
				Action::None
			}
		}
	}

	pub fn view<'a>(
		&'a self,
		database: Option<&'a Database>,
		preferences: &'a Option<Preferences>,
	) -> Element<'a, project_tracker::Message> {
		let edit_needed_time_view: Element<project_tracker::Message> = edit_needed_time_button(
			None,
			&self.needed_time_minutes,
			Message::EditNeededTime,
			Message::ChangeNeededTimeInput,
			None,
			Message::StopEditingNeededTime,
			Message::StopEditingNeededTime,
			EDIT_NEEDED_TIME_INPUT_ID.clone(),
		)
		.map(project_tracker::Message::CreateTaskModalMessage);

		let date_formatting = preferences.date_formatting();
		let due_date_view: Element<project_tracker::Message> = due_date_button(
			self.edit_due_date,
			&self.due_date,
			date_formatting,
			Message::EditDueDate,
			Message::StopEditingDueDate,
			|date| Message::ChangeDueDate(SerializableDate::from_iced_date(date)),
			Message::ClearDueDate,
		)
		.map(project_tracker::Message::CreateTaskModalMessage);

		card(
			match database
				.as_ref()
				.and_then(|db| db.get_project(&self.project_id))
			{
				Some(project) => text(format!("Create Task in {}", project.name)),
				None => text("Create Task"),
			}
			.size(LARGE_TEXT_SIZE),
			container(vertical_scrollable(column![
				match database
					.as_ref()
					.and_then(|db| db.get_project(&self.project_id))
				{
					Some(project) => {
						let task_tags_list: Vec<Element<project_tracker::Message>> = project
							.task_tags
							.iter()
							.map(|(tag_id, tag)| {
								task_tag_button(tag, self.task_tags.contains(&tag_id))
									.on_press(Message::ToggleTaskTag(tag_id).into())
									.into()
							})
							.collect();

						if task_tags_list.is_empty() {
							Element::new(Space::new(0.0, 0.0))
						} else {
							horizontal_scrollable(
								Row::with_children(task_tags_list).spacing(SPACING_AMOUNT),
							)
							.width(Fill)
							.into()
						}
					}
					None => {
						error!("invalid project_id: doesnt exist in database!");
						Element::new(text("<invalid project id>"))
					}
				},
				text_input("task name", &self.task_name)
					.id(TASK_NAME_INPUT_ID.clone())
					.on_input(|name| Message::ChangeTaskName(name).into())
					.on_submit(Message::CreateTask.into())
					.style(|t, s| text_input_style_borderless(t, s, true))
					.size(HEADING_TEXT_SIZE)
					.font(Font {
						weight: font::Weight::Bold,
						..Default::default()
					}),
				Space::new(0.0, SPACING_AMOUNT),
				text("Description:"),
				task_description_editor(
					&self.task_description,
					|action| { Message::TaskDescriptionAction(action) },
					None,
					Message::UnindentDescription
				)
				.map(project_tracker::Message::CreateTaskModalMessage),
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
			]))
			.padding(Padding::default().bottom(SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT)),
		)
		.max_width(600.0)
		.close_size(LARGE_TEXT_SIZE)
		.on_close(project_tracker::Message::CloseCreateTaskModal)
		.style(card_style)
		.into()
	}
}
