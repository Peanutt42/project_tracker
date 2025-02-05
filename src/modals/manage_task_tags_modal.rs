use crate::{
	components::{
		cancel_create_new_task_tag_button, color_palette, color_palette_item_button,
		create_new_task_tags_button, delete_task_tag_button, on_input, task_tag_name_button,
	},
	core::IcedColorConversion,
	project_tracker,
	styles::{card_style, text_input_style_only_round_left, LARGE_TEXT_SIZE, SMALL_SPACING_AMOUNT},
	ProjectTrackerApp,
};
use iced::{
	widget::{column, row, text, text_input, Column, Space},
	Alignment, Color, Element,
	Length::Fill,
	Task,
};
use iced_aw::{card, drop_down, DropDown};
use project_tracker_core::{
	Database, DatabaseMessage, Project, ProjectId, SerializableColor, TaskTag, TaskTagId,
};
use std::sync::LazyLock;

static CREATE_NEW_TASK_TAG_NAME_TEXT_INPUT_ID: LazyLock<text_input::Id> =
	LazyLock::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub enum Message {
	OpenCreateNewTaskTag,
	CloseCreateNewTaskTag,
	ChangeCreateNewTaskTagName(String),
	EditTaskTagColor(TaskTagId),
	EditTaskTagName(TaskTagId),
	ChangeEditTaskTagName(String),
	ChangeTaskTagName,
	ChangeTaskTagColor(Color),
	StopEditTaskTagColor,
	StopEditTaskTagName,
	CreateNewTaskTag,
	DeleteTaskTag(TaskTagId),
}

impl From<Message> for project_tracker::Message {
	fn from(value: Message) -> Self {
		project_tracker::Message::ManageTaskTagsModalMessage(value)
	}
}

pub enum Action {
	None,
	Task(Task<Message>),
	DatabaseMessage(DatabaseMessage),
}
impl From<Task<Message>> for Action {
	fn from(value: Task<Message>) -> Self {
		Self::Task(value)
	}
}
impl From<DatabaseMessage> for Action {
	fn from(value: DatabaseMessage) -> Self {
		Self::DatabaseMessage(value)
	}
}

pub struct Modal {
	project_id: ProjectId,
	create_new_task_tag: Option<String>,
	edit_task_tag_color_id: Option<TaskTagId>,
	edit_task_tag_name_id: Option<(TaskTagId, String)>,
}

impl Modal {
	pub fn new(project_id: ProjectId) -> Self {
		Self {
			project_id,
			create_new_task_tag: None,
			edit_task_tag_color_id: None,
			edit_task_tag_name_id: None,
		}
	}

	#[must_use]
	pub fn update(&mut self, message: Message, database: Option<&Database>) -> Action {
		match message {
			Message::OpenCreateNewTaskTag => {
				self.create_new_task_tag = Some(String::new());
				self.edit_task_tag_name_id = None;
				self.edit_task_tag_color_id = None;
				text_input::focus(CREATE_NEW_TASK_TAG_NAME_TEXT_INPUT_ID.clone()).into()
			}
			Message::CloseCreateNewTaskTag => {
				self.close_create_new_task_tag();
				Action::None
			}
			Message::ChangeCreateNewTaskTagName(new_name) => {
				self.create_new_task_tag = Some(new_name);
				Action::None
			}
			Message::EditTaskTagColor(task_tag_id) => {
				self.edit_task_tag_color_id = Some(task_tag_id);
				self.update(Message::StopEditTaskTagName, database)
			}
			Message::EditTaskTagName(task_tag_id) => {
				let task_tag_name = database
					.as_ref()
					.and_then(|database| {
						database.get_project(&self.project_id).and_then(|project| {
							project
								.task_tags
								.get(&task_tag_id)
								.map(|task_tag| task_tag.name.clone())
						})
					})
					.unwrap_or_default();
				self.edit_task_tag_name_id = Some((task_tag_id, task_tag_name));
				self.update(Message::StopEditTaskTagColor, database)
			}
			Message::ChangeTaskTagColor(new_color) => {
				let action = match &self.edit_task_tag_color_id {
					Some(edit_task_tag_id) => DatabaseMessage::ChangeTaskTagColor {
						project_id: self.project_id,
						task_tag_id: *edit_task_tag_id,
						new_color: SerializableColor::from_iced_color(new_color),
					}
					.into(),
					None => Action::None,
				};
				self.stop_editing_task_tag_color();
				action
			}
			Message::ChangeEditTaskTagName(new_name) => {
				if let Some((_edit_task_tag_id, edit_task_tag_name)) =
					&mut self.edit_task_tag_name_id
				{
					*edit_task_tag_name = new_name;
				}
				Action::None
			}
			Message::ChangeTaskTagName => {
				let action = match &mut self.edit_task_tag_name_id {
					Some((edit_task_tag_id, new_name)) => DatabaseMessage::ChangeTaskTagName {
						project_id: self.project_id,
						task_tag_id: *edit_task_tag_id,
						new_name: std::mem::take(new_name),
					}
					.into(),
					None => Action::None,
				};
				self.stop_editing_task_tag_name();
				action
			}
			Message::StopEditTaskTagColor => {
				self.stop_editing_task_tag_color();
				Action::None
			}
			Message::StopEditTaskTagName => {
				self.edit_task_tag_name_id = None;
				Action::None
			}
			Message::CreateNewTaskTag => {
				let action = match &mut self.create_new_task_tag {
					Some(new_task_tag_name) => DatabaseMessage::CreateTaskTag {
						project_id: self.project_id,
						task_tag_id: TaskTagId::generate(),
						task_tag: TaskTag::new(
							std::mem::take(new_task_tag_name),
							SerializableColor::from_iced_color(Color::WHITE),
						),
					}
					.into(),
					None => Action::None,
				};
				self.close_create_new_task_tag();
				action
			}
			Message::DeleteTaskTag(task_tag_id) => DatabaseMessage::DeleteTaskTag {
				project_id: self.project_id,
				task_tag_id,
			}
			.into(),
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, project_tracker::Message> {
		match app
			.database
			.ok()
			.and_then(|db| db.get_project(&self.project_id))
		{
			Some(project) => card(
				text(format!("Manage project '{}' task tags", project.name)).size(LARGE_TEXT_SIZE),
				self.tags_list_view(
					project,
					&self.create_new_task_tag,
					&self.edit_task_tag_color_id,
					&self.edit_task_tag_name_id,
				)
				.map(project_tracker::Message::ManageTaskTagsModalMessage),
			)
			.style(card_style)
			.max_width(500.0)
			.close_size(LARGE_TEXT_SIZE)
			.on_close(project_tracker::Message::CloseManageTaskTagsModal)
			.into(),
			None => Space::new(0.0, 0.0).into(),
		}
	}

	fn stop_editing_task_tag_color(&mut self) {
		self.edit_task_tag_color_id = None;
	}

	fn stop_editing_task_tag_name(&mut self) {
		self.edit_task_tag_name_id = None;
	}

	fn close_create_new_task_tag(&mut self) {
		self.create_new_task_tag = None;
	}

	fn tags_list_view<'a>(
		&'a self,
		project: &'a Project,
		create_new_task_tag: &'a Option<String>,
		edit_task_tag_color_id: &'a Option<TaskTagId>,
		edit_task_tag_name_id: &'a Option<(TaskTagId, String)>,
	) -> Element<'a, Message> {
		let mut tags_list: Vec<Element<'a, Message>> = Vec::new();
		for (tag_id, tag) in project.task_tags.iter() {
			let show_color_palette = match edit_task_tag_color_id {
				Some(edit_task_tag_color_id) => tag_id == *edit_task_tag_color_id,
				None => false,
			};

			let edited_name = match edit_task_tag_name_id {
				Some((edit_task_tag_name_id, new_name)) if tag_id == *edit_task_tag_name_id => {
					Some(new_name)
				}
				_ => None,
			};

			let name_element: Element<Message> = match edited_name {
				Some(edited_name) => text_input("tag name", edited_name)
					.on_input(Message::ChangeEditTaskTagName)
					.on_submit(Message::ChangeTaskTagName)
					.style(text_input_style_only_round_left)
					.into(),
				None => task_tag_name_button(tag_id, &tag.name).into(),
			};

			let color_picker = DropDown::new(
				color_palette_item_button(
					tag.color.to_iced_color(),
					false,
					true,
					true,
					true,
					true,
					if show_color_palette {
						Message::StopEditTaskTagColor
					} else {
						Message::EditTaskTagColor(tag_id)
					},
				),
				color_palette(tag.color.to_iced_color(), move |new_color| {
					Message::ChangeTaskTagColor(new_color)
				}),
				show_color_palette,
			)
			.width(Fill)
			.alignment(drop_down::Alignment::End)
			.on_dismiss(Message::StopEditTaskTagColor);

			tags_list.push(
				column![
					row![color_picker, name_element, delete_task_tag_button(tag_id),]
						.align_y(Alignment::Center)
						.spacing(SMALL_SPACING_AMOUNT)
				]
				.into(),
			);
		}

		tags_list.push(match create_new_task_tag {
			Some(create_new_task_tag_name) => row![
				on_input(
					text_input("New tag name", create_new_task_tag_name)
						.id(CREATE_NEW_TASK_TAG_NAME_TEXT_INPUT_ID.clone())
						.on_input(Message::ChangeCreateNewTaskTagName)
						.on_submit(Message::CreateNewTaskTag)
						.style(text_input_style_only_round_left)
				)
				.on_esc(Message::CloseCreateNewTaskTag),
				cancel_create_new_task_tag_button(),
			]
			.into(),
			None => create_new_task_tags_button().into(),
		});

		Column::with_children(tags_list)
			.spacing(SMALL_SPACING_AMOUNT)
			.into()
	}
}
